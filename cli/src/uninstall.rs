use colored::Colorize;
use std::{io::stdin, process::Command, thread, time::Duration};

use crate::essential::{BASE_COMMAND, CliError, connect_to_client};
use kube::{Error, core::ErrorResponse};

//docs:
//
// This function manages the uninstall process for the cortexflow components
// Steps:
//      - connects to kubernetes client
//      - display the uninstall options
//      - read the user input (e.g. 1 > all components)
//      - uninstall the selected component or the whole namespace
//
// Returns an CliError if something fails

pub async fn uninstall() -> Result<(), CliError> {
    match connect_to_client().await {
        Ok(_) => {
            println!(
                "{} {}",
                "=====>".blue().bold(),
                "Uninstalling cortexflow..."
            );
            let mut userinput: String = String::new();
            println!("{} {}", "=====>".blue().bold(), "Select one option:");
            display_uninstall_options();
            stdin()
                .read_line(&mut userinput)
                .expect("Error reading user input");

            let trimmed_input = userinput.trim();
            if trimmed_input == "1" {
                uninstall_all().await?;
            } else if trimmed_input == "2" {
                uninstall_component("deployment", "cortexflow-identity").await?;
            }
            Ok(())
        }
        Err(_) => Err(CliError::ClientError(Error::Api(ErrorResponse {
            status: "failed".to_string(),
            message: "Failed to connect to kubernetes client".to_string(),
            reason: "Your cluster is probably disconnected".to_string(),
            code: 404,
        }))),
    }
}

//docs:
//
// This function only print the uninstall options

fn display_uninstall_options() {
    println!("{} {}", "=====>".blue().bold(), "1 > all");
    println!("{} {}", "=====>".blue().bold(), "2 > identity-service");
}

//docs:
//
// This function manages the uninstall of the whole cortexflow namespace
// Steps:
//      - connects to kubernetes client
//      - execute the command to uninstall the cortexflow namespace
//
// Returns an CliError if something fails

async fn uninstall_all() -> Result<(), CliError> {
    match connect_to_client().await {
        Ok(_) => {
            println!(
                "{} {}",
                "=====>".blue().bold(),
                "Deleting cortexflow components".red().bold()
            );
            let output = Command::new(BASE_COMMAND)
                .args(["delete", "namespace", "cortexflow"])
                .output()
                .map_err(|e| CliError::InstallerError {
                    reason: format!("Failed to execute delete command: {}", e),
                })?;

            if output.status.success() {
                println!("✅ Removed cortexflow namespace");
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("Error deleting cortexflow namespace. Error: {} ", stderr);
                Err(CliError::InstallerError {
                    reason: format!("Failed to delete cortexflow namespace. Error: {}", stderr),
                })
            }
        }
        Err(_) => Err(CliError::ClientError(Error::Api(ErrorResponse {
            status: "failed".to_string(),
            message: "Failed to connect to kubernetes client".to_string(),
            reason: "Your cluster is probably disconnected".to_string(),
            code: 404,
        }))),
    }
}

//docs:
//
// This function manages the uninstall of given cortexflow components
// Steps:
//      - connects to kubernetes client
//      - executes the command to uninstall a given component
//
// Returns an InstallerError if something fails

async fn uninstall_component(component_type: &str, component: &str) -> Result<(), CliError> {
    match connect_to_client().await {
        Ok(_) => {
            println!(
                "{} {} {}",
                "=====>".blue().bold(),
                "Deleting service",
                component
            );

            let output = Command::new(BASE_COMMAND)
                .args(["delete", component_type, component, "-n", "cortexflow"])
                .output()
                .map_err(|e| CliError::InstallerError {
                    reason: format!("Failed to execute delete command: {}", e),
                })?;

            if output.status.success() {
                println!("✅ Removed component {}", component);
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("Error deleting {}:\n{}", component, stderr);
                Err(CliError::InstallerError {
                    reason: format!("Failed to delete component '{}': {}", component, stderr),
                })
            }
        }
        Err(_) => Err(CliError::ClientError(Error::Api(ErrorResponse {
            status: "failed".to_string(),
            message: "Failed to connect to kubernetes client".to_string(),
            reason: "Your cluster is probably disconnected".to_string(),
            code: 404,
        }))),
    }
}

//
//
//docs:
//
// This function is deprecated and will be removed in the next version
//
// Do not include or refactor this function
#[deprecated(since = "0.1.4")]
fn rm_dir(directory_to_remove: &str) {
    let output = Command::new("rm")
        .args(["-rf", directory_to_remove])
        .output()
        .expect("cannot remove directory");

    if !output.status.success() {
        eprintln!(
            "Error removing directory: {}:\n{}",
            directory_to_remove,
            String::from_utf8_lossy(&output.stderr)
        );
    } else {
        println!("✅ Removed directory {}", directory_to_remove);
    }

    thread::sleep(Duration::from_secs(2));
}
