//TODO: add an example with test pods during installation
mod essential;
mod install;
mod logs;
mod monitoring;
mod service;
mod status;
mod uninstall;

use clap::command;
use clap::{ Args, Error, Parser, Subcommand };
use colored::Colorize;
use std::result::Result::Ok;
use tracing::debug;

use crate::essential::{
    get_config_directory,
    get_startup_config_dir,
    info,
    read_configs,
    update_cli,
};
use crate::install::{ InstallArgs, InstallCommands, install_cortexflow, install_simple_example };
use crate::logs::{ LogsArgs, logs_command };
use crate::monitoring::{ list_features, monitor_identity_events, MonitorArgs, MonitorCommands };
use crate::service::{ ServiceCommands, ServiceArgs, describe_service, list_services };
use crate::status::{ StatusArgs, status_command };
use crate::uninstall::uninstall;

use crate::essential::GeneralData;

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
}
#[derive(Args, Debug, Clone)]
struct SetArgs {
    val: String,
}

async fn args_parser() -> Result<(), Error> {
    let args = Cli::parse();

    //get the environment from the config file metadata

    let config_dir = get_startup_config_dir();

    if !config_dir {
        eprintln!(
            "{} {}",
            "[SYSTEM]".blue().bold(),
            "Config files not found. Please proceed with the installation"
        );
        install_cortexflow();
        Ok(())
    } else {
        println!("{} {}", "[SYSTEM]".blue().bold(), "Founded config files".white());
        let config_file_path = get_config_directory();
        let file_path = config_file_path.unwrap().1;
        let env = read_configs(file_path.to_path_buf());
        let general_data = GeneralData::new(env);
        debug!("Arguments {:?}", args.cmd);
        match args.cmd {
            Some(Commands::SetEnv(env)) => {
                general_data.set_env(env.val);
                Ok(())
            }
            Some(Commands::GetEnv) => {
                general_data.get_env_output();
                Ok(())
            }
            Some(Commands::Install(installation_args)) =>
                match installation_args.install_cmd {
                    InstallCommands::All => {
                        install_cortexflow();
                        Ok(())
                    }
                    InstallCommands::TestPods => {
                        install_simple_example();
                        Ok(())
                    }
                }
            Some(Commands::Uninstall) => {
                uninstall();
                Ok(())
            }
            Some(Commands::Update) => {
                update_cli();
                Ok(())
            }
            Some(Commands::Info) => {
                info(general_data);
                Ok(())
            }
            Some(Commands::Service(service_args)) =>
                match service_args.service_cmd {
                    ServiceCommands::List { namespace } => {
                        Some(list_services(namespace));
                        Ok(())
                    }
                    ServiceCommands::Describe { service_name, namespace } => {
                        describe_service(service_name, &namespace);
                        Ok(())
                    }
                }
            Some(Commands::Status(status_args)) => {
                status_command(status_args.output, status_args.namespace);
                Ok(())
            }
            Some(Commands::Logs(logs_args)) => {
                logs_command(logs_args.service, logs_args.component, logs_args.namespace);
                Ok(())
            }
            Some(Commands::Monitor(monitor_args)) => {
                match monitor_args.monitor_cmd {
                    MonitorCommands::List => {
                        match monitor_args.flags {
                            None => {
                                let _ = list_features().await;
                                Ok(())
                            }
                            Some(exclude_flag) => {
                                println!("Inserted flag: {}", exclude_flag);
                                Ok(())
                            }
                        }
                    }
                    MonitorCommands::Connections => {
                        match monitor_args.flags {
                            None => {
                                let _ = monitor_identity_events().await;
                                Ok(())
                            }
                            Some(exclude_flag) => {
                                println!("Inserted flag: {}", exclude_flag);
                                Ok(())
                            }
                        }
                    }
                }
            }
            None => {
                eprintln!("CLI unknown argument. Cli arguments passed: {:?}", args.cmd);
                Ok(())
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let _ = args_parser().await;
}
