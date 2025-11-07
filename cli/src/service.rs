use std::{ str, process::Command };
use colored::Colorize;
use clap::{ Args, Subcommand };
use kube::{ core::ErrorResponse, Error };

use crate::essential::{ BASE_COMMAND, connect_to_client, CliError };
use crate::logs::{ get_available_namespaces, check_namespace_exists };

//service subcommands
#[derive(Subcommand, Debug, Clone)]
pub enum ServiceCommands {
    #[command(name = "list", about = "Check services list")] List {
        #[arg(long)]
        namespace: Option<String>,
    },
    #[command(name = "describe", about = "Describe service")] Describe {
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

            println!("{} {} {}", "=====>".blue().bold(), "Listing services in namespace:", ns);

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
                        eprintln!("Error executing {}: {}", BASE_COMMAND, error);
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
                    println!("{:<40} {:<20} {:<10} {:<10}", "NAME", "STATUS", "RESTARTS", "AGE");
                    println!("{}", "-".repeat(80));

                    // Display Each Pod.
                    for line in stdout.lines() {
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

                            println!(
                                "{:<40} {:<20} {:<10} {:<10}",
                                name,
                                full_status,
                                restarts,
                                age
                            );
                        }
                    }
                    Ok(())
                }
                Err(err) => {
                    Err(
                        CliError::ClientError(
                            Error::Api(ErrorResponse {
                                status: "failed".to_string(),
                                message: "Failed to execute the kubectl command".to_string(),
                                reason: "Your cluster is probably disconnected".to_string(),
                                code: 404,
                            })
                        )
                    )
                }
            }
        }
        Err(_) => {
            Err(
                CliError::ClientError(
                    Error::Api(ErrorResponse {
                        status: "failed".to_string(),
                        message: "Failed to connect to kubernetes client".to_string(),
                        reason: "Your cluster is probably disconnected".to_string(),
                        code: 404,
                    })
                )
            )
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
// Returns a CliError if the connection fails

pub async fn describe_service(
    service_name: String,
    namespace: &Option<String>
) -> Result<(), CliError> {
    match connect_to_client().await {
        Ok(_) => {
            match list_services(namespace.clone()).await {
                Ok(_) => {
                    //let file_path = get_config_directory().unwrap().1;

                    let ns = namespace.clone().unwrap_or_else(|| "cortexflow".to_string());

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
                            println!("\nTry: cortex service describe {} --namespace <namespace-name>", service_name);
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
                                let error = str
                                    ::from_utf8(&output.stderr)
                                    .unwrap_or("Unknown error");
                                eprintln!("Error executing kubectl describe: {}", error);
                                eprintln!(
                                    "Make sure the pod '{}' exists in namespace '{}'",
                                    service_name,
                                    ns
                                );
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
                            Err(
                                CliError::ClientError(
                                    Error::Api(ErrorResponse {
                                        status: "failed".to_string(),
                                        message: "Failed to execute the kubectl command ".to_string(),
                                        reason: "Your cluster is probably disconnected".to_string(),
                                        code: 404,
                                    })
                                )
                            )
                        }
                    }
                }
                Err(e) => todo!(),
            }
        }
        Err(_) => {
            Err(
                CliError::ClientError(
                    Error::Api(ErrorResponse {
                        status: "failed".to_string(),
                        message: "Failed to connect to kubernetes client".to_string(),
                        reason: "Your cluster is probably disconnected".to_string(),
                        code: 404,
                    })
                )
            )
        }
    }
}
