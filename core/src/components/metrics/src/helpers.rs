use aya::{maps::{
        perf::PerfEventArrayBuffer, Map, MapData, PerfEventArray
    }, util::online_cpus};

use bytes::BytesMut;
use tokio::signal;
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use tracing::{error, info};

use crate::structs::NetworkMetrics;
use crate::structs::TimeStampMetrics;

pub async fn display_metrics_map(
    mut perf_buffers: Vec<PerfEventArrayBuffer<MapData>>,
    running: Arc<AtomicBool>,  // Changed to Arc<AtomicBool>
    mut buffers: Vec<BytesMut>,
) {
    info!("Starting metrics event listener...");
    while running.load(Ordering::SeqCst) {
        for buf in perf_buffers.iter_mut() {
            match buf.read_events(&mut buffers) {
                std::result::Result::Ok(events) => {
                    if events.read > 0 {
                        info!("Read {} metric events", events.read);
                    }
                    for i in 0..events.read {
                        let data = &buffers[i];
                        if data.len() >= std::mem::size_of::<NetworkMetrics>() {
                            let net_metrics: NetworkMetrics =
                                unsafe { std::ptr::read_unaligned(data.as_ptr() as *const _) };
                            let sk_drop_count = net_metrics.sk_drops;
                            let sk_err = net_metrics.sk_err;
                            let sk_err_soft = net_metrics.sk_err_soft;
                            let sk_backlog_len = net_metrics.sk_backlog_len;
                            let sk_write_memory_queued = net_metrics.sk_write_memory_queued;
                            let sk_ack_backlog = net_metrics.sk_ack_backlog;
                            let sk_receive_buffer_size = net_metrics.sk_receive_buffer_size;
                            info!(
                                "sk_drops: {}, sk_err: {}, sk_err_soft: {}, sk_backlog_len: {}, sk_write_memory_queued: {}, sk_ack_backlog: {}, sk_receive_buffer_size: {}",
                                sk_drop_count, sk_err, sk_err_soft, sk_backlog_len, sk_write_memory_queued, sk_ack_backlog, sk_receive_buffer_size
                            );
                        } else {
                            info!("Received data too small: {} bytes, expected: {}", data.len(), std::mem::size_of::<NetworkMetrics>());
                        }
                    }
                }
                Err(e) => {
                    error!("Error reading events: {:?}", e);
                }
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    info!("Metrics event listener stopped");
}

pub async fn display_time_stamp_events_map(
    mut perf_buffers: Vec<PerfEventArrayBuffer<MapData>>,
    running: Arc<AtomicBool>,  // Changed to Arc<AtomicBool>
    mut buffers: Vec<BytesMut>,
) {
    info!("Starting timestamp event listener...");
    while running.load(Ordering::SeqCst) {
        for buf in perf_buffers.iter_mut() {
            match buf.read_events(&mut buffers) {
                std::result::Result::Ok(events) => {
                    if events.read > 0 {
                        info!("Read {} timestamp events", events.read);
                    }
                    for i in 0..events.read {
                        let data = &buffers[i];
                        if data.len() >= std::mem::size_of::<TimeStampMetrics>() {
                            let time_stamp_event: TimeStampMetrics =
                                unsafe { std::ptr::read_unaligned(data.as_ptr() as *const _) };
                            let delta_us = time_stamp_event.delta_us;
                            let ts_us = time_stamp_event.ts_us;
                            let tgid = time_stamp_event.tgid;
                            let comm = String::from_utf8_lossy(&time_stamp_event.comm);
                            let lport = time_stamp_event.lport;
                            let dport_be = time_stamp_event.dport_be;
                            let af = time_stamp_event.af;
                            info!(
                                "TimeStampEvent - delta_us: {}, ts_us: {}, tgid: {}, comm: {}, lport: {}, dport_be: {}, af: {}",
                                delta_us, ts_us, tgid, comm, lport, dport_be, af
                            );
                        } else {
                            info!("Received timestamp data too small: {} bytes", data.len());
                        }
                    }
                }
                Err(e) => {
                    error!("Error reading timestamp events: {:?}", e);
                }
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    info!("Timestamp event listener stopped");
}

pub async fn event_listener(bpf_maps: (Map, Map)) -> Result<(), anyhow::Error> {
    info!("Getting CPU count...");
    let cpu_count = online_cpus().map_err(|e| anyhow::anyhow!("Error {:?}", e))?.len();
    info!("CPU count: {}", cpu_count);
    
    info!("Creating perf buffers...");
    let mut net_perf_buffer: Vec<PerfEventArrayBuffer<MapData>> = Vec::new();
    let mut net_perf_array: PerfEventArray<MapData> = PerfEventArray::try_from(bpf_maps.0)?;
    let mut time_stamp_events_perf_buffer: Vec<PerfEventArrayBuffer<MapData>> = Vec::new();
    let mut time_stamp_events_perf_array: PerfEventArray<MapData> =
        PerfEventArray::try_from(bpf_maps.1)?;

    info!("Opening perf buffers for {} CPUs...", cpu_count);
    for cpu_id in online_cpus().map_err(|e| anyhow::anyhow!("Error {:?}", e))? {
        let buf: PerfEventArrayBuffer<MapData> = net_perf_array.open(cpu_id, None)?;
        net_perf_buffer.push(buf);
    }
    for cpu_id in online_cpus().map_err(|e| anyhow::anyhow!("Error {:?}", e))? {
        let buf: PerfEventArrayBuffer<MapData> = time_stamp_events_perf_array.open(cpu_id, None)?;
        time_stamp_events_perf_buffer.push(buf);
    }
    info!("Perf buffers created successfully");

    // Create shared running flags
    let net_metrics_running = Arc::new(AtomicBool::new(true));
    let time_stamp_events_running = Arc::new(AtomicBool::new(true));
    
    // Create proper sized buffers
    let net_metrics_buffers = vec![BytesMut::with_capacity(1024); cpu_count];
    let time_stamp_events_buffers = vec![BytesMut::with_capacity(1024); cpu_count];
    
    // Clone for the signal handler
    let net_metrics_running_signal = net_metrics_running.clone();
    let time_stamp_events_running_signal = time_stamp_events_running.clone();
    
    info!("Starting event listener tasks...");
    let metrics_map_displayer = tokio::spawn(async move {
        display_metrics_map(net_perf_buffer, net_metrics_running, net_metrics_buffers).await;
    });

    let time_stamp_events_displayer = tokio::spawn(async move {
        display_time_stamp_events_map(time_stamp_events_perf_buffer, time_stamp_events_running, time_stamp_events_buffers).await
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
            // Stop the event loops
            net_metrics_running_signal.store(false, std::sync::atomic::Ordering::SeqCst);
            time_stamp_events_running_signal.store(false, std::sync::atomic::Ordering::SeqCst);
        }
    }

    // return success
    Ok(())
}