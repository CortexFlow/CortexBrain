mod errors;
mod essential;
mod install;
mod logs;
mod monitoring;
mod policies;
mod service;
mod status;
mod uninstall;

use clap::{Args, Parser, Subcommand};
use colored::Colorize;
use std::result::Result::Ok;
use tracing::debug;

use crate::errors::CliError;
use crate::essential::{info, update_cli};
use crate::install::{InstallArgs, InstallCommands, install_cortexflow, install_simple_example};
use crate::logs::{LogsArgs, logs_command};
use crate::monitoring::{
    MonitorArgs, MonitorCommands, list_features, monitor_dropped_packets, monitor_identity_events,
    monitor_latency_metrics,
};
use crate::policies::{
    PoliciesArgs, PoliciesCommands, check_blocklist, create_blocklist, remove_ip,
};
use crate::service::{ServiceArgs, ServiceCommands, describe_service, list_services};
use crate::status::{StatusArgs, status_command};
use crate::uninstall::uninstall;

use crate::essential::update_config_metadata;

#[derive(Parser, Debug)]
#[command(
    author = env!("CARGO_PKG_AUTHORS"),
    version = env!("CARGO_PKG_VERSION"),
    about = env!("CARGO_PKG_DESCRIPTION"),
    long_about = env!("CARGO_PKG_DESCRIPTION")
)]
struct Cli {
    //name: String,
    #[clap(subcommand)]
    cmd: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    /* list of available commands */
    #[command(name = "install", about = "Manage installation")]
    Install(InstallArgs),
    #[command(name = "uninstall", about = "Manage uninstallation")]
    Uninstall,
    #[command(name = "update", about = "Check for updates")]
    Update,
    #[command(name = "info", about = "Check core info")]
    Info,
    #[command(name = "service", about = "Manage services")]
    Service(ServiceArgs),
    #[command(name = "status", about = "Check components status")]
    Status(StatusArgs),
    #[command(name = "logs", about = "Check services logs")]
    Logs(LogsArgs),
    #[command(name = "monitoring", about = "Monitoring commands")]
    Monitor(MonitorArgs),
    #[command(name = "policy", about = "Network Policies")]
    Policies(PoliciesArgs),
}
#[derive(Args, Debug, Clone)]
struct SetArgs {
    val: String,
}

async fn args_parser() -> Result<(), CliError> {
    let args = Cli::parse();
    debug!("Arguments {:?}", args.cmd);
    match args.cmd {
        Some(Commands::Install(installation_args)) => match installation_args.install_cmd {
            InstallCommands::All => {
                install_cortexflow().await.map_err(|e| eprintln!("{}", e))?;
            }
            InstallCommands::TestPods => {
                install_simple_example()
                    .await
                    .map_err(|e| eprintln!("{}", e))?;
            }
        },
        Some(Commands::Uninstall) => {
            uninstall().await.map_err(|e| eprintln!("{}", e))?;
        }
        Some(Commands::Update) => {
            update_cli();
        }
        Some(Commands::Info) => {
            info();
        }
        Some(Commands::Service(service_args)) => match service_args.service_cmd {
            ServiceCommands::List { namespace } => {
                list_services(namespace)
                    .await
                    .map_err(|e| eprintln!("{}", e))?;
            }
            ServiceCommands::Describe {
                service_name,
                namespace,
            } => {
                describe_service(service_name, &namespace)
                    .await
                    .map_err(|e| eprintln!("{}", e))?;
            }
        },
        Some(Commands::Status(status_args)) => {
            status_command(status_args.output, status_args.namespace)
                .await
                .map_err(|e| eprintln!("{}", e))?;
        }
        Some(Commands::Logs(logs_args)) => {
            logs_command(logs_args.service, logs_args.component, logs_args.namespace)
                .await
                .map_err(|e| eprintln!("{}", e))?;
        }
        Some(Commands::Monitor(monitor_args)) => match monitor_args.monitor_cmd {
            MonitorCommands::List => {
                let _ = list_features().await.map_err(|e| eprintln!("{}", e))?;
            }
            MonitorCommands::Connections => {
                let _ = monitor_identity_events()
                    .await
                    .map_err(|e| eprintln!("{}", e))?;
            }
            MonitorCommands::Latencymetrics => {
                let _ = monitor_latency_metrics()
                    .await
                    .map_err(|e| eprintln!("{}", e))?;
            }
            MonitorCommands::Droppedpackets => {
                let _ = monitor_dropped_packets()
                    .await
                    .map_err(|e| eprintln!("{}", e))?;
            }
        },
        Some(Commands::Policies(policies_args)) => {
            match policies_args.policy_cmd {
                PoliciesCommands::CheckBlocklist => {
                    let _ = check_blocklist().await.map_err(|e| eprintln!("{}", e))?;
                }
                PoliciesCommands::CreateBlocklist => {
                    // pass the ip as a monitoring flag
                    match policies_args.flags {
                        None => {
                            eprintln!("{}", "Insert at least one ip to create a blocklist".red());
                        }
                        Some(ip) => {
                            println!("inserted ip: {} ", ip);
                            //insert the ip in the blocklist
                            match create_blocklist(&ip).await {
                                Ok(_) => {
                                    //update the config metadata
                                    let _ = update_config_metadata(&ip, "add")
                                        .await
                                        .map_err(|e| eprintln!("{}", e))?;
                                }
                                Err(e) => {
                                    eprintln!("{}", e);
                                }
                            }
                        }
                    }
                }
                PoliciesCommands::RemoveIpFromBlocklist => match policies_args.flags {
                    None => {
                        eprintln!(
                            "{}",
                            "Insert at least one ip to remove from the blocklist".red()
                        );
                    }
                    Some(ip) => {
                        println!("Inserted ip: {}", ip);
                        match remove_ip(&ip).await {
                            Ok(_) => {
                                let _ = update_config_metadata(&ip, "delete")
                                    .await
                                    .map_err(|e| eprintln!("{}", e))?;
                            }
                            Err(e) => {
                                eprintln!("{}", e);
                            }
                        }
                    }
                },
            }
        }
        None => {
            eprintln!("CLI unknown argument. Cli arguments passed: {:?}", args.cmd);
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    let _ = args_parser().await;
}
