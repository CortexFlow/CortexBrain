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

    // Allocate byte-buffers sized for each structure type
    let net_metrics_buffers = BufferSize::NetworkMetricsEvents.set_buffer();
    let time_stamp_events_buffers = BufferSize::TimeMetricsEvents.set_buffer();

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

        _ = signal::ctrl_c() => {
            info!("Ctrl-C received, shutting down...");
        }
    }

    Ok(())
}
