use std::process::Command;
use std::str;

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

pub fn status_command(output_format: Option<String>) {
    let format = output_format.map(OutputFormat::from).unwrap_or(OutputFormat::Text);
    
    println!("Checking CortexFlow status...");
    
    // namespace checking
    let namespace_status = check_namespace_exists("cortexflow");
    
    // get pods
    let pods_status = get_pods_status("cortexflow");
    
    // get services
    let services_status = get_services_status("cortexflow");
    
    // display options (format)
    match format {
        OutputFormat::Text => display_text_format(namespace_status, pods_status, services_status),
        OutputFormat::Json => display_json_format(namespace_status, pods_status, services_status),
        OutputFormat::Yaml => display_yaml_format(namespace_status, pods_status, services_status),
    }
}

fn check_namespace_exists(namespace: &str) -> bool {
    let output = Command::new("kubectl")
        .args(["get", "namespace", namespace])
        .output();
    
    match output {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

fn get_pods_status(namespace: &str) -> Vec<(String, String, String)> {
    let output = Command::new("kubectl")
        .args(["get", "pods", "-n", namespace, "--no-headers"])
        .output();
    
    match output {
        Ok(output) if output.status.success() => {
            let stdout = str::from_utf8(&output.stdout).unwrap_or("");
            stdout.lines()
                .filter_map(|line| {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 3 {
                        Some((
                            parts[0].to_string(),  // name
                            parts[1].to_string(),  // ready
                            parts[2].to_string(),  // status
                        ))
                    } else {
                        None
                    }
                })
                .collect()
        }
        _ => Vec::new(),
    }
}

fn get_services_status(namespace: &str) -> Vec<(String, String, String)> {
    let output = Command::new("kubectl")
        .args(["get", "services", "-n", namespace, "--no-headers"])
        .output();
    
    match output {
        Ok(output) if output.status.success() => {
            let stdout = str::from_utf8(&output.stdout).unwrap_or("");
            stdout.lines()
                .filter_map(|line| {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 4 {
                        Some((
                            parts[0].to_string(),  // name
                            parts[1].to_string(),  // type
                            parts[2].to_string(),  // cluster ips
                        ))
                    } else {
                        None
                    }
                })
                .collect()
        }
        _ => Vec::new(),
    }
}

fn display_text_format(namespace_exists: bool, pods: Vec<(String, String, String)>, services: Vec<(String, String, String)>) {
    println!("\n🔍 CortexFlow Status Report");
    println!("{}", "=".repeat(50));
    
    println!("\n📦 Namespace Status:");
    if namespace_exists {
        println!("  ✅ cortexflow namespace: EXISTS");
    } else {
        println!("  ❌ cortexflow namespace: NOT FOUND");
    }
    
    println!("\n🚀 Pods Status:");
    if pods.is_empty() {
        println!("  ⚠️  No pods found in cortexflow namespace");
    } else {
        for (name, ready, status) in pods {
            let icon = if status == "Running" { "✅" } else { "⚠️" };
            println!("  {} {}: {} ({})", icon, name, status, ready);
        }
    }
    
    println!("\n🌐 Services Status:");
    if services.is_empty() {
        println!("  ⚠️  No services found in cortexflow namespace");
    } else {
        for (name, service_type, cluster_ip) in services {
            println!("  🔗 {}: {} ({})", name, service_type, cluster_ip);
        }
    }
    
    println!("\n{}", "=".repeat(50));
}

fn display_json_format(namespace_exists: bool, pods: Vec<(String, String, String)>, services: Vec<(String, String, String)>) {
    println!("{{");
    println!("  \"namespace\": {{");
    println!("    \"name\": \"cortexflow\",");
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

fn display_yaml_format(namespace_exists: bool, pods: Vec<(String, String, String)>, services: Vec<(String, String, String)>) {
    println!("namespace:");
    println!("  name: cortexflow");
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