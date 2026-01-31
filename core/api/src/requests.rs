use anyhow::Error;
use std::result::Result::Ok;
use tonic::{Request, Response, Streaming, transport::Channel};
use tonic_reflection::pb::v1::{
    ServerReflectionRequest, ServerReflectionResponse,
    server_reflection_client::ServerReflectionClient, server_reflection_request::MessageRequest,
};

use crate::agent::ActiveConnectionResponse;
use crate::agent::AddIpToBlocklistRequest;
use crate::agent::BlocklistResponse;
use crate::agent::DroppedPacketsResponse;
use crate::agent::LatencyMetricsResponse;
use crate::agent::RequestActiveConnections;
use crate::agent::RmIpFromBlocklistRequest;
use crate::agent::RmIpFromBlocklistResponse;
use crate::agent::VethResponse;
use crate::agent::agent_client::AgentClient;

#[cfg(feature = "client")]
pub async fn send_active_connection_request(
    mut client: AgentClient<Channel>,
) -> Result<Response<ActiveConnectionResponse>, Error> {
    let request = Request::new(RequestActiveConnections { pod_ip: None });
    let response = client.active_connections(request).await?;
    Ok(response)
}

#[cfg(feature = "client")]
pub async fn get_all_features(
    mut client: ServerReflectionClient<Channel>,
) -> Result<Response<Streaming<ServerReflectionResponse>>, Error> {
    let request = ServerReflectionRequest {
        host: "".to_string(),
        message_request: Some(MessageRequest::FileContainingSymbol(
            "agent.Agent".to_string(),
        )),
    };
    let response = client
        .server_reflection_info(tokio_stream::iter(vec![request]))
        .await?;

    Ok(response)
}

#[cfg(feature = "client")]
pub async fn send_create_blocklist_request(
    mut client: AgentClient<Channel>,
    ip: &str,
) -> Result<Response<BlocklistResponse>, Error> {
    let ip = Some(ip.to_string());
    let request = Request::new(AddIpToBlocklistRequest { ip });
    let response = client.add_ip_to_blocklist(request).await?;
    Ok(response)
}

#[cfg(feature = "client")]
pub async fn send_check_blocklist_request(
    mut client: AgentClient<Channel>,
) -> Result<Response<BlocklistResponse>, Error> {
    let request = Request::new(());
    let response = client.check_blocklist(request).await?;
    Ok(response)
}

#[cfg(feature = "client")]
pub async fn remove_ip_from_blocklist_request(
    mut client: AgentClient<Channel>,
    ip: &str,
) -> Result<Response<RmIpFromBlocklistResponse>, Error> {
    let ip = ip.to_string();
    let request = Request::new(RmIpFromBlocklistRequest { ip });
    let response = client.rm_ip_from_blocklist(request).await?;
    Ok(response)
}

#[cfg(feature = "client")]
pub async fn send_dropped_packets_request(
    mut client: AgentClient<Channel>,
) -> Result<Response<DroppedPacketsResponse>, Error> {
    let request = Request::new(());
    let response = client.get_dropped_packets_metrics(request).await?;
    Ok(response)
}

#[cfg(feature = "client")]
pub async fn send_latency_metrics_request(
    mut client: AgentClient<Channel>,
) -> Result<Response<LatencyMetricsResponse>, Error> {
    let request = Request::new(());
    let response = client.get_latency_metrics(request).await?;
    Ok(response)
}

#[cfg(feature = "client")]
pub async fn send_tracked_veth_request(
    mut client: AgentClient<Channel>,
) -> Result<Response<VethResponse>, Error> {
    let request = Request::new(());
    let response = client.get_tracked_veth(request).await?;
    Ok(response)
}
