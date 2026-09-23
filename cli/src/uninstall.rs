use colored::Colorize;
use std::io::stdin;

use crate::command_runner::{CommandRunner, RealCommandRunner};
use crate::errors::CliError;
use crate::essential::{BASE_COMMAND, connect_to_client};
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
        Err(e) => {
            return {
                Err(CliError::ClientError(Error::Api(ErrorResponse {
                    status: "failed".to_string(),
                    message: "Failed to connect to kubernetes client".to_string(),
                    reason: e.to_string(),
                    code: 404,
                })))
            };
        }
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
            uninstall_all_with(&RealCommandRunner)
        }
        Err(e) => {
            return {
                Err(CliError::ClientError(Error::Api(ErrorResponse {
                    status: "failed".to_string(),
                    message: "Failed to connect to kubernetes client".to_string(),
                    reason: e.to_string(),
                    code: 404,
                })))
            };
        }
    }
}

fn uninstall_all_with(runner: &dyn CommandRunner) -> Result<(), CliError> {
    let args = [
        "delete".to_string(),
        "namespace".to_string(),
        "cortexflow".to_string(),
    ];
    let output = runner
        .run(BASE_COMMAND, &args)
        .map_err(|e| CliError::InstallerError {
            reason: format!("Failed to execute delete command: {}", e),
        })?;

    if output.status.success() {
        println!("✅ Removed cortexflow namespace");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(CliError::InstallerError {
            reason: format!("Failed to delete cortexflow namespace. Error: {}", stderr),
        })
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

            uninstall_component_with(&RealCommandRunner, component_type, component)
        }
        Err(e) => {
            return {
                Err(CliError::ClientError(Error::Api(ErrorResponse {
                    status: "failed".to_string(),
                    message: "Failed to connect to kubernetes client".to_string(),
                    reason: e.to_string(),
                    code: 404,
                })))
            };
        }
    }
}

fn uninstall_component_with(
    runner: &dyn CommandRunner,
    component_type: &str,
    component: &str,
) -> Result<(), CliError> {
    let args = [
        "delete".to_string(),
        component_type.to_string(),
        component.to_string(),
        "-n".to_string(),
        "cortexflow".to_string(),
    ];
    let output = runner
        .run(BASE_COMMAND, &args)
        .map_err(|e| CliError::InstallerError {
            reason: format!("Failed to execute delete command: {}", e),
        })?;

    if output.status.success() {
        println!("✅ Removed component {}", component);
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(CliError::InstallerError {
            reason: format!("Failed to delete component '{}': {}", component, stderr),
        })
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::command_runner::test_support::StubCommandRunner;

    #[test]
    fn test_uninstall_all_with_success() {
        let runner = StubCommandRunner::success("");
        assert!(uninstall_all_with(&runner).is_ok());
    }

    #[test]
    fn test_uninstall_all_with_command_failure() {
        let runner = StubCommandRunner::failure("namespace not found");
        assert!(uninstall_all_with(&runner).is_err());
    }

    #[test]
    fn test_uninstall_all_with_io_error() {
        let runner = StubCommandRunner::io_error();
        assert!(uninstall_all_with(&runner).is_err());
    }

    #[test]
    fn test_uninstall_component_with_success() {
        let runner = StubCommandRunner::success("");
        assert!(uninstall_component_with(&runner, "deployment", "cortexflow-identity").is_ok());
    }

    #[test]
    fn test_uninstall_component_with_command_failure() {
        let runner = StubCommandRunner::failure("component not found");
        assert!(uninstall_component_with(&runner, "deployment", "unknown").is_err());
    }

    #[test]
    fn test_uninstall_component_with_io_error() {
        let runner = StubCommandRunner::io_error();
        assert!(uninstall_component_with(&runner, "deployment", "cortexflow-identity").is_err());
    }
}