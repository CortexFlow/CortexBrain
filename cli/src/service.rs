use std::process::Command;
use std::str;

fn check_namespace_exists(namespace: &str) -> bool {
    let output = Command::new("kubectl")
        .args(["get", "namespace", namespace])
        .output();
    
    match output {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

fn get_available_namespaces() -> Vec<String> {
    let output = Command::new("kubectl")
        .args(["get", "namespaces", "--no-headers", "-o", "custom-columns=NAME:.metadata.name"])
        .output();
    
    match output {
        Ok(output) if output.status.success() => {
            let stdout = str::from_utf8(&output.stdout).unwrap_or("");
            stdout.lines()
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect()
        }
        _ => Vec::new(),
    }
}

pub fn list_services(namespace: Option<String>) {
    let ns = namespace.unwrap_or_else(|| "cortexflow".to_string());
    
    println!("Listing services in namespace: {}", ns);
    
    // Check if namespace exists first
    if !check_namespace_exists(&ns) {
        let available_namespaces = get_available_namespaces();
        
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
    
    // kubectl command to get services
    let output = Command::new("kubectl")
        .args(["get", "pods", "-n", &ns, "--no-headers"])
        .output();
    
    match output {
        Ok(output) => {
            if !output.status.success() {
                let error = str::from_utf8(&output.stderr).unwrap_or("Unknown error");
                eprintln!("Error executing kubectl: {}", error);
                std::process::exit(1);
            }
            
            let stdout = str::from_utf8(&output.stdout).unwrap_or("");
            
            if stdout.trim().is_empty() {
                println!("No pods found in namespace '{}'", ns);
                return;
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
                    
                    println!("{:<40} {:<20} {:<10} {:<10}", name, full_status, restarts, age);
                }
            }
        }
        Err(err) => {
            eprintln!("Failed to execute kubectl command: {}", err);
            eprintln!("Make sure kubectl is installed and configured properly");
            std::process::exit(1);
        }
    }
}

pub fn describe_service(service_name: String, namespace: Option<String>) {
    let ns = namespace.unwrap_or_else(|| "cortexflow".to_string());
    
    println!("Describing service '{}' in namespace: {}", service_name, ns);
    println!("{}", "=".repeat(60));
    
    // Check if namespace exists first
    if !check_namespace_exists(&ns) {
        let available_namespaces = get_available_namespaces();
        
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
        
        std::process::exit(1);
    }
    
    // Execute kubectl describe pod command
    let output = Command::new("kubectl")
        .args(["describe", "pod", &service_name, "-n", &ns])
        .output();
    
    match output {
        Ok(output) => {
            if !output.status.success() {
                let error = str::from_utf8(&output.stderr).unwrap_or("Unknown error");
                eprintln!("Error executing kubectl describe: {}", error);
                eprintln!("Make sure the pod '{}' exists in namespace '{}'", service_name, ns);
                std::process::exit(1);
            }
            
            let stdout = str::from_utf8(&output.stdout).unwrap_or("");
            
            if stdout.trim().is_empty() {
                println!("No description found for pod '{}'", service_name);
                return;
            }
            
            // Print the full kubectl describe output
            println!("{}", stdout);
        }
        Err(err) => {
            eprintln!("Failed to execute kubectl describe command: {}", err);
            eprintln!("Make sure kubectl is installed and configured properly");
            std::process::exit(1);
        }
    }
}