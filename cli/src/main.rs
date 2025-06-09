mod essential;
mod install;
mod general;

use clap::{ Error, Parser, Subcommand, Args };
use clap::command;
use tracing::debug;

use crate::essential::{ info, update_cli };
use crate::install::install_cortexflow;

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
    #[command(name="update")]
    Update,
    #[command(name="info")]
    Info,
}
#[derive(Args, Debug, Clone)]
struct SetArgs {
    val: String,
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
        Some(Commands::Update) => {
            update_cli();
            Ok(())
        }
        Some(Commands::Info) => {
            info(general_data);
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
