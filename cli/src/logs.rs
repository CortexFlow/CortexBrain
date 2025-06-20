use std::process::Command;
use std::str;

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

fn get_pods_for_service(namespace: &str, service_name: &str) -> Vec<String> {
    let output = Command::new("kubectl")
        .args(["get", "pods", "-n", namespace, "-l", &format!("app={}", service_name), "--no-headers", "-o", "custom-columns=NAME:.metadata.name"])
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

fn get_pods_for_component(namespace: &str, component: &Component) -> Vec<String> {
    let output = Command::new("kubectl")
        .args(["get", "pods", "-n", namespace, "-l", component.to_label_selector(), "--no-headers", "-o", "custom-columns=NAME:.metadata.name"])
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

fn get_all_pods(namespace: &str) -> Vec<String> {
    let output = Command::new("kubectl")
        .args(["get", "pods", "-n", namespace, "--no-headers", "-o", "custom-columns=NAME:.metadata.name"])
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

pub fn logs_command(service: Option<String>, component: Option<String>, namespace: Option<String>) {
    let ns = namespace.unwrap_or_else(|| "cortexflow".to_string());
    
    // namespace check
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
    
    // determine pods.
    let pods = match (service, component) {
        (Some(service_name), Some(component_str)) => {
            let comp = Component::from(component_str);
            println!("Getting logs for service '{}' with component '{:?}' in namespace '{}'", service_name, comp, ns);
            
            let service_pods = get_pods_for_service(&ns, &service_name);
            let component_pods = get_pods_for_component(&ns, &comp);
            
            // intersection
            service_pods.into_iter()
                .filter(|pod| component_pods.contains(pod))
                .collect()
        }
        (Some(service_name), None) => {
            //only service
            println!("Getting logs for service '{}' in namespace '{}'", service_name, ns);
            get_pods_for_service(&ns, &service_name)
        }
        (None, Some(component_str)) => {
            //only component
            let comp = Component::from(component_str);
            println!("Getting logs for component '{:?}' in namespace '{}'", comp, ns);
            get_pods_for_component(&ns, &comp)
        }
        (None, None) => {
            //neither, get all
            println!("Getting logs for all pods in namespace '{}'", ns);
            get_all_pods(&ns)
        }
    };
    
    if pods.is_empty() {
        println!("No pods found matching the specified criteria");
        return;
    }

    for pod in pods {
        println!("\n{}", "=".repeat(80));
        println!("📋 Logs for pod: {}", pod);
        println!("{}", "=".repeat(80));
        
        let output = Command::new("kubectl")
            .args(["logs", &pod, "-n", &ns, "--tail=50"])
            .output();
        
        match output {
            Ok(output) => {
                if output.status.success() {
                    let stdout = str::from_utf8(&output.stdout).unwrap_or("");
                    if stdout.trim().is_empty() {
                        println!("No logs available for pod '{}'", pod);
                    } else {
                        println!("{}", stdout);
                    }
                } else {
                    let stderr = str::from_utf8(&output.stderr).unwrap_or("Unknown error");
                    eprintln!("Error getting logs for pod '{}': {}", pod, stderr);
                }
            }
            Err(err) => {
                eprintln!("Failed to execute kubectl logs for pod '{}': {}", pod, err);
            }
        }
    }
}