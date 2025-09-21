#![allow(warnings)]

//monitoring CLI function for identity service
use anyhow::Error;
use colored::Colorize;
use prost::Message;
use prost_types::FileDescriptorProto;
use std::result::Result::Ok;
use tonic_reflection::pb::v1::{ server_reflection_response::MessageResponse };

use agent_api::client::{ connect_to_client, connect_to_server_reflection };
use agent_api::requests::{ get_all_features, send_active_connection_request };

use clap::command;
use clap::{ Args, Parser, Subcommand };

//monitoring subcommands
#[derive(Subcommand, Debug, Clone)]
pub enum MonitorCommands {
    #[command(name = "list", about = "List all the agent API available functions")]
    List,
    #[command(
        name = "connections",
        about = "Monitor the recent connections detected by the identity service"
    )] Connections,
}

// cfcli monitor <args>
#[derive(Args, Debug, Clone)]
pub struct MonitorArgs {
    #[command(subcommand)]
    pub monitor_cmd: MonitorCommands,
    #[arg(long, short)]
    pub flags: Option<String>,
}

pub async fn list_features() -> Result<(), Error> {
    match connect_to_server_reflection().await {
        Ok(client) => {
            println!(
                "{} {}",
                "=====>".blue().bold(),
                "Connected to CortexFlow Server Reflection".green()
            );
            match get_all_features(client).await {
                Ok(response) => {
                    let mut streaming = response.into_inner();

                    //decoding the proto file
                    while let Some(resp) = streaming.message().await? {
                        if
                            let Some(MessageResponse::FileDescriptorResponse(fdr)) =
                                resp.message_response
                        {
                            println!("Available services:");
                            for bytes in fdr.file_descriptor_proto {
                                //decode file descriptor
                                let fd = FileDescriptorProto::decode(bytes.as_slice())?;

                                for service in fd.service {
                                    for method in service.method {
                                        let method_name = method.name.unwrap_or_default();
                                        println!("{}", method_name);
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    println!(
                        "{} {} {} {}",
                        "=====>".blue().bold(),
                        "An error occured".red(),
                        "Error:",
                        e
                    );
                }
            }
        }
        Err(_) =>
            println!(
                "{} {}",
                "=====>".blue().bold(),
                "Failed to connect to CortexFlow Server Reflection".red()
            ),
    }
    Ok(())
}

pub async fn monitor_identity_events() -> Result<(), Error> {
    println!("{} {}", "=====>".blue().bold(), "Connecting to cortexflow Client".white());

    match connect_to_client().await {
        Ok(client) => {
            println!("{} {}", "=====>".blue().bold(), "Connected to CortexFlow Client".green());
            match send_active_connection_request(client).await {
                Ok(response) => {
                    println!("{:?}", response.into_inner().events);
                }
                Err(e) => {
                    println!(
                        "{} {} {} {}",
                        "=====>".blue().bold(),
                        "An error occured".red(),
                        "Error:",
                        e
                    );
                }
            }
        }
        Err(_) =>
            println!(
                "{} {}",
                "=====>".blue().bold(),
                "Failed to connect to CortexFlow Client".red()
            ),
    }

    Ok(())
}
