// This module is experimental and may be subject to major changes.


// Do not use any of these functions 
// FIXME: this module will be deprecated in the next version probably


use tokio::sync::mpsc;
use tonic::{Status, async_trait};

use crate::{
    agent::{ConnectionEvent, DroppedPacketMetric, LatencyMetric, VethEvent},
    api::AgentApi,
};

// Event sender trait. Takes an event from a map and send that to the mpsc channel
// using the send_map function
#[async_trait]
pub trait EventSender: Send + Sync + 'static {
    async fn send_active_connection_event(&self, event: Vec<ConnectionEvent>);
    async fn send_active_connection_event_map(
        &self,
        map: Vec<ConnectionEvent>,
        tx: mpsc::Sender<Result<Vec<ConnectionEvent>, Status>>,
    ) {
        let status = Status::new(tonic::Code::Ok, "success");
        let event = Ok(map);

        let _ = tx.send(event).await;
    }

    async fn send_latency_metrics_event(&self, event: Vec<LatencyMetric>);
    async fn send_latency_metrics_event_map(
        &self,
        map: Vec<LatencyMetric>,
        tx: mpsc::Sender<Result<Vec<LatencyMetric>, Status>>,
    ) {
        let status = Status::new(tonic::Code::Ok, "success");
        let event = Ok(map);
        let _ = tx.send(event).await;
    }

    async fn send_dropped_packet_metrics_event(&self, event: Vec<DroppedPacketMetric>);
    async fn send_dropped_packet_metrics_event_map(
        &self,
        map: Vec<DroppedPacketMetric>,
        tx: mpsc::Sender<Result<Vec<DroppedPacketMetric>, Status>>,
    ) {
        let status = Status::new(tonic::Code::Ok, "success");
        let event = Ok(map);
        let _ = tx.send(event).await;
    }

    async fn send_tracked_veth_event(&self, event: Vec<VethEvent>);
    async fn send_tracked_veth_event_map(
        &self,
        map: Vec<VethEvent>,
        tx: mpsc::Sender<Result<Vec<VethEvent>, Status>>,
    ) {
        let status = Status::new(tonic::Code::Ok, "success");
        let event = Ok(map);
        let _ = tx.send(event).await;
    }
}

// send event function. takes an HashMap and send that using mpsc event_tx
#[async_trait]
impl EventSender for AgentApi {
    async fn send_active_connection_event(&self, event: Vec<ConnectionEvent>) {
        self.send_active_connection_event_map(event, self.active_connection_event_tx.clone())
            .await;
    }

    async fn send_latency_metrics_event(&self, event: Vec<LatencyMetric>) {
        self.send_latency_metrics_event_map(event, self.latency_metrics_tx.clone())
            .await;
    }

    async fn send_dropped_packet_metrics_event(&self, event: Vec<DroppedPacketMetric>) {
        self.send_dropped_packet_metrics_event_map(event, self.dropped_packet_metrics_tx.clone())
            .await;
    }
    async fn send_tracked_veth_event(&self, event: Vec<VethEvent>) {
        self.send_tracked_veth_event_map(event, self.tracked_veth_tx.clone())
            .await;
    }
}
