mod essential;
mod install;

use clap::{ Error, Parser, Subcommand };
use clap::command;

use crate::essential::{ info, update_cli };
use crate::install::install_cortexflow;

#[derive(Parser, Debug)]
#[command(author = "CortexFlow", version = "0.1", about = None, long_about = None)]
struct Args {
    //name: String,
    #[clap(subcommand)]
    cmd: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    /* list of available commands */
    Install,
    Update,
    Info,
}

fn args_parser() -> Result<(), Error> {
    let args = Args::parse();
    println!("Arguments {:?}", args.cmd);
    match args.cmd {
        Some(Commands::Install) => {
            install_cortexflow();
            Ok(())
        }
        Some(Commands::Update) => {
            update_cli();
            Ok(())
        }
        Some(Commands::Info) => {
            info();
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
