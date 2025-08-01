// module imports
use tonic::transport::{Error, Server};

mod agent;
mod api;

mod agent_proto {
    use tonic::include_file_descriptor_set;

    pub(crate) const AGENT_DESCRIPTOR: &[u8] = include_file_descriptor_set!("agent_api_descriptor");
}

use crate::agent::agent_server::AgentServer;
use crate::api::AgentApi; //api implementations //from tonic. generated from agent.proto

use tokio::main;

#[main]
async fn main() -> Result<(), Error> {
    let address = "127.0.0.1:9090".parse().unwrap();
    let api = AgentApi::default();

    match tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(agent_proto::AGENT_DESCRIPTOR)
        .build_v1()
    {
        Ok(reflection_server) => {
            print!("reflection server started correctly");
            match Server::builder()
                .add_service(AgentServer::new(api))
                .add_service(reflection_server)
                .serve(address)
                .await
            {
                Ok(_) => println!("Server started with no errors"),
                Err(e) => eprintln!(
                    "An error occured during the Server::builder processe. Error {}",
                    e
                ),
            }
        }
        Err(e) => eprintln!(
            "An error occured during the starting of the reflection server. Error {}",
            e
        ),
    }
    Ok(())
}
