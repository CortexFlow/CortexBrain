use crate::errors::CliError;
use crate::essential::{BASE_COMMAND, connect_to_client};
use clap::Args;
use colored::Colorize;
use kube::{Error, core::ErrorResponse};
use std::{process::Command, result::Result::Ok, str};

#[derive(Args, Debug, Clone)]
pub struct LogsArgs {
    #[arg(long)]
    pub service: Option<String>,
    #[arg(long)]
    pub component: Option<String>,
    #[arg(long)]
    pub namespace: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Component {
    ControlPlane,
    DataPlane,
}

impl From<String> for Component {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "control-plane" => Component::ControlPlane,
            "data-plane" => Component::DataPlane,
            //default will be control plane.
            _ => Component::ControlPlane,
        }
    }
}

impl Component {
    fn to_label_selector(&self) -> &str {
        match self {
            Component::ControlPlane => "component=control-plane",
            Component::DataPlane => "component=data-plane",
        }
    }
}

// docs:
//
// This is the main function for the logs command
// Steps:
//      - connects to kubernetes client
//      - returns the list of namespaces in Vec<String> format
//
//
// Returns a CliError if the connectiion to the kubeapi fails

pub async fn logs_command(
    service: Option<String>,
    component: Option<String>,
    namespace: Option<String>,
) -> Result<(), CliError> {
    match connect_to_client().await {
        Ok(_) => {
            let ns = namespace.unwrap_or_else(|| "cortexflow".to_string());

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
                std::process::exit(1);
            }

            let pods = match (service, component) {
                (Some(service_name), Some(component_str)) => {
                    let comp = Component::from(component_str);
                    println!(
                        "{} Getting logs for service '{}' with component '{:?}' in namespace '{}'",
                        "=====>".blue().bold(),
                        service_name,
                        comp,
                        ns
                    );
                    let service_pods = get_pods_for_service(&ns, &service_name).await?;
                    let component_pods = get_pods_for_component(&ns, &comp).await?;
                    service_pods
                        .into_iter()
                        .filter(|pod| component_pods.contains(pod))
                        .collect()
                }
                (Some(service_name), None) => {
                    println!(
                        "Getting logs for service '{}' in namespace '{}'",
                        service_name, ns
                    );
                    get_pods_for_service(&ns, &service_name).await?
                }
                (None, Some(component_str)) => {
                    let comp = Component::from(component_str);
                    println!(
                        "Getting logs for component '{:?}' in namespace '{}'",
                        comp, ns
                    );
                    get_pods_for_component(&ns, &comp).await?
                }
                (None, None) => {
                    println!(
                        "{} Getting logs for all pods in namespace '{}'",
                        "=====>".blue().bold(),
                        ns
                    );
                    get_all_pods(&ns).await?
                }
            };

            if pods.is_empty() {
                println!("No pods found matching the specified criteria");
                return Ok(());
            }

            for pod in pods {
                println!("{} Logs for pod: {:?}", "=====>".blue().bold(), pod);
                match Command::new(BASE_COMMAND)
                    .args(["logs", &pod, "-n", &ns, "--tail=50"])
                    .output()
                {
                    Ok(output) => {
                        if output.status.success() {
                            let stdout = str::from_utf8(&output.stdout).unwrap_or("");
                            if stdout.trim().is_empty() {
                                println!("No logs available for pod '{:?}'", pod);
                            } else {
                                println!("{}", stdout);
                            }
                        } else {
                            let stderr = str::from_utf8(&output.stderr).unwrap_or("Unknown error");
                            eprintln!("Error getting logs for pod '{:?}': {}", pod, stderr);
                        }
                    }
                    Err(err) => {
                        eprintln!(
                            "Failed to execute {} logs for pod '{:?}': {}",
                            BASE_COMMAND, pod, err
                        );
                    }
                }
            }

            Ok(())
        }
        Err(e) => {
            return Err(CliError::ClientError(Error::Api(ErrorResponse {
                status: "failed".to_string(),
                message: "Failed to connect to kubernetes client".to_string(),
                reason: e.to_string(),
                code: 404,
            })));
        }
    }
}

// docs:
//
// This is an auxiliary function used in the logs_command
// Steps:
//      - connects to kubernetes client
//      - returns true if the namespace exists or false if the namespace doesn't exists
//
//
// Returns a CliError if the connection fails

pub async fn check_namespace_exists(namespace: &str) -> Result<bool, CliError> {
    match connect_to_client().await {
        Ok(_) => {
            let output = Command::new(BASE_COMMAND)
                .args(["get", "namespace", namespace])
                .output();

            match output {
                Ok(output) => Ok(output.status.success()),
                Err(_) => Ok(false),
            }
        }
        Err(e) => {
            return Err(CliError::ClientError(Error::Api(ErrorResponse {
                status: "failed".to_string(),
                message: "Failed to connect to kubernetes client".to_string(),
                reason: e.to_string(),
                code: 404,
            })));
        }
    }
}

// docs:
//
// This function returns the available namespaces:
// Steps:
//      - connects to kubernetes client
//      - returns the list of namespaces in Vec<String> format
//
//
// Returns a CliError if the connectiion to the kubeapi fails

pub async fn get_available_namespaces() -> Result<Vec<String>, CliError> {
    match connect_to_client().await {
        Ok(_) => {
            let output = Command::new(BASE_COMMAND)
                .args([
                    "get",
                    "namespaces",
                    "--no-headers",
                    "-o",
                    "custom-columns=NAME:.metadata.name",
                ])
                .output();

            match output {
                Ok(output) if output.status.success() => {
                    let stdout = str::from_utf8(&output.stdout).unwrap_or("");
                    let ns = stdout
                        .lines()
                        .map(|line| line.trim().to_string())
                        .filter(|line| !line.is_empty())
                        .collect();
                    Ok(ns)
                }
                _ => Ok(Vec::new()),
            }
        }
        Err(e) => {
            return Err(CliError::ClientError(Error::Api(ErrorResponse {
                status: "failed".to_string(),
                message: "Failed to connect to kubernetes client".to_string(),
                reason: e.to_string(),
                code: 404,
            })));
        }
    }
}

// docs:
//
// This function returns the pods:
// Steps:
//      - connects to kubernetes client
//      - returns the list of pods associated with a kubernetes service filtering by labels in Vec<String> format
//
//
// Returns a CliError if the connectiion to the kubeapi fails

async fn get_pods_for_service(
    namespace: &str,
    service_name: &str,
) -> Result<Vec<String>, CliError> {
    match connect_to_client().await {
        Ok(_) => {
            let output = Command::new(BASE_COMMAND)
                .args([
                    "get",
                    "pods",
                    "-n",
                    namespace,
                    "-l",
                    &format!("app={}", service_name),
                    "--no-headers",
                    "-o",
                    "custom-columns=NAME:.metadata.name",
                ])
                .output();

            match output {
                Ok(output) if output.status.success() => {
                    let stdout = str::from_utf8(&output.stdout).unwrap_or("");
                    let pods = stdout
                        .lines()
                        .map(|line| line.trim().to_string())
                        .filter(|line| !line.is_empty())
                        .collect();
                    Ok(pods)
                }
                _ => Ok(Vec::new()),
            }
        }
        Err(e) => {
            return Err(CliError::ClientError(Error::Api(ErrorResponse {
                status: "failed".to_string(),
                message: "Failed to connect to kubernetes client".to_string(),
                reason: e.to_string(),
                code: 404,
            })));
        }
    }
}

// docs:
//
// This function returns the pods:
// Steps:
//      - connects to kubernetes client
//      - returns the list of pods associated with a componet object to dynamically construct the
//        label selector,in Vec<String> format
//
//
// Returns a CliError if the connectiion to the kubeapi fails

async fn get_pods_for_component(
    namespace: &str,
    component: &Component,
) -> Result<Vec<String>, CliError> {
    match connect_to_client().await {
        Ok(_) => {
            let output = Command::new(BASE_COMMAND)
                .args([
                    "get",
                    "pods",
                    "-n",
                    namespace,
                    "-l",
                    component.to_label_selector(),
                    "--no-headers",
                    "-o",
                    "custom-columns=NAME:.metadata.name",
                ])
                .output();

            match output {
                Ok(output) if output.status.success() => {
                    let stdout = str::from_utf8(&output.stdout).unwrap_or("");
                    let pods = stdout
                        .lines()
                        .map(|line| line.trim().to_string())
                        .filter(|line| !line.is_empty())
                        .collect();
                    Ok(pods)
                }
                _ => Ok(Vec::new()),
            }
        }
        Err(e) => {
            return Err(CliError::ClientError(Error::Api(ErrorResponse {
                status: "failed".to_string(),
                message: "Failed to connect to kubernetes client".to_string(),
                reason: e.to_string(),
                code: 404,
            })));
        }
    }
}

// docs:
//
// This function returns the available namespaces:
// Steps:
//      - connects to kubernetes client
//      - returns the list of all pods in Vec<String> format
//
//
// Returns a CliError if the connectiion to the kubeapi fails

async fn get_all_pods(namespace: &str) -> Result<Vec<String>, CliError> {
    match connect_to_client().await {
        Ok(_) => {
            let output = Command::new(BASE_COMMAND)
                .args([
                    "get",
                    "pods",
                    "-n",
                    namespace,
                    "--no-headers",
                    "-o",
                    "custom-columns=NAME:.metadata.name",
                ])
                .output();

            match output {
                Ok(output) if output.status.success() => {
                    let stdout = str::from_utf8(&output.stdout).unwrap_or("");
                    let pods = stdout
                        .lines()
                        .map(|line| line.trim().to_string())
                        .filter(|line| !line.is_empty())
                        .collect();
                    Ok(pods)
                }
                _ => Ok(Vec::new()),
            }
        }
        Err(e) => {
            return Err(CliError::ClientError(Error::Api(ErrorResponse {
                status: "failed".to_string(),
                message: "Failed to connect to kubernetes client".to_string(),
                reason: e.to_string(),
                code: 404,
            })));
        }
    }
}
