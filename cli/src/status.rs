use colored::Colorize;
use std::{ process::Command, str };
use clap::Args;
use kube::{ Error, core::ErrorResponse };

use crate::logs::{ get_available_namespaces, check_namespace_exists };
use crate::essential::{ BASE_COMMAND, connect_to_client };
use crate::errors::CliError;

// docs:
//
// Pure parsing helper shared by get_pods_status/get_services_status so the
// column-parsing logic can be unit tested with stubbed kubectl output.

fn parse_status_columns(stdout: &str, min_parts: usize) -> Vec<(String, String, String)> {
    stdout
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= min_parts {
                Some((
                    parts[0].to_string(),
                    parts[1].to_string(),
                    parts[2].to_string(),
                ))
            } else {
                None
            }
        })
        .collect()
}

#[derive(Debug)]
pub enum OutputFormat {
    Text,
    Json,
    Yaml,
}

impl From<String> for OutputFormat {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "json" => OutputFormat::Json,
            "yaml" => OutputFormat::Yaml,
            _ => OutputFormat::Text,
        }
    }
}

#[derive(Args, Debug, Clone)]
pub struct StatusArgs {
    #[arg(long)]
    pub output: Option<String>,
    #[arg(long)]
    pub namespace: Option<String>,
}

// docs:
//
// This is the main function for the status command. The status command display all the pods and services status in 3 types of format : Text, Json,Yaml
// defaul type is Text
//
// Steps:
//      - connects to kubernetes client
//      - check if the given namespace exists
//      - if the namespace exists
//          - return the pods status and the service status for all the pods and services in the namespace
//      - else
//          - return a failed state
//
// Returns a CliError if the connection fails

pub async fn status_command(
    output_format: Option<String>,
    namespace: Option<String>
) -> Result<(), CliError> {
    match connect_to_client().await {
        Ok(_) => {
            let format = output_format.map(OutputFormat::from).unwrap_or(OutputFormat::Text);
            let ns = namespace.unwrap_or_else(|| "cortexflow".to_string());

            println!(
                "{} {} {}",
                "=====>".blue().bold(),
                "Checking CortexFlow status for namespace: ",
                ns
            );

            // namespace checking
            let namespace_status = check_namespace_exists(&ns).await?;

            // If namespace doesn't exist, display error with available namespaces and exit
            if !namespace_status {
                let available_namespaces = get_available_namespaces().await?;

                match format {
                    OutputFormat::Text => {
                        println!("\n❌ Namespace Status Check Failed");
                        println!("{}", "=".repeat(50));
                        println!("  ❌ {} namespace: NOT FOUND", ns);

                        if !available_namespaces.is_empty() {
                            println!("\n📋 Available namespaces:");
                            for available_ns in &available_namespaces {
                                println!("  • {}", available_ns);
                            }
                        }
                    }
                    OutputFormat::Json => {
                        println!("{{");
                        println!("  \"error\": \"{} namespace not found\",", ns);
                        println!("  \"namespace\": {{");
                        println!("    \"name\": \"{}\",", ns);
                        println!("    \"exists\": false");
                        println!("  }},");
                        println!("  \"available_namespaces\": [");
                        for (i, ns) in available_namespaces.iter().enumerate() {
                            let comma = if i == available_namespaces.len() - 1 { "" } else { "," };
                            println!("    \"{}\"{}", ns, comma);
                        }
                        println!("  ]");
                        println!("}}");
                    }
                    OutputFormat::Yaml => {
                        println!("error: {} namespace not found", ns);
                        println!("namespace:");
                        println!("  name: {}", ns);
                        println!("  exists: false");
                        println!("available_namespaces:");
                        for ns in &available_namespaces {
                            println!("  - {}", ns);
                        }
                    }
                }
            }

            // get pods and services only if namespace exists
            let pods_status = get_pods_status(&ns).await?;
            let services_status = get_services_status(&ns).await?;

            // display options (format)
            match format {
                OutputFormat::Text => {
                    display_text_format(&ns, namespace_status, pods_status, services_status);
                    Ok(())
                }
                OutputFormat::Json => {
                    display_json_format(&ns, namespace_status, pods_status, services_status);
                    Ok(())
                }
                OutputFormat::Yaml => {
                    display_yaml_format(&ns, namespace_status, pods_status, services_status);
                    Ok(())
                }
            }
        }
        Err(e) => {
            return Err(
                CliError::ClientError(
                    Error::Api(ErrorResponse {
                        status: "failed".to_string(),
                        message: "Failed to connect to kubernetes client".to_string(),
                        reason: e.to_string(),
                        code: 404,
                    })
                )
            )
        }
    }
}

// docs:
//
// This is an auxiliary function that returns the status for a given pod
// Steps:
//      - connects to kubernetes client
//      - return the pod status in this format: (name,ready?,status)
//
// Returns a CliError if the connection fails

async fn get_pods_status(namespace: &str) -> Result<Vec<(String, String, String)>, CliError> {
    match connect_to_client().await {
        Ok(_) => {
            let output = Command::new(BASE_COMMAND)
                .args(["get", "pods", "-n", namespace, "--no-headers"])
                .output();

            match output {
                Ok(output) if output.status.success() => {
                    let stdout = str::from_utf8(&output.stdout).unwrap_or("");
                    Ok(parse_status_columns(stdout, 3))
                }
                _ => Ok(Vec::new()),
            }
        }
        Err(e) => {
            return Err(
                CliError::ClientError(
                    Error::Api(ErrorResponse {
                        status: "failed".to_string(),
                        message: "Failed to connect to kubernetes client".to_string(),
                        reason: e.to_string(),
                        code: 404,
                    })
                )
            )
        }
    }
}

// docs:
//
// This is an auxiliary function that returns the status for a given service
// Steps:
//      - connects to kubernetes client
//      - return the service status in this format: (name,type,cluster ips)
//
// Returns a CliError if the connection fails

async fn get_services_status(namespace: &str) -> Result<Vec<(String, String, String)>, CliError> {
    match connect_to_client().await {
        Ok(_) => {
            let output = Command::new(BASE_COMMAND)
                .args(["get", "services", "-n", namespace, "--no-headers"])
                .output();

            match output {
                Ok(output) if output.status.success() => {
                    let stdout = str::from_utf8(&output.stdout).unwrap_or("");
                    Ok(parse_status_columns(stdout, 4))
                }
                _ => Ok(Vec::new()),
            }
        }
        Err(e) => {
            return Err(
                CliError::ClientError(
                    Error::Api(ErrorResponse {
                        status: "failed".to_string(),
                        message: "Failed to connect to kubernetes client".to_string(),
                        reason: e.to_string(),
                        code: 404,
                    })
                )
            )
        }
    }
}

// docs: displays outputs in a text format

fn display_text_format(
    ns: &str,
    namespace_exists: bool,
    pods: Vec<(String, String, String)>,
    services: Vec<(String, String, String)>
) {
    println!("\n🔍 CortexFlow Status Report");
    println!("{}", "=".repeat(50));

    println!("\n📦 Namespace Status:");
    if namespace_exists {
        println!("  ✅ {} namespace: EXISTS", ns);
    } else {
        println!("  ❌ {} namespace: NOT FOUND", ns);
    }

    println!("\n🚀 Pods Status:");
    if pods.is_empty() {
        println!("  ⚠️  No pods found in {} namespace", ns);
    } else {
        for (name, ready, status) in pods {
            let icon = if status == "Running" { "✅" } else { "⚠️" };
            println!("  {} {}: {} ({})", icon, name, status, ready);
        }
    }

    println!("\n🌐 Services Status:");
    if services.is_empty() {
        println!("  ⚠️  No services found in {} namespace", ns);
    } else {
        for (name, service_type, cluster_ip) in services {
            println!("  🔗 {}: {} ({})", name, service_type, cluster_ip);
        }
    }

    println!("\n{}", "=".repeat(50));
}

// docs: displays outputs in a json format

fn display_json_format(
    ns: &str,
    namespace_exists: bool,
    pods: Vec<(String, String, String)>,
    services: Vec<(String, String, String)>
) {
    println!("{{");
    println!("  \"namespace\": {{");
    println!("    \"name\": \"{}\",", ns);
    println!("    \"exists\": {}", namespace_exists);
    println!("  }},");

    println!("  \"pods\": [");
    for (i, (name, ready, status)) in pods.iter().enumerate() {
        let comma = if i == pods.len() - 1 { "" } else { "," };
        println!("    {{");
        println!("      \"name\": \"{}\",", name);
        println!("      \"ready\": \"{}\",", ready);
        println!("      \"status\": \"{}\"", status);
        println!("    }}{}", comma);
    }
    println!("  ],");

    println!("  \"services\": [");
    for (i, (name, service_type, cluster_ip)) in services.iter().enumerate() {
        let comma = if i == services.len() - 1 { "" } else { "," };
        println!("    {{");
        println!("      \"name\": \"{}\",", name);
        println!("      \"type\": \"{}\",", service_type);
        println!("      \"cluster_ip\": \"{}\"", cluster_ip);
        println!("    }}{}", comma);
    }
    println!("  ]");
    println!("}}");
}

// docs: displays outputs in a yaml format

fn display_yaml_format(
    ns: &str,
    namespace_exists: bool,
    pods: Vec<(String, String, String)>,
    services: Vec<(String, String, String)>
) {
    println!("namespace:");
    println!("  name: {}", ns);
    println!("  exists: {}", namespace_exists);

    println!("pods:");
    for (name, ready, status) in pods {
        println!("  - name: {}", name);
        println!("    ready: {}", ready);
        println!("    status: {}", status);
    }

    println!("services:");
    for (name, service_type, cluster_ip) in services {
        println!("  - name: {}", name);
        println!("    type: {}", service_type);
        println!("    cluster_ip: {}", cluster_ip);
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format_from_json() {
        assert!(matches!(OutputFormat::from("json".to_string()), OutputFormat::Json));
        assert!(matches!(OutputFormat::from("JSON".to_string()), OutputFormat::Json));
    }

    #[test]
    fn test_output_format_from_yaml() {
        assert!(matches!(OutputFormat::from("yaml".to_string()), OutputFormat::Yaml));
    }

    #[test]
    fn test_output_format_from_defaults_to_text() {
        assert!(matches!(OutputFormat::from("unknown".to_string()), OutputFormat::Text));
    }

    #[test]
    fn test_parse_status_columns_pods() {
        let stdout = "pod-a   1/1   Running\npod-b   0/1   Pending\n";
        let parsed = parse_status_columns(stdout, 3);
        assert_eq!(
            parsed,
            vec![
                ("pod-a".to_string(), "1/1".to_string(), "Running".to_string()),
                ("pod-b".to_string(), "0/1".to_string(), "Pending".to_string()),
            ]
        );
    }

    #[test]
    fn test_parse_status_columns_services() {
        let stdout = "svc-a   ClusterIP   10.0.0.1   443/TCP\n";
        let parsed = parse_status_columns(stdout, 4);
        assert_eq!(
            parsed,
            vec![("svc-a".to_string(), "ClusterIP".to_string(), "10.0.0.1".to_string())]
        );
    }

    #[test]
    fn test_parse_status_columns_skips_short_lines() {
        let stdout = "only-one-column\n";
        assert!(parse_status_columns(stdout, 3).is_empty());
    }

    #[test]
    fn test_parse_status_columns_empty_input() {
        assert!(parse_status_columns("", 3).is_empty());
    }

    #[test]
    fn test_display_text_format_does_not_panic() {
        display_text_format(
            "cortexflow",
            true,
            vec![("pod-a".to_string(), "1/1".to_string(), "Running".to_string())],
            vec![("svc-a".to_string(), "ClusterIP".to_string(), "10.0.0.1".to_string())],
        );
    }

    #[test]
    fn test_display_json_format_does_not_panic() {
        display_json_format("cortexflow", false, Vec::new(), Vec::new());
    }

    #[test]
    fn test_display_yaml_format_does_not_panic() {
        display_yaml_format("cortexflow", false, Vec::new(), Vec::new());
    }
}