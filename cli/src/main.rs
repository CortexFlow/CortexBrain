mod essential;
mod install;
mod general;
mod uninstall;
mod service;
mod status;

use clap::{ Error, Parser, Subcommand, Args };
use clap::command;
use tracing::debug;

use crate::essential::{ info, update_cli };
use crate::install::install_cortexflow;
use crate::uninstall::uninstall;
use crate::service::{list_services, describe_service};
use crate::status::status_command;

use crate::general::GeneralData;

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
    env: String,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    /* list of available commands */
    #[command(name="set-env")]
    SetEnv(SetArgs),
    #[command(name="get-env")]
    GetEnv,
    #[command(name="install")]
    Install,
    #[command(name="uninstall")]
    Uninstall,
    #[command(name="update")]
    Update,
    #[command(name="info")]
    Info,
    #[command(name="service")]
    Service(ServiceArgs),
    #[command(name="status")]
    Status(StatusArgs),
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

#[derive(Subcommand, Debug, Clone)]
enum ServiceCommands {
    #[command(name="list")]
    List {
        #[arg(long)]
        namespace: Option<String>,
    },
    #[command(name="describe")]
    Describe {
        service_name: String,
        #[arg(long)]
        namespace: Option<String>,
    },
}

#[derive(Args, Debug, Clone)]
struct StatusArgs {
    #[arg(long, value_enum)]
    output: Option<String>,
}

fn args_parser() -> Result<(), Error> {
    let args = Cli::parse();
    let env = args.env;
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
        Some(Commands::Install) => {
            install_cortexflow();
            Ok(())
        }
        Some(Commands::Uninstall)=>{
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
        Some(Commands::Service(service_args)) => {
            match service_args.service_cmd {
                ServiceCommands::List { namespace } => {
                    list_services(namespace);
                    Ok(())
                }
                ServiceCommands::Describe { service_name, namespace } => {
                    describe_service(service_name, namespace);
                    Ok(())
                }
            }
        }
        Some(Commands::Status(status_args)) => {
            status_command(status_args.output);
            Ok(())
        }
        None => {
            eprintln!("CLI unknown argument. Cli arguments passed: {:?}", args.cmd);
            Ok(())
        }
    }
}

fn main() {
    let _ = args_parser();
}
