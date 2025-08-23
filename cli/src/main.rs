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
use std::thread;
use std::time::Duration;
use tracing::debug;

use crate::essential::{
    get_config_directory,
    get_startup_config_dir,
    info,
    read_configs,
    update_cli,
};
use crate::install::{install_cortexflow, install_simple_example};
use crate::logs::logs_command;
use crate::monitoring::{ list_features, monitor_identity_events };
use crate::service::{ describe_service, list_services };
use crate::status::status_command;
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
    #[command(name = "install")] Install(InstallArgs),
    #[command(name = "uninstall")]
    Uninstall,
    #[command(name = "update")]
    Update,
    #[command(name = "info")]
    Info,
    #[command(name = "service")] Service(ServiceArgs),
    #[command(name = "status")] Status(StatusArgs),
    #[command(name = "logs")] Logs(LogsArgs),
    #[command(name = "monitoring")] Monitor(MonitorArgs),
}
#[derive(Args, Debug, Clone)]
struct SetArgs {
    val: String,
}

#[derive(Args, Debug, Clone)]
struct ServiceArgs {
    #[command(subcommand)]
    service_cmd: ServiceCommands,
}

//install args
#[derive(Args, Debug, Clone)]
struct InstallArgs {
    #[command(subcommand)]
    install_cmd: InstallCommands,
}
//install subcommands
#[derive(Subcommand, Debug, Clone)]
enum InstallCommands {
    #[command(name = "cortexflow")]
    All,
    #[command(name = "simple-example")]
    TestPods,
}

//monitoring subcommands
#[derive(Subcommand, Debug, Clone)]
enum MonitorCommands {
    #[command(name = "list")]
    List,
    #[command(name = "connections")]
    Connections,
}

//service subcommands
#[derive(Subcommand, Debug, Clone)]
enum ServiceCommands {
    #[command(name = "list")] List {
        #[arg(long)]
        namespace: Option<String>,
    },
    #[command(name = "describe")] Describe {
        service_name: String,
        #[arg(long)]
        namespace: Option<String>,
    },
}

// cfcli monitor <args>
#[derive(Args, Debug, Clone)]
struct MonitorArgs {
    #[command(subcommand)]
    monitor_cmd: MonitorCommands,
}

#[derive(Args, Debug, Clone)]
struct StatusArgs {
    #[arg(long)]
    output: Option<String>,
    #[arg(long)]
    namespace: Option<String>,
}

#[derive(Args, Debug, Clone)]
struct LogsArgs {
    #[arg(long)]
    service: Option<String>,
    #[arg(long)]
    component: Option<String>,
    #[arg(long)]
    namespace: Option<String>,
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
        thread::sleep(Duration::from_secs(1));
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
            Some(Commands::Monitor(monitor_args)) =>
                match monitor_args.monitor_cmd {
                    MonitorCommands::List => {
                        let _ = list_features().await;
                        Ok(())
                    }
                    MonitorCommands::Connections => {
                        let _ = monitor_identity_events().await;
                        Ok(())
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
