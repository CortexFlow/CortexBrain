use std::process::Command;
use std::str;

pub fn list_services(namespace: Option<String>) {
    let ns = namespace.unwrap_or_else(|| "cortexflow".to_string());
    
    println!("Listing services in namespace: {}", ns);
    
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