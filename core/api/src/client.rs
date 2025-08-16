use anyhow::Error;
use std::result::Result::Ok;
use tonic::{transport::Channel};
use tonic_reflection::pb::v1::{
    server_reflection_client::ServerReflectionClient,
};
use crate::agent::agent_client::AgentClient;



pub async fn connect_to_client() -> Result<AgentClient<Channel>, Error> {
    //this methods force a HTTP/2 connection from a static string
    //FIXME: this will require an update to ensure a protected connection
    let channel = Channel::from_static("http://192.168.49.2:30092")
        .connect()
        .await?;
    let client = AgentClient::new(channel);
    Ok(client)
}

pub async fn connect_to_server_reflection() -> Result<ServerReflectionClient<Channel>, Error> {
    //this methods force a HTTP/2 connection from a static string
    let channel = Channel::from_static("http://192.168.49.2:30092")
        .connect()
        .await?;
    let client = ServerReflectionClient::new(channel);
    Ok(client)
}
