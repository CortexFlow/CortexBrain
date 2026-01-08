// module imports
use cortexbrain_common::logger;
use tonic::transport::{Error, Server};

mod agent;
mod api;
mod constants;
mod helpers;
mod structs;

mod agent_proto {
    use tonic::include_file_descriptor_set;

    pub(crate) const AGENT_DESCRIPTOR: &[u8] = include_file_descriptor_set!("agent_api_descriptor");
}

use crate::agent::agent_server::AgentServer;
use crate::api::AgentApi; //api implementations //from tonic. generated from agent.proto

use tokio::main;
use tracing::{error, info};

#[main]
async fn main() -> Result<(), Error> {
    //init tracing subscriber
    logger::init_default_logger();

    info!("Starting agent server...");
    info!("Fetching data");

    //FIXME: binding on 0.0.0.0 address is not ideal for a production environment. This will need future fixes
    let address = "0.0.0.0:9090".parse().unwrap();
    let api = AgentApi::default();

    match tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(agent_proto::AGENT_DESCRIPTOR)
        .build_v1()
    {
        Ok(reflection_server) => {
            info!("Reflection server started correctly");
            match Server::builder()
                .add_service(AgentServer::new(api))
                .add_service(reflection_server)
                .serve(address)
                .await
            {
                Ok(_) => info!("Server started with no errors"),
                Err(e) => error!(
                    "An error occured during the Server::builder process. Error {}",
                    e
                ),
            }
        }
        Err(e) => error!(
            "An error occured during the starting of the reflection server. Error {}",
            e
        ),
    }
    Ok(())
}
