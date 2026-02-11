use aya::util::online_cpus;
use cortexbrain_common::map_handlers::map_manager;
use cortexbrain_common::{
    buffer_type::{BufferSize, BufferType, read_perf_buffer},
    map_handlers::BpfMapsData,
};
use tokio::signal;
use tracing::{error, info};

pub async fn event_listener(bpf_maps: BpfMapsData) -> Result<(), anyhow::Error> {
    info!("Getting CPU count...");

    let mut maps = map_manager(bpf_maps)?;

    let cpu_count = online_cpus().map_err(|e| anyhow::anyhow!("Error {:?}", e))?;

    for cpu_id in cpu_count {
        for (name, (perf_event_array, perf_event_buffer)) in maps.iter_mut() {
            let buf = perf_event_array.open(cpu_id, None)?;
            perf_event_buffer.push(buf);
        }
    }

    info!("Perf buffers created successfully");

    let (time_stamp_events_array, time_stamp_events_perf_buffer) = maps
        .remove("time_stamp_events")
        .expect("Cannot create time_stamp_events_buffer");
    let (net_perf_array, net_perf_buffer) = maps
        .remove("net_metrics")
        .expect("Cannot create net_perf_buffer");

    // Create proper sized buffers
    let net_metrics_buffers = BufferSize::NetworkMetricsEvents.set_buffer();
    let time_stamp_events_buffers = BufferSize::TimeMetricsEvents.set_buffer();

    info!("Starting event listener tasks...");
    let metrics_map_displayer = tokio::spawn(async move {
        read_perf_buffer(
            net_perf_buffer,
            net_metrics_buffers,
            BufferType::NetworkMetrics,
        )
        .await;
    });

    let time_stamp_events_displayer = tokio::spawn(async move {
        read_perf_buffer(
            time_stamp_events_perf_buffer,
            time_stamp_events_buffers,
            BufferType::TimeStampMetrics,
        )
        .await;
    });

    info!("Event listeners started, entering main loop...");

    tokio::select! {
        result = metrics_map_displayer => {
            if let Err(e) = result {
                error!("Metrics map displayer task failed: {:?}", e);
            }
        }

        result = time_stamp_events_displayer => {
            if let Err(e) = result {
                error!("Time stamp events displayer task failed: {:?}", e);
            }
        }

        _ = signal::ctrl_c() => {
            info!("Ctrl-C received, shutting down...");
        }
    }

    // return success
    Ok(())
}
