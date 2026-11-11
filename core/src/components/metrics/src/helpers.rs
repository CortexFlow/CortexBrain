use anyhow::anyhow;
use aya::{
    Ebpf,
    programs::{ProgramError, UProbe},
    util::online_cpus,
};
use cortexbrain_common::{
    buffer_type::BufferSize,
    constants,
    consumer::{Consumer, read_perf_buffer},
    map_handlers::{BpfMapsData, map_manager},
    otel_metrics::Metrics,
    service_cache::ServiceCache,
};
use opentelemetry::metrics::Meter;
use std::{
    collections::HashSet,
    env, fs,
    os::unix::fs::MetadataExt,
    path::Path,
    sync::{Arc, Mutex, Once},
    time::Duration,
};
use tokio::{signal, time};
use tracing::{debug, error, info, warn};

/// Locate the OpenSSL shared library used for the SSL uprobes.
///
/// Resolution order:
///   1. The `LIBSSL_PATH` environment variable, if set. An explicit path that
///      does not exist is treated as an error so misconfiguration is surfaced.
///   2. The default system library directory, checking each candidate in turn.
///
/// Returns `Ok(None)` when no usable library can be found so the caller can
/// skip SSL tracing gracefully.
pub fn resolve_libssl_path() -> anyhow::Result<Option<String>> {
    let candidates = [
        "libssl.so.3",
        "libssl.so",
        "libssl.so.1.1",
        "libssl.so.1.0.0",
    ];

    if let Ok(path) = env::var(constants::LIBSSL_PATH) {
        if !path.is_empty() && Path::new(&path).is_file() {
            return Ok(Some(path));
        }
        return Err(anyhow!(
            "LIBSSL_PATH is set but does not point to an existing file: {}",
            path
        ));
    }

    const LIB_DIRS: [&str; 4] = [
        "/usr/lib/x86_64-linux-gnu",
        "/usr/lib64",
        "/usr/lib/aarch64-linux-gnu",
        "/lib/x86_64-linux-gnu",
    ];

    for dir in LIB_DIRS {
        for name in candidates {
            let path = format!("{}/{}", dir, name);
            if Path::new(&path).is_file() {
                return Ok(Some(path));
            }
        }
    }

    Ok(None)
}

/// Initialise the shared OpenTelemetry metrics handle and the Docker/K8s
/// service cache used by every perf-buffer consumer.
pub async fn init_metrics_and_cache(
    meter: &Meter,
) -> Result<(Arc<Metrics>, Arc<tokio::sync::RwLock<ServiceCache>>), anyhow::Error> {
    let metrics = Arc::new(Metrics::new(meter));

    let mut cache_obj = ServiceCache { service_map: None };
    cache_obj.init();
    cache_obj.populate_map_with_pod_info().await?;
    let cache = Arc::new(tokio::sync::RwLock::new(cache_obj));

    Ok((metrics, cache))
}

const LIBSSL_NAMES: [&str; 4] = [
    "libssl.so.3",
    "libssl.so.1.1",
    "libssl.so.1.0.0",
    "libssl.so",
];

const LIBSSL_DIRS: [&str; 8] = [
    "/usr/lib/x86_64-linux-gnu",
    "/usr/lib64",
    "/usr/lib/aarch64-linux-gnu",
    "/lib/x86_64-linux-gnu",
    "/lib64",
    "/usr/lib",
    "/lib",
    "/usr/local/lib",
];

const SSL_UPROBES: [(&str, &str); 4] = [
    ("ssl_read", "SSL_read"),
    ("ssl_read_ret", "SSL_read"),
    ("ssl_write", "SSL_write"),
    ("ssl_write_ret", "SSL_write"),
];

static INVALID_LIBSSL_PATH: Once = Once::new();

fn resolve_libssl_paths() -> Vec<String> {
    let mut paths = Vec::new();

    if let Ok(path) = env::var(constants::LIBSSL_PATH) {
        if Path::new(&path).is_file() {
            paths.push(path);
        } else if !path.is_empty() {
            INVALID_LIBSSL_PATH.call_once(|| warn!("Invalid LIBSSL_PATH: {}", path));
        }
    }

    let Some(proc_dir) = ["/host/proc", "/proc"]
        .into_iter()
        .find(|dir| Path::new(dir).is_dir())
    else {
        return paths;
    };
    let Ok(entries) = fs::read_dir(proc_dir) else {
        return paths;
    };

    let mut roots = HashSet::new();
    for entry in entries.flatten() {
        let Ok(pid) = entry.file_name().to_string_lossy().parse::<u32>() else {
            continue;
        };
        let pid = pid.to_string();
        let root = Path::new(proc_dir).join(&pid).join("root");
        let Ok(metadata) = fs::metadata(&root) else {
            continue;
        };
        if !roots.insert((metadata.dev(), metadata.ino())) {
            continue;
        }

        for dir in LIBSSL_DIRS {
            for name in LIBSSL_NAMES {
                let path = root.join(dir.trim_start_matches('/')).join(name);
                if path.is_file() {
                    paths.push(path.to_string_lossy().into_owned());
                }
            }
        }
    }

    paths
}

fn attach_uprobe(
    bpf: &Arc<Mutex<Ebpf>>,
    program_name: &str,
    symbol: &str,
    target: &str,
) -> Result<(), anyhow::Error> {
    let mut bpf = bpf
        .lock()
        .map_err(|e| anyhow::anyhow!("Cannot get value from lock. Reason: {}", e))?;
    let program: &mut UProbe = bpf
        .program_mut(program_name)
        .ok_or_else(|| anyhow::anyhow!("Program {} not found", program_name))?
        .try_into()
        .map_err(|e| anyhow::anyhow!("Failed to convert program: {:?}", e))?;
    match program.load() {
        Ok(()) | Err(ProgramError::AlreadyLoaded) => {}
        Err(e) => {
            return Err(anyhow::anyhow!(
                "Cannot load program: {}. Error: {}",
                program_name,
                e
            ));
        }
    }
    program
        .attach(Some(symbol), 0, target, None)
        .map_err(|e| anyhow::anyhow!("Failed to attach {} to {}: {:?}", program_name, target, e))?;
    info!(
        "{} program attached successfully to user space function {}",
        program_name, target
    );
    Ok(())
}

fn attach_ssl_uprobes(bpf: &Arc<Mutex<Ebpf>>, attached: &mut HashSet<(u64, u64)>) {
    for path in resolve_libssl_paths() {
        let Ok(metadata) = fs::metadata(&path) else {
            continue;
        };
        if !attached.insert((metadata.dev(), metadata.ino())) {
            continue;
        }
        for (program, symbol) in SSL_UPROBES {
            if let Err(e) = attach_uprobe(bpf, program, symbol, &path) {
                debug!("SSL uprobe {} -> {}: {}", program, path, e);
            }
        }
    }
}

/// Listen for eBPF perf-buffer events and record OpenTelemetry metrics.
///
/// This function bridges the eBPF perf-buffer layer with the OpenTelemetry
/// metrics pipeline.  It opens per-CPU buffers for the two maps of interest
/// (`net_metrics` and `time_stamp_events`), spawns asynchronous consumers,
/// and parks until a `Ctrl-C` signal is received or one of the consumers
/// terminates.
///
/// # Arguments
///
/// -`bpf_maps` – handles for the pinned BPF maps produced by
///   [`cortexbrain_common::map_handlers::map_pinner`].
/// - `meter`    – an initialised OpenTelemetry [`Meter`].
///
/// # Errors
///
/// Returns `Err` if the map manager or CPU enumeration fails.
///
pub async fn event_listener(bpf_maps: BpfMapsData, meter: Meter) -> Result<(), anyhow::Error> {
    info!("Getting CPU count...");

    let bpf = Arc::clone(&bpf_maps.bpf);
    let mut maps = map_manager(bpf_maps)?;

    let cpu_count = online_cpus().map_err(|e| anyhow::anyhow!("Error {:?}", e))?;

    for cpu_id in cpu_count {
        for (name, (perf_event_array, perf_event_buffer)) in maps.iter_mut() {
            let buf = perf_event_array.open(cpu_id, None).map_err(|e| {
                anyhow!(
                    "Cannot create perf_event_array buffer from perf_event_array. Reason: {}",
                    e
                )
            })?;
            info!(
                "Buffer created for map {:?} on cpu_id {:?}. Buffer size: {}",
                name,
                cpu_id,
                std::mem::size_of_val(&buf)
            );
            perf_event_buffer.push(buf);
        }
    }

    info!("Perf buffers created successfully");

    let (_time_stamp_events_array, time_stamp_events_perf_buffer) = maps
        .remove("time_stamp_events")
        .expect("Cannot create time_stamp_events_buffer");
    let (_net_perf_array, net_perf_buffer) = maps
        .remove("net_metrics")
        .expect("Cannot create net_perf_buffer");

    let (_cpu_frequency_events_array, cpu_frequency_perf_buffer) = maps
        .remove("cpu_frequency")
        .expect("Cannot create cpu_frequency_perf_buffer");

    let (_cpu_idle_array, cpu_idle_perf_buffer) = maps
        .remove("cpu_idle")
        .expect("Cannot create cpu_idle perf buffer");

    let (_mem_alloc_array, mem_alloc_perf_buffer) = maps
        .remove("mem_alloc")
        .expect("Cannot create mem_alloc perf buffer");

    let (_sched_stat_wait_array, sched_stat_wait_perf_buffer) = maps
        .remove("sched_stat_wait")
        .expect("Cannot create sched_stat_wait perf buffer");

    let (_sched_stat_runtime_array, sched_stat_runtime_perf_buffer) = maps
        .remove("sched_stat_runtime")
        .expect("Cannot create sched_stat_runtime perf buffer");

    let (_ssl_events, ssl_events_perf_buffer) = maps
        .remove("ssl_events")
        .expect("Cannot create ssl_events perf buffer");

    // Allocate byte-buffers sized for each structure type
    let net_metrics_buffers = BufferSize::NetworkMetricsEvents.set_buffer();
    let time_stamp_events_buffers = BufferSize::TimeMetricsEvents.set_buffer();
    let cpu_frequency_events_buffers = BufferSize::CpuFrequency.set_buffer();
    let cpu_idle_buffers = BufferSize::CpuIdle.set_buffer();
    let mem_alloc_buffers = BufferSize::MemAlloc.set_buffer();
    let sched_stat_wait_buffers = BufferSize::SchedStatWait.set_buffer();
    let sched_stat_runtime_buffers = BufferSize::SchedStatRuntime.set_buffer();
    let ssl_events_buffers = BufferSize::SslEvents.set_buffer();

    let (metrics, cache) = init_metrics_and_cache(&meter).await?;

    info!("Starting event listener tasks...");

    let cache_refresher = Arc::clone(&cache);
    let cache_handle = {
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(60));
            // every 60 seconds the cache auto updates and qu
            loop {
                interval.tick().await;
                // scan the cache once again to check if we need to update it
                // acquire cache write lock
                let mut cache_lock = cache_refresher.write().await;
                if let Err(e) = cache_lock.populate_map_with_pod_info().await {
                    error!("cache refresh fail. {}", e);
                };
            }
        })
    };

    let mut attached = HashSet::new();
    if let Ok(Some(path)) = resolve_libssl_path() {
        if let Ok(metadata) = fs::metadata(&path) {
            attached.insert((metadata.dev(), metadata.ino()));
        }
    }
    let _ssl_handle = {
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                attach_ssl_uprobes(&bpf, &mut attached);
            }
        })
    };

    let net_metrics_handle = {
        let metrics = Arc::clone(&metrics);
        let service_cache = Arc::clone(&cache);
        let array_buffers = net_perf_buffer;
        let buffers = net_metrics_buffers;
        tokio::spawn(async move {
            read_perf_buffer(
                array_buffers,
                buffers,
                Consumer::PacketLossMetrics,
                Some(metrics),
                Some(service_cache),
            )
            .await;
        })
    };

    let time_stamp_handle = {
        let metrics = Arc::clone(&metrics);
        let service_cache = Arc::clone(&cache);
        let array_buffers = time_stamp_events_perf_buffer;
        let buffers = time_stamp_events_buffers;
        tokio::spawn(async move {
            read_perf_buffer(
                array_buffers,
                buffers,
                Consumer::TimeStampMetrics,
                Some(metrics),
                Some(service_cache),
            )
            .await;
        })
    };

    let cpu_frequency_metrics = {
        let metrics = Arc::clone(&metrics);
        let service_cache = Arc::clone(&cache);
        let array_buffers = cpu_frequency_perf_buffer;
        let buffers = cpu_frequency_events_buffers;
        tokio::spawn(async move {
            read_perf_buffer(
                array_buffers,
                buffers,
                Consumer::CpuFrequency,
                Some(metrics),
                Some(service_cache),
            )
            .await;
        })
    };

    let cpu_idle_metrics = {
        let metrics = Arc::clone(&metrics);
        let service_cache = Arc::clone(&cache);
        let array_buffers = cpu_idle_perf_buffer;
        let buffers = cpu_idle_buffers;
        tokio::spawn(async move {
            read_perf_buffer(
                array_buffers,
                buffers,
                Consumer::CpuIdle,
                Some(metrics),
                Some(service_cache),
            )
            .await;
        })
    };

    let mem_alloc_metrics = {
        let metrics = Arc::clone(&metrics);
        let service_cache = Arc::clone(&cache);
        let array_buffers = mem_alloc_perf_buffer;
        let buffers = mem_alloc_buffers;
        tokio::spawn(async move {
            read_perf_buffer(
                array_buffers,
                buffers,
                Consumer::MemAlloc,
                Some(metrics),
                Some(service_cache),
            )
            .await;
        })
    };

    let sched_stat_wait_metrics = {
        let metrics = Arc::clone(&metrics);
        let service_cache = Arc::clone(&cache);

        let array_buffers = sched_stat_wait_perf_buffer;
        let buffers = sched_stat_wait_buffers;
        tokio::spawn(async move {
            read_perf_buffer(
                array_buffers,
                buffers,
                Consumer::SchedStatWait,
                Some(metrics),
                Some(service_cache),
            )
            .await;
        })
    };

    let sched_stat_runtime_metrics = {
        let metrics = Arc::clone(&metrics);
        let service_cache = Arc::clone(&cache);
        let array_buffers = sched_stat_runtime_perf_buffer;
        let buffers = sched_stat_runtime_buffers;
        tokio::spawn(async move {
            read_perf_buffer(
                array_buffers,
                buffers,
                Consumer::SchedStatRuntime,
                Some(metrics),
                Some(service_cache),
            )
            .await;
        })
    };

    let ssl_events_metrics = {
        let metrics = Arc::clone(&metrics);
        let service_cache = Arc::clone(&cache);
        let array_buffers = ssl_events_perf_buffer;
        let buffers = ssl_events_buffers;
        tokio::spawn(async move {
            read_perf_buffer(
                array_buffers,
                buffers,
                Consumer::SslEvents,
                Some(metrics),
                Some(service_cache),
            )
            .await;
        })
    };

    info!("Event listeners started, entering main loop...");

    tokio::select! {
        result = cache_handle => {
            if let Err(e) = result {
                error!("Cache handle task failed: {:?}", e);
            }
        }

        result = net_metrics_handle => {
            if let Err(e) = result {
                error!("Network metrics task failed: {:?}", e);
            }
        }

        result = time_stamp_handle => {
            if let Err(e) = result {
                error!("Timestamp events task failed: {:?}", e);
            }
        }

        result = cpu_frequency_metrics => {
            if let Err(e) = result {
                error!("Cpu frequency events task failed: {:?}", e);
            }
        }

        result = cpu_idle_metrics => {
            if let Err(e) = result {
                error!("CpuIdle events task failed: {:?}", e);
            }
        }

        result = mem_alloc_metrics => {
            if let Err(e) = result {
                error!("MemAlloc events task failed: {:?}", e);
            }
        }

        result = sched_stat_wait_metrics => {
            if let Err(e) = result {
                error!("SchedStatWait events task failed: {:?}", e);
            }
        }

        result = sched_stat_runtime_metrics => {
            if let Err(e) = result {
                error!("SchedStatRuntime events task failed: {:?}", e);
            }
        }

        result = ssl_events_metrics => {
            if let Err(e) = result {
                error!("Ssl events task failed: {:?}", e);
            }
        }

        _ = signal::ctrl_c() => {
            info!("Ctrl-C received, shutting down...");
        }
    }

    Ok(())
}
