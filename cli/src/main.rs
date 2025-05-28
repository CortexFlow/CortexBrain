mod essential;
mod install;

use clap::{Arg, Parser, Subcommand};

#[derive(Parser)]
#[command(author="CortexFlow",version="0.1",about=None,long_about=None)]
struct Args {
    name: String,
}
enum Commands {
    /* continue from here */
    Install(String),
    Update(String),
}
fn args_parser() {
    let args = Args::parse();
    println!("Arguments {:?}", args.name);
}

fn main() {
    args_parser();
}
