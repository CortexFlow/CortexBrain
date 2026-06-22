use anyhow::anyhow;
use aya::util::online_cpus;
use cortexbrain_common::map_handlers::map_manager;
use cortexbrain_common::{buffer_type::BufferSize, map_handlers::BpfMapsData};
use opentelemetry::metrics::Meter;
use std::sync::Arc;
use tokio::signal;
use tracing::{error, info};

use cortexbrain_common::buffer_type::{BufferType, read_perf_buffer};
use cortexbrain_common::otel_metrics::Metrics;

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

    // Allocate byte-buffers sized for each structure type
    let net_metrics_buffers = BufferSize::NetworkMetricsEvents.set_buffer();
    let time_stamp_events_buffers = BufferSize::TimeMetricsEvents.set_buffer();
    let cpu_frequency_events_buffers = BufferSize::CpuFrequency.set_buffer();
    let cpu_idle_buffers = BufferSize::CpuIdle.set_buffer();
    let mem_alloc_buffers = BufferSize::MemAlloc.set_buffer();
    let sched_stat_wait_buffers = BufferSize::SchedStatWait.set_buffer();
    let sched_stat_runtime_buffers = BufferSize::SchedStatRuntime.set_buffer();

    let metrics = Arc::new(Metrics::new(&meter));

    info!("Starting event listener tasks...");

    let net_metrics_handle = {
        let metrics = Arc::clone(&metrics);
        let mut array_buffers = net_perf_buffer;
        let mut buffers = net_metrics_buffers;
        tokio::spawn(async move {
            read_perf_buffer(
                array_buffers,
                buffers,
                BufferType::NetworkMetrics,
                Some(metrics),
            )
            .await;
        })
    };

    let time_stamp_handle = {
        let metrics = Arc::clone(&metrics);
        let mut array_buffers = time_stamp_events_perf_buffer;
        let mut buffers = time_stamp_events_buffers;
        tokio::spawn(async move {
            read_perf_buffer(
                array_buffers,
                buffers,
                BufferType::TimeStampMetrics,
                Some(metrics),
            )
            .await;
        })
    };

    let cpu_frequency_metrics = {
        let metrics = Arc::clone(&metrics);
        let mut array_buffers = cpu_frequency_perf_buffer;
        let mut buffers = cpu_frequency_events_buffers;
        tokio::spawn(async move {
            read_perf_buffer(
                array_buffers,
                buffers,
                BufferType::CpuFrequency,
                Some(metrics),
            )
            .await;
        })
    };

    let cpu_idle_metrics = {
        let metrics = Arc::clone(&metrics);
        let mut array_buffers = cpu_idle_perf_buffer;
        let mut buffers = cpu_idle_buffers;
        tokio::spawn(async move {
            read_perf_buffer(array_buffers, buffers, BufferType::CpuIdle, Some(metrics)).await;
        })
    };

    let mem_alloc_metrics = {
        let metrics = Arc::clone(&metrics);
        let mut array_buffers = mem_alloc_perf_buffer;
        let mut buffers = mem_alloc_buffers;
        tokio::spawn(async move {
            read_perf_buffer(array_buffers, buffers, BufferType::MemAlloc, Some(metrics)).await;
        })
    };

    let sched_stat_wait_metrics = {
        let metrics = Arc::clone(&metrics);
        let mut array_buffers = sched_stat_wait_perf_buffer;
        let mut buffers = sched_stat_wait_buffers;
        tokio::spawn(async move {
            read_perf_buffer(
                array_buffers,
                buffers,
                BufferType::SchedStatWait,
                Some(metrics),
            )
            .await;
        })
    };

    let sched_stat_runtime_metrics = {
        let metrics = Arc::clone(&metrics);
        let mut array_buffers = sched_stat_runtime_perf_buffer;
        let mut buffers = sched_stat_runtime_buffers;
        tokio::spawn(async move {
            read_perf_buffer(
                array_buffers,
                buffers,
                BufferType::SchedStatRuntime,
                Some(metrics),
            )
            .await;
        })
    };

    info!("Event listeners started, entering main loop...");

    tokio::select! {
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

        _ = signal::ctrl_c() => {
            info!("Ctrl-C received, shutting down...");
        }
    }

    Ok(())
}
