#![allow(warnings)]
use std::result::Result::Ok;
use colored::Colorize;
use agent_api::requests::send_check_blocklist_request;
use agent_api::requests::send_create_blocklist_request;
use agent_api::requests::remove_ip_from_blocklist_request;
use anyhow::Error;
use clap::{ Args, Parser, Subcommand };
use agent_api::client::{ connect_to_client, connect_to_server_reflection };

//policies subcommands
#[derive(Subcommand, Debug, Clone)]
pub enum PoliciesCommands {
    #[command(name = "create-blocklist", about = "Create a blocklist to filter ips")]
    CreateBlocklist,
    #[command(name = "check-blocklist", about = "Check current ip blocklist")]
    CheckBlocklist,
    #[command(name="remove-ip",about ="Remove an ip from the blocklist")]
    RemoveIpFromBlocklist
}

// cfcli policies <args>
#[derive(Args, Debug, Clone)]
pub struct PoliciesArgs {
    #[command(subcommand)]
    pub policy_cmd: PoliciesCommands,
    #[arg(long, short)]
    pub flags: Option<String>,
}

pub async fn create_blocklist(ip: &str) -> Result<(), Error> {
    println!("{} {}", "=====>".blue().bold(), "Connecting to cortexflow Client".white());

    match connect_to_client().await {
        Ok(client) => {
            println!("{} {}", "=====>".blue().bold(), "Connected to CortexFlow Client".green());
            match send_create_blocklist_request(client, ip).await {
                Ok(response) => {
                    println!("{:?}", response.into_inner().events);
                }
                Err(e) => {
                    println!(
                        "{} {} {} {}",
                        "=====>".blue().bold(),
                        "An error occured".red(),
                        "Error:",
                        e
                    );
                }
            }
        }
        Err(_) => {
            println!(
                "{} {}",
                "=====>".blue().bold(),
                "Failed to connect to CortexFlow Client".red()
            );
        }
    }
    Ok(())
}

pub async fn check_blocklist() -> Result<(), Error> {
    println!("{} {}", "=====>".blue().bold(), "Connecting to cortexflow Client".white());

    match connect_to_client().await {
        Ok(client) => {
            println!("{} {}", "=====>".blue().bold(), "Connected to CortexFlow Client".green());
            match send_check_blocklist_request(client).await {
                Ok(response) => {
                    println!("{:?}", response.into_inner().events);
                }
                Err(e) => {
                    println!(
                        "{} {} {} {}",
                        "=====>".blue().bold(),
                        "An error occured".red(),
                        "Error:",
                        e
                    );
                }
            }
        }
        Err(_) => {
            println!(
                "{} {}",
                "=====>".blue().bold(),
                "Failed to connect to CortexFlow Client".red()
            );
        }
    }
    Ok(())
}
pub async fn remove_ip(ip:&str) -> Result<(), Error> {
    println!("{} {}", "=====>".blue().bold(), "Connecting to cortexflow Client".white());
    match connect_to_client().await {
        Ok(client) => {
            println!("{} {}", "=====>".blue().bold(), "Connected to CortexFlow Client".green());
            match remove_ip_from_blocklist_request(client,ip).await {
                Ok(response) => {
                    println!("{:?}", response.into_inner().events);
                }
                Err(e) => {
                    println!(
                        "{} {} {} {}",
                        "=====>".blue().bold(),
                        "An error occured".red(),
                        "Error:",
                        e
                    );
                }
            }
        }
        Err(_) => {
            println!(
                "{} {}",
                "=====>".blue().bold(),
                "Failed to connect to CortexFlow Client".red()
            );
        }
    }
    Ok(())
}
