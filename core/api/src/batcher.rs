// This module is experimental and may be subject to major changes.


use crate::agent::{ConnectionEvent, DroppedPacketMetric, LatencyMetric};

pub enum MetricsBatcher {
    LatencyMetrics,
    DroppedPacketsMetrics,
}
pub enum EventBatcher {}

impl MetricsBatcher {
    pub async fn send_batched_metrics() {
        todo!();
    }
}

impl EventBatcher {
    pub async fn send_batched_logs() {
        todo!();
    }
}
