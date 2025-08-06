use aya::{
    maps::{
        MapData,
        perf::{PerfEventArrayBuffer},
    }
};

use bytes::BytesMut;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
    },
};

use tracing::{error, info};

use crate::structs::NetworkMetrics;

pub async fn display_metrics_map(
    mut perf_buffers: Vec<PerfEventArrayBuffer<MapData>>,
    running: AtomicBool,
    mut buffers: Vec<BytesMut>,
) {
    while running.load(Ordering::SeqCst) {
        for buf in perf_buffers.iter_mut() {
            match buf.read_events(&mut buffers) {
                std::result::Result::Ok(events) => {
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
}
