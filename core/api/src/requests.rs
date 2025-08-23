use anyhow::Error;
use std::result::Result::Ok;
use tonic::{Request, Response, Streaming, transport::Channel};
use tonic_reflection::pb::v1::{
    ServerReflectionRequest, ServerReflectionResponse,
    server_reflection_client::ServerReflectionClient, server_reflection_request::MessageRequest,
};

use crate::agent::agent_client::AgentClient;
use crate::agent::ActiveConnectionResponse;
use crate::agent::RequestActiveConnections;

pub async fn send_active_connection_request(
    mut client: AgentClient<Channel>,
) -> Result<Response<ActiveConnectionResponse>, Error> {
    let request = Request::new(RequestActiveConnections { pod_ip: None });
    let response = client.active_connections(request).await?;
    Ok(response)
}

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
