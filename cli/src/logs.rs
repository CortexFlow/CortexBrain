use crate::command_runner::{CommandRunner, RealCommandRunner};
use crate::errors::CliError;
use crate::essential::{BASE_COMMAND, connect_to_client};
use clap::Args;
use colored::Colorize;
use kube::{Error, core::ErrorResponse};
use std::{process::Command, result::Result::Ok, str};

fn parse_lines(stdout: &[u8]) -> Vec<String> {
    str::from_utf8(stdout)
        .unwrap_or("")
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect()
}

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
                            return Err(CliError::BaseError {
                                reason: format!(
                                    "Error getting logs for pod '{:?}': {}",
                                    pod, stderr
                                ),
                            });
                        }
                    }
                    Err(err) => {
                        return Err(CliError::BaseError {
                            reason: format!(
                                "Failed to execute {} logs for pod '{:?}': {}",
                                BASE_COMMAND, pod, err
                            ),
                        });
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
        Ok(_) => Ok(check_namespace_exists_with(&RealCommandRunner, namespace)),
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

fn check_namespace_exists_with(runner: &dyn CommandRunner, namespace: &str) -> bool {
    let args = ["get".to_string(), "namespace".to_string(), namespace.to_string()];
    match runner.run(BASE_COMMAND, &args) {
        Ok(output) => output.status.success(),
        Err(_) => false,
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
        Ok(_) => Ok(get_available_namespaces_with(&RealCommandRunner)),
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

fn get_available_namespaces_with(runner: &dyn CommandRunner) -> Vec<String> {
    let args = [
        "get".to_string(),
        "namespaces".to_string(),
        "--no-headers".to_string(),
        "-o".to_string(),
        "custom-columns=NAME:.metadata.name".to_string(),
    ];

    match runner.run(BASE_COMMAND, &args) {
        Ok(output) if output.status.success() => parse_lines(&output.stdout),
        _ => Vec::new(),
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
        Ok(_) => Ok(get_pods_for_service_with(
            &RealCommandRunner,
            namespace,
            service_name,
        )),
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

fn get_pods_for_service_with(
    runner: &dyn CommandRunner,
    namespace: &str,
    service_name: &str,
) -> Vec<String> {
    let args = [
        "get".to_string(),
        "pods".to_string(),
        "-n".to_string(),
        namespace.to_string(),
        "-l".to_string(),
        format!("app={}", service_name),
        "--no-headers".to_string(),
        "-o".to_string(),
        "custom-columns=NAME:.metadata.name".to_string(),
    ];

    match runner.run(BASE_COMMAND, &args) {
        Ok(output) if output.status.success() => parse_lines(&output.stdout),
        _ => Vec::new(),
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
        Ok(_) => Ok(get_pods_for_component_with(
            &RealCommandRunner,
            namespace,
            component,
        )),
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

fn get_pods_for_component_with(
    runner: &dyn CommandRunner,
    namespace: &str,
    component: &Component,
) -> Vec<String> {
    let args = [
        "get".to_string(),
        "pods".to_string(),
        "-n".to_string(),
        namespace.to_string(),
        "-l".to_string(),
        component.to_label_selector().to_string(),
        "--no-headers".to_string(),
        "-o".to_string(),
        "custom-columns=NAME:.metadata.name".to_string(),
    ];

    match runner.run(BASE_COMMAND, &args) {
        Ok(output) if output.status.success() => parse_lines(&output.stdout),
        _ => Vec::new(),
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
        Ok(_) => Ok(get_all_pods_with(&RealCommandRunner, namespace)),
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

fn get_all_pods_with(runner: &dyn CommandRunner, namespace: &str) -> Vec<String> {
    let args = [
        "get".to_string(),
        "pods".to_string(),
        "-n".to_string(),
        namespace.to_string(),
        "--no-headers".to_string(),
        "-o".to_string(),
        "custom-columns=NAME:.metadata.name".to_string(),
    ];

    match runner.run(BASE_COMMAND, &args) {
        Ok(output) if output.status.success() => parse_lines(&output.stdout),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command_runner::test_support::StubCommandRunner;

    // Component

    #[test]
    fn test_component_from_control_plane() {
        assert!(matches!(
            Component::from("control-plane".to_string()),
            Component::ControlPlane
        ));
    }

    #[test]
    fn test_component_from_data_plane() {
        assert!(matches!(
            Component::from("data-plane".to_string()),
            Component::DataPlane
        ));
    }

    #[test]
    fn test_component_from_is_case_insensitive() {
        assert!(matches!(
            Component::from("Data-Plane".to_string()),
            Component::DataPlane
        ));
    }

    #[test]
    fn test_component_from_unknown_defaults_to_control_plane() {
        assert!(matches!(
            Component::from("unknown".to_string()),
            Component::ControlPlane
        ));
    }

    #[test]
    fn test_component_to_label_selector() {
        assert_eq!(
            Component::ControlPlane.to_label_selector(),
            "component=control-plane"
        );
        assert_eq!(
            Component::DataPlane.to_label_selector(),
            "component=data-plane"
        );
    }

    // parse_lines

    #[test]
    fn test_parse_lines_filters_empty_lines_and_trims() {
        let stdout = b"  pod-a  \n\npod-b\n   \n";
        let parsed = parse_lines(stdout);
        assert_eq!(parsed, vec!["pod-a".to_string(), "pod-b".to_string()]);
    }

    #[test]
    fn test_parse_lines_empty_input() {
        assert!(parse_lines(b"").is_empty());
    }

    // check_namespace_exists_with

    #[test]
    fn test_check_namespace_exists_with_success() {
        let runner = StubCommandRunner::success("");
        assert!(check_namespace_exists_with(&runner, "cortexflow"));
    }

    #[test]
    fn test_check_namespace_exists_with_failure() {
        let runner = StubCommandRunner::failure("not found");
        assert!(!check_namespace_exists_with(&runner, "missing"));
    }

    #[test]
    fn test_check_namespace_exists_with_io_error() {
        let runner = StubCommandRunner::io_error();
        assert!(!check_namespace_exists_with(&runner, "cortexflow"));
    }

    // get_available_namespaces_with

    #[test]
    fn test_get_available_namespaces_with_success() {
        let runner = StubCommandRunner::success("default\ncortexflow\nkube-system\n");
        let namespaces = get_available_namespaces_with(&runner);
        assert_eq!(
            namespaces,
            vec![
                "default".to_string(),
                "cortexflow".to_string(),
                "kube-system".to_string()
            ]
        );
    }

    #[test]
    fn test_get_available_namespaces_with_empty_output() {
        let runner = StubCommandRunner::success("");
        assert!(get_available_namespaces_with(&runner).is_empty());
    }

    #[test]
    fn test_get_available_namespaces_with_command_failure() {
        let runner = StubCommandRunner::failure("connection refused");
        assert!(get_available_namespaces_with(&runner).is_empty());
    }

    // get_pods_for_service_with

    #[test]
    fn test_get_pods_for_service_with_success() {
        let runner = StubCommandRunner::success("pod-a\npod-b\n");
        let pods = get_pods_for_service_with(&runner, "cortexflow", "my-service");
        assert_eq!(pods, vec!["pod-a".to_string(), "pod-b".to_string()]);
    }

    #[test]
    fn test_get_pods_for_service_with_no_matches() {
        let runner = StubCommandRunner::success("");
        assert!(get_pods_for_service_with(&runner, "cortexflow", "unknown-service").is_empty());
    }

    #[test]
    fn test_get_pods_for_service_with_command_failure() {
        let runner = StubCommandRunner::failure("error");
        assert!(get_pods_for_service_with(&runner, "cortexflow", "my-service").is_empty());
    }

    // get_pods_for_component_with

    #[test]
    fn test_get_pods_for_component_with_success() {
        let runner = StubCommandRunner::success("agent-pod\n");
        let pods = get_pods_for_component_with(&runner, "cortexflow", &Component::DataPlane);
        assert_eq!(pods, vec!["agent-pod".to_string()]);
    }

    #[test]
    fn test_get_pods_for_component_with_no_matches() {
        let runner = StubCommandRunner::success("");
        assert!(
            get_pods_for_component_with(&runner, "cortexflow", &Component::ControlPlane)
                .is_empty()
        );
    }

    #[test]
    fn test_get_pods_for_component_with_command_failure() {
        let runner = StubCommandRunner::failure("error");
        assert!(
            get_pods_for_component_with(&runner, "cortexflow", &Component::ControlPlane)
                .is_empty()
        );
    }

    // get_all_pods_with

    #[test]
    fn test_get_all_pods_with_success() {
        let runner = StubCommandRunner::success("pod-a\npod-b\npod-c\n");
        let pods = get_all_pods_with(&runner, "cortexflow");
        assert_eq!(
            pods,
            vec![
                "pod-a".to_string(),
                "pod-b".to_string(),
                "pod-c".to_string()
            ]
        );
    }

    #[test]
    fn test_get_all_pods_with_empty_namespace() {
        let runner = StubCommandRunner::success("");
        assert!(get_all_pods_with(&runner, "empty_namespace").is_empty());
    }

    #[test]
    fn test_get_all_pods_with_command_failure() {
        let runner = StubCommandRunner::failure("namespace not found");
        assert!(get_all_pods_with(&runner, "non_existent_namespace").is_empty());
    }

    #[test]
    fn test_get_all_pods_with_io_error() {
        let runner = StubCommandRunner::io_error();
        assert!(get_all_pods_with(&runner, "cortexflow").is_empty());
    }
}