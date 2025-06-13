mod essential;
mod install;

use clap::{Arg, Parser, Subcommand};

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Install,
    Update,
    Version,
}

fn args_parser() {
    let args = Args::parse();
    
    match args.command {
        Commands::Version => {
            essential::version();
        },
        Commands::Install => {
            // TODO: Not Implemented Yet. Check install.rs
            install::install_cortexflow();
        },
        Commands::Update => {
            // TODO: Not Implemented Yet. Check essential.rs
            essential::update_cli();
        }
    }
}

fn main() {
    args_parser();
}
