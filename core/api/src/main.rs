// module imports
use tonic::transport::{Error, Server};
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};

mod agent;
mod api;

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
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_level(true)
        .with_span_events(FmtSpan::NONE)
        .with_file(false)
        .pretty()
        .with_env_filter(EnvFilter::new("info"))
        .with_line_number(false)
        .init();

    info!("Starting agent server...");
    info!("fetching data");

    //FIXME: binding on 0.0.0.0 address is not ideal for a production environment. This will need future fixes
    let address = "0.0.0.0:9090".parse().unwrap();
    let api = AgentApi::default();

    match tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(agent_proto::AGENT_DESCRIPTOR)
        .build_v1()
    {
        Ok(reflection_server) => {
            info!("reflection server started correctly");
            match Server::builder()
                .add_service(AgentServer::new(api))
                .add_service(reflection_server)
                .serve(address)
                .await
            {
                Ok(_) => info!("Server started with no errors"),
                Err(e) => error!(
                    "An error occured during the Server::builder processe. Error {}",
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
