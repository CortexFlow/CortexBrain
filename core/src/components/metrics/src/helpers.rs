use anyhow::anyhow;
use aya::util::online_cpus;
use cortexbrain_common::map_handlers::map_manager;
use cortexbrain_common::{buffer_type::BufferSize, map_handlers::BpfMapsData};
use opentelemetry::metrics::Meter;
use std::env;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::{signal, time};
use tracing::{error, info};

use cortexbrain_common::constants;
use cortexbrain_common::consumer::{Consumer, read_perf_buffer};
use cortexbrain_common::otel_metrics::Metrics;
use cortexbrain_common::service_cache::ServiceCache;

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

    let metrics = Arc::new(Metrics::new(&meter));
    let mut cache_obj = ServiceCache { service_map: None };
    cache_obj.init(); //init cache
    cache_obj.populate_map_with_pod_info().await?; // populate cache with a first scan of the services 
    let cache = Arc::new(tokio::sync::RwLock::new(cache_obj));

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

    let net_metrics_handle = {
        let metrics = Arc::clone(&metrics);
        let service_cache = Arc::clone(&cache);
        let mut array_buffers = net_perf_buffer;
        let mut buffers = net_metrics_buffers;
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
        let mut array_buffers = time_stamp_events_perf_buffer;
        let mut buffers = time_stamp_events_buffers;
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
        let mut array_buffers = cpu_frequency_perf_buffer;
        let mut buffers = cpu_frequency_events_buffers;
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
        let mut array_buffers = cpu_idle_perf_buffer;
        let mut buffers = cpu_idle_buffers;
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
        let mut array_buffers = mem_alloc_perf_buffer;
        let mut buffers = mem_alloc_buffers;
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

        let mut array_buffers = sched_stat_wait_perf_buffer;
        let mut buffers = sched_stat_wait_buffers;
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
        let mut array_buffers = sched_stat_runtime_perf_buffer;
        let mut buffers = sched_stat_runtime_buffers;
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
        let mut array_buffers = ssl_events_perf_buffer;
        let mut buffers = ssl_events_buffers;
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
