use clap::{Args, Subcommand};
use colored::Colorize;
use kube::{Error, core::ErrorResponse};
use std::{process::Command, str};

use crate::errors::CliError;
use crate::essential::{BASE_COMMAND, connect_to_client};
use crate::logs::{check_namespace_exists, get_available_namespaces};

// docs:
//
// Pure formatting helper extracted from list_services so the table-row
// formatting logic can be unit tested with stubbed kubectl output.

fn format_service_rows(stdout: &str) -> Vec<String> {
    stdout
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                let name = parts[0];
                let ready = parts[1];
                let status = parts[2];
                let restarts = parts[3];
                let age = parts[4];

                let full_status = if ready.contains('/') {
                    format!("{} ({})", status, ready)
                } else {
                    status.to_string()
                };

                Some(format!(
                    "{:<40} {:<20} {:<10} {:<10}",
                    name, full_status, restarts, age
                ))
            } else {
                None
            }
        })
        .collect()
}

//service subcommands
#[derive(Subcommand, Debug, Clone)]
pub enum ServiceCommands {
    #[command(name = "list", about = "Check services list")]
    List {
        #[arg(long)]
        namespace: Option<String>,
    },
    #[command(name = "describe", about = "Describe service")]
    Describe {
        service_name: String,
        #[arg(long)]
        namespace: Option<String>,
    },
}
#[derive(Args, Debug, Clone)]
pub struct ServiceArgs {
    #[command(subcommand)]
    pub service_cmd: ServiceCommands,
}

// docs:
//
// This is the main function that lists all the services in the cluster
// Steps:
//      - connects to kubernetes client
//      - check if the namespace exists
//          - if the cortexflow namespace exists returns the service list
//          - else return an empty Vector
//
//
// Returns a CliError if the connection fails

pub async fn list_services(namespace: Option<String>) -> Result<(), CliError> {
    //TODO: maybe we can list both services and pods?

    match connect_to_client().await {
        Ok(_) => {
            let ns = namespace.unwrap_or_else(|| "cortexflow".to_string());

            println!(
                "{} {} {}",
                "=====>".blue().bold(),
                "Listing services in namespace:",
                ns
            );

            // Check if namespace exists first
            if !check_namespace_exists(&ns).await? {
                let available_namespaces = get_available_namespaces().await?;

                println!("\n❌ Namespace '{}' not found", ns);
                println!("{}", "=".repeat(50));

                if !available_namespaces.is_empty() {
                    println!("\n📋 Available namespaces:");
                    for available_ns in &available_namespaces {
                        println!("  • {}", available_ns);
                    }
                } else {
                    println!("No namespaces found in the cluster.");
                }
            }

            // kubectl command to get services
            let output = Command::new(BASE_COMMAND)
                .args(["get", "svc", "-n", &ns, "--no-headers"])
                .output();

            match output {
                Ok(output) => {
                    if !output.status.success() {
                        let error = str::from_utf8(&output.stderr).unwrap_or("Unknown error");
                        return Err(CliError::BaseError {
                            reason: format!("Error executing {}: {}", BASE_COMMAND, error),
                        });
                    }

                    let stdout = str::from_utf8(&output.stdout).unwrap_or("");

                    if stdout.trim().is_empty() {
                        println!(
                            "{} {} {}",
                            "=====>".blue().bold(),
                            "No services found in namespace",
                            ns
                        );
                    }

                    // header for Table
                    println!(
                        "{:<40} {:<20} {:<10} {:<10}",
                        "NAME", "STATUS", "RESTARTS", "AGE"
                    );
                    println!("{}", "-".repeat(80));

                    // Display Each Pod.
                    for row in format_service_rows(stdout) {
                        println!("{}", row);
                    }
                    Ok(())
                }
                Err(err) => {
                    return {
                        Err(CliError::ClientError(Error::Api(ErrorResponse {
                            status: "failed".to_string(),
                            message: "Failed to execute the kubectl command".to_string(),
                            reason: err.to_string(),
                            code: 404,
                        })))
                    };
                }
            }
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

// docs:
//
// This is the main function to describe a kubernetes service
// Steps:
//      - connects to kubernetes client
//      - check if the namespace exists
//          - if the cortexflow namespace exists executes the kubectl describe command
//              - output the result of the command
//          - else return an empty Vector
//
//
// Returns a CliError if the connection failsss

pub async fn describe_service(
    service_name: String,
    namespace: &Option<String>,
) -> Result<(), CliError> {
    match connect_to_client().await {
        Ok(_) => {
            match list_services(namespace.clone()).await {
                Ok(_) => {
                    let ns = namespace
                        .clone()
                        .unwrap_or_else(|| "cortexflow".to_string());

                    println!(
                        "{} {} {} {} {}",
                        "=====>".blue().bold(),
                        "Describing service",
                        "in namespace:",
                        service_name,
                        ns
                    );
                    println!("{}", "=".repeat(60));

                    // Check if namespace exists first
                    if !check_namespace_exists(&ns).await? {
                        let available_namespaces = get_available_namespaces().await?;

                        println!("\n❌ Namespace '{}' not found", ns);
                        println!("{}", "=".repeat(50));

                        if !available_namespaces.is_empty() {
                            println!("\n📋 Available namespaces:");
                            for available_ns in &available_namespaces {
                                println!("  • {}", available_ns);
                            }
                            println!(
                                "\nTry: cortexflow service describe {} --namespace <namespace-name>",
                                service_name
                            );
                        } else {
                            println!("No namespaces found in the cluster.");
                        }
                    }

                    // Execute kubectl describe pod command
                    let output = Command::new(BASE_COMMAND)
                        .args(["describe", "pod", &service_name, "-n", &ns])
                        .output();

                    match output {
                        Ok(output) => {
                            if !output.status.success() {
                                let error =
                                    str::from_utf8(&output.stderr).unwrap_or("Unknown error");
                                return Err(CliError::BaseError {
                                    reason: format!(
                                        "Error executing kubectl describe: {}.Make sure the pod '{}' exists in namespace '{}'",
                                        error, service_name, ns
                                    ),
                                });
                            }

                            let stdout = str::from_utf8(&output.stdout).unwrap_or("");

                            if stdout.trim().is_empty() {
                                println!("No description found for pod '{}'", service_name);
                            }

                            // Print the full kubectl describe output
                            println!("{}", stdout);
                            Ok(())
                        }
                        Err(err) => {
                            return {
                                Err(CliError::ClientError(Error::Api(ErrorResponse {
                                    status: "failed".to_string(),
                                    message: "Failed to execute the kubectl command ".to_string(),
                                    reason: err.to_string(),
                                    code: 404,
                                })))
                            };
                        }
                    }
                }
                Err(e) => {
                    return Err(CliError::BaseError {
                        reason: format!("Cannot list services: {}", e),
                    });
                }
            }
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_service_rows_formats_matching_lines() {
        let stdout = "my-svc        1/1     Running   0   5d\nother-svc     2/2     Pending   1   1h\n";
        let rows = format_service_rows(stdout);
        assert_eq!(rows.len(), 2);
        assert!(rows[0].contains("my-svc"));
        assert!(rows[0].contains("Running (1/1)"));
        assert!(rows[1].contains("other-svc"));
        assert!(rows[1].contains("Pending (2/2)"));
    }

    #[test]
    fn test_format_service_rows_skips_short_lines() {
        let stdout = "incomplete line\n";
        assert!(format_service_rows(stdout).is_empty());
    }

    #[test]
    fn test_format_service_rows_without_ready_slash() {
        let stdout = "svc-a   Active   0   5d   extra\n";
        let rows = format_service_rows(stdout);
        assert_eq!(rows.len(), 1);
        assert!(rows[0].contains("svc-a"));
    }

    #[test]
    fn test_format_service_rows_empty_input() {
        assert!(format_service_rows("").is_empty());
    }
}