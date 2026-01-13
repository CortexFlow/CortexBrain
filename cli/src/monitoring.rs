//monitoring CLI function for identity service
use colored::Colorize;
use k8s_openapi::chrono::DateTime;
use kube::core::ErrorResponse;
use prost::Message;
use prost_types::FileDescriptorProto;
use std::result::Result::Ok;
use tonic_reflection::pb::v1::server_reflection_response::MessageResponse;

use agent_api::client::{connect_to_client, connect_to_server_reflection};
use agent_api::requests::{get_all_features, send_active_connection_request};

use crate::errors::CliError;
use clap::{Args, Subcommand};

//monitoring subcommands
#[derive(Subcommand, Debug, Clone)]
pub enum MonitorCommands {
    #[command(name = "list", about = "List all the agent API available functions")]
    List,
    #[command(
        name = "connections",
        about = "Monitor the recent connections detected by the identity service"
    )]
    Connections,
    #[command(
        name = "latencymetrics",
        about = "Monitor the latency metrics detected by the metrics service"
    )]
    Latencymetrics,
    #[command(
        name = "droppedpackets",
        about = "Monitor the dropped packets metrics detected by the metrics service"
    )]
    Droppedpackets,
}

// cfcli monitor <args>
#[derive(Args, Debug, Clone)]
pub struct MonitorArgs {
    #[command(subcommand)]
    pub monitor_cmd: MonitorCommands,
    //#[arg(long, short)]
    //pub flags: Option<String>,
}

pub async fn list_features() -> Result<(), CliError> {
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
                        if let Some(MessageResponse::FileDescriptorResponse(fdr)) =
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
                    return Err(CliError::AgentError(
                        tonic_reflection::server::Error::InvalidFileDescriptorSet(e.to_string()),
                    ));
                }
            }
        }
        Err(e) => {
            return Err(CliError::ClientError(kube::Error::Api(ErrorResponse {
                status: "failed".to_string(),
                message: "Failed to connect to kubernetes client".to_string(),
                reason: e.to_string(),
                code: 404,
            })));
        }
    }
    Ok(())
}

pub async fn monitor_identity_events() -> Result<(), CliError> {
    println!(
        "{} {}",
        "=====>".blue().bold(),
        "Connecting to cortexflow Client".white()
    );

    match connect_to_client().await {
        Ok(client) => {
            println!(
                "{} {}",
                "=====>".blue().bold(),
                "Connected to CortexFlow Client".green()
            );
            match send_active_connection_request(client).await {
                Ok(response) => {
                    let resp = response.into_inner();
                    if resp.events.is_empty() {
                        println!("{} No events found", "=====>".blue().bold());
                    } else {
                        println!(
                            "{} Found {} events",
                            "=====>".blue().bold(),
                            resp.events.len()
                        );
                        for (i, ev) in resp.events.iter().enumerate() {
                            println!(
                                "{} Event[{}] id: {}  src: {}  dst: {}",
                                "=====>".blue().bold(),
                                i,
                                ev.event_id,
                                ev.src_ip_port,
                                ev.dst_ip_port
                            );
                        }
                    }
                }
                Err(e) => {
                    return Err(CliError::AgentError(
                        tonic_reflection::server::Error::InvalidFileDescriptorSet(e.to_string()),
                    ));
                }
            }
        }
        Err(e) => {
            return Err(CliError::ClientError(kube::Error::Api(ErrorResponse {
                status: "failed".to_string(),
                message: "Failed to connect to kubernetes client".to_string(),
                reason: e.to_string(),
                code: 404,
            })));
        }
    }

    Ok(())
}

pub async fn monitor_latency_metrics() -> Result<(), CliError> {
    //function to monitor latency metrics
    println!(
        "{} {}",
        "=====>".blue().bold(),
        "Connecting to cortexflow Client".white()
    );

    match connect_to_client().await {
        Ok(client) => {
            println!(
                "{} {}",
                "=====>".blue().bold(),
                "Connected to CortexFlow Client".green()
            );
            //send request to get latency metrics
            match agent_api::requests::send_latency_metrics_request(client).await {
                Ok(response) => {
                    let resp = response.into_inner();
                    if resp.metrics.is_empty() {
                        println!("{} No latency metrics found", "=====>".blue().bold());
                    } else {
                        println!(
                            "{} Found {} latency metrics",
                            "=====>".blue().bold(),
                            resp.metrics.len()
                        );

                        for (i, metric) in resp.metrics.iter().enumerate() {
                            let converted_timestamp =
                                convert_timestamp_to_date(metric.timestamp_us);
                            println!(
                                "{} Latency[{}] \n tgid: {} \n process_name: {} \n address_family: {} \n delta(us): {} \n src_address_v4: {} \n dst_address_v4: {} \n src_address_v6: {} \n dst_address_v6: {} \n local_port: {} \n remote_port: {} \n timestamp_us: {}\n",
                                "=====>".blue().bold(),
                                i,
                                metric.tgid,
                                metric.process_name,
                                metric.address_family,
                                metric.delta_us,
                                metric.src_address_v4,
                                metric.dst_address_v4,
                                format!("{:?}", metric.src_address_v6),
                                format!("{:?}", metric.dst_address_v6),
                                metric.local_port,
                                metric.remote_port,
                                converted_timestamp
                            );
                        }
                    }
                }
                Err(e) => {
                    return Err(CliError::AgentError(
                        tonic_reflection::server::Error::InvalidFileDescriptorSet(e.to_string()),
                    ));
                }
            }
        }
        Err(e) => {
            return Err(CliError::ClientError(kube::Error::Api(ErrorResponse {
                status: "failed".to_string(),
                message: "Failed to connect to kubernetes client".to_string(),
                reason: e.to_string(),
                code: 404,
            })));
        }
    }
    Ok(())
}

pub async fn monitor_dropped_packets() -> Result<(), CliError> {
    //function to monitor dropped packets metrics
    println!(
        "{} {}",
        "=====>".blue().bold(),
        "Connecting to cortexflow Client".white()
    );

    match connect_to_client().await {
        Ok(client) => {
            println!(
                "{} {}",
                "=====>".blue().bold(),
                "Connected to CortexFlow Client".green()
            );
            //send request to get dropped packets metrics
            match agent_api::requests::send_dropped_packets_request(client).await {
                Ok(response) => {
                    let resp = response.into_inner();
                    if resp.metrics.is_empty() {
                        println!(
                            "{} No dropped packets metrics found",
                            "=====>".blue().bold()
                        );
                    } else {
                        println!(
                            "{} Found {} dropped packets metrics",
                            "=====>".blue().bold(),
                            resp.metrics.len()
                        );
                        for (i, metric) in resp.metrics.iter().enumerate() {
                            let converted_timestamp =
                                convert_timestamp_to_date(metric.timestamp_us);
                            println!(
                                "{} DroppedPackets[{}]\n  TGID: {}\n  Process: {}\n  SK Drops: {}\n  Socket Errors: {}\n  Soft Errors: {}\n  Backlog Length: {}\n  Write Memory Queued: {}\n  Receive Buffer Size: {}\n  ACK Backlog: {}\n  Timestamp: {} µs",
                                "=====>".blue().bold(),
                                i,
                                metric.tgid,
                                metric.process_name,
                                metric.sk_drops,
                                metric.sk_err,
                                metric.sk_err_soft,
                                metric.sk_backlog_len,
                                metric.sk_wmem_queued,
                                metric.sk_rcvbuf,
                                metric.sk_ack_backlog,
                                converted_timestamp
                            );
                        }
                    }
                }
                Err(e) => {
                    return Err(CliError::AgentError(
                        tonic_reflection::server::Error::InvalidFileDescriptorSet(e.to_string()),
                    ));
                }
            }
        }
        Err(e) => {
            return Err(CliError::ClientError(kube::Error::Api(ErrorResponse {
                status: "failed".to_string(),
                message: "Failed to connect to kubernetes client".to_string(),
                reason: e.to_string(),
                code: 404,
            })));
        }
    }
    Ok(())
}

fn convert_timestamp_to_date(timestamp: u64) -> String {
    let datetime = DateTime::from_timestamp_micros(timestamp as i64).unwrap();
    datetime.to_string()
}
