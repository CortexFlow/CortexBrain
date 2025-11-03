#![allow(warnings)]
//TODO: add an example with test pods during installation
mod essential;
mod install;
mod logs;
mod monitoring;
mod policies;
mod service;
mod status;
mod uninstall;

use clap::command;
use clap::{Args, Error, Parser, Subcommand};
use colored::Colorize;
use std::result::Result::Ok;
use std::string;
use tracing::debug;

use crate::essential::{info, read_configs, update_cli};
use crate::install::{InstallArgs, InstallCommands, install_cortexflow, install_simple_example};
use crate::logs::{LogsArgs, logs_command};
use crate::monitoring::{MonitorArgs, MonitorCommands, list_features, monitor_identity_events};
use crate::policies::{PoliciesArgs, PoliciesCommands, check_blocklist, create_blocklist, remove_ip};
use crate::service::{ServiceArgs, ServiceCommands, describe_service, list_services};
use crate::status::{StatusArgs, status_command};
use crate::uninstall::uninstall;

use crate::essential::GeneralData;
use crate::essential::update_config_metadata;

#[derive(Parser, Debug)]
#[command(
    author = GeneralData::AUTHOR,
    version = GeneralData::VERSION,
    about = None,
    long_about = None
)]
struct Cli {
    //name: String,
    #[clap(subcommand)]
    cmd: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    /* list of available commands */
    #[command(name = "set-env")] SetEnv(SetArgs),
    #[command(name = "get-env")]
    GetEnv,
    #[command(name = "install", about = "Manage installation")] Install(InstallArgs),
    #[command(name = "uninstall", about = "Manage uninstallation")]
    Uninstall,
    #[command(name = "update", about = "Check for updates")]
    Update,
    #[command(name = "info", about = "Check core info")]
    Info,
    #[command(name = "service", about = "Manage services")] Service(ServiceArgs),
    #[command(name = "status", about = "Check components status")] Status(StatusArgs),
    #[command(name = "logs", about = "Check services logs")] Logs(LogsArgs),
    #[command(name = "monitoring", about = "Monitoring commands")] Monitor(MonitorArgs),
    #[command(name = "policy", about = "Network Policies")] Policies(PoliciesArgs),
}
#[derive(Args, Debug, Clone)]
struct SetArgs {
    val: String,
}

async fn args_parser(){
    let args = Cli::parse();
    let env = "kubernetes".to_string();
    let general_data = GeneralData::new(env);
    debug!("Arguments {:?}", args.cmd);
    match args.cmd {
        Some(Commands::SetEnv(env)) => {
            general_data.set_env(env.val);
        }
        Some(Commands::GetEnv) => {
            general_data.get_env_output();
        }
        Some(Commands::Install(installation_args)) => match installation_args.install_cmd {
            InstallCommands::All => {
                install_cortexflow().await;
            }
            InstallCommands::TestPods => {
                install_simple_example();
            }
        },
        Some(Commands::Uninstall) => {
            uninstall();
        }
        Some(Commands::Update) => {
            update_cli();
        }
        Some(Commands::Info) => {
            info(general_data);
        }
        Some(Commands::Service(service_args)) => match service_args.service_cmd {
            ServiceCommands::List { namespace } => {
                Some(list_services(namespace));
            }
            ServiceCommands::Describe {
                service_name,
                namespace,
            } => {
                describe_service(service_name, &namespace);
            }
        },
        Some(Commands::Status(status_args)) => {
            status_command(status_args.output, status_args.namespace);
        }
        Some(Commands::Logs(logs_args)) => {
            logs_command(logs_args.service, logs_args.component, logs_args.namespace);
        }
        Some(Commands::Monitor(monitor_args)) => match monitor_args.monitor_cmd {
            MonitorCommands::List => {
                let _ = list_features().await;
            }
            MonitorCommands::Connections => {
                let _ = monitor_identity_events().await;
            }
        },
        Some(Commands::Policies(policies_args)) => {
            match policies_args.policy_cmd {
                PoliciesCommands::CheckBlocklist => {
                    let _ = check_blocklist().await;
                }
                PoliciesCommands::CreateBlocklist => {
                    // pass the ip as a monitoring flag
                    match policies_args.flags {
                        None => {
                            println!("{}", "Insert at least one ip to create a blocklist".red());
                        }
                        Some(ip) => {
                            println!("inserted ip: {} ", ip);
                            //insert the ip in the blocklist
                            match create_blocklist(&ip).await {
                                Ok(_) => {
                                    //update the config metadata
                                    let _ = update_config_metadata(&ip, "add").await;
                                }
                                Err(e) => {
                                    println!("{}", e);
                                }
                            }
                        }
                    }
                }
                PoliciesCommands::RemoveIpFromBlocklist => match policies_args.flags {
                    None => {
                        println!(
                            "{}",
                            "Insert at least one ip to remove from the blocklist".red()
                        );
                    }
                    Some(ip) => {
                        println!("Inserted ip: {}", ip);
                        match remove_ip(&ip).await {
                            Ok(_) => {
                                let _ = update_config_metadata(&ip, "delete").await;
                            }
                            Err(e) => {
                                println!("{}", e);
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
}

#[tokio::main]
async fn main() {
    let _ = args_parser().await;
}
