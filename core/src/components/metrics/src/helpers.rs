use aya::{
    maps::{
        MapData,
        perf::{PerfEventArrayBuffer},
    }
};

use bytes::BytesMut;
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