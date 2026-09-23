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
                    return Err(e)
                }
            }
        }
        Err(e) => {
            println!(
                "{} {}",
                "=====>".blue().bold(),
                "Failed to connect to CortexFlow Client".red()
            );
            return Err(e)
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
                    return Err(e);
                }
            }
        }
        Err(e) => {
            println!(
                "{} {}",
                "=====>".blue().bold(),
                "Failed to connect to CortexFlow Client".red()
            );
            return Err(e);
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
                    return Err(e);
                }
            }
        }
        Err(e) => {
            println!(
                "{} {}",
                "=====>".blue().bold(),
                "Failed to connect to CortexFlow Client".red()
            );
            return Err(e);
        }
    }
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser)]
    struct TestCli {
        #[command(flatten)]
        args: PoliciesArgs,
    }

    // docs: the gRPC-calling functions in this module have no pure logic to
    // unit test without mocking the agent_api client; these tests instead
    // verify the clap argument parsing, which is the only externally-free logic here.

    #[test]
    fn test_parse_create_blocklist_with_ip() {
        let cli = TestCli::try_parse_from(["cfcli", "--flags", "1.2.3.4", "create-blocklist"])
            .unwrap();
        assert!(matches!(
            cli.args.policy_cmd,
            PoliciesCommands::CreateBlocklist
        ));
        assert_eq!(cli.args.flags, Some("1.2.3.4".to_string()));
    }

    #[test]
    fn test_parse_check_blocklist() {
        let cli = TestCli::try_parse_from(["cfcli", "check-blocklist"]).unwrap();
        assert!(matches!(
            cli.args.policy_cmd,
            PoliciesCommands::CheckBlocklist
        ));
        assert_eq!(cli.args.flags, None);
    }

    #[test]
    fn test_parse_remove_ip() {
        let cli =
            TestCli::try_parse_from(["cfcli", "--flags", "5.6.7.8", "remove-ip"]).unwrap();
        assert!(matches!(
            cli.args.policy_cmd,
            PoliciesCommands::RemoveIpFromBlocklist
        ));
        assert_eq!(cli.args.flags, Some("5.6.7.8".to_string()));
    }

    #[test]
    fn test_parse_unknown_subcommand_fails() {
        assert!(TestCli::try_parse_from(["cfcli", "not-a-command"]).is_err());
    }
}