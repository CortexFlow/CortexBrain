use serde_json::Value;
use tracing::{error, info, instrument, warn};

#[instrument]
pub fn check_and_validate_pod(pod: &Value) -> Result<bool, String> {
    if let Some(containers) = pod["spec"]["containers"].as_array() {
        for container in containers {
            if let Some(name) = container["name"].as_str() {
                if name.contains("cortexflow-proxy") {
                    error!(
                        "The pod is not eligible for proxy injection. Sidecar proxy already present"
                    );
                    return Err("The pod is not eligible for proxy injection. Sidecar proxy already present".to_string());
                }
            }
        }
    }
    //check if the namespace allows the automatic insertion
    if let Some(namespace) = pod["metadata"]["namespace"].as_str() {
        let namespace_annotation = pod["metadata"]["annotations"].as_object();
        if let Some(annotations) = namespace_annotation {
            if let Some(value) = annotations.get("proxy-injection") {
                if value.as_str() == Some("disabled") {
                    warn!(
                        "Automatic namespace injection is disabled in namespace {}",
                        namespace
                    );
                    return Err("Automatic namespace injection is disabled".to_string());
                }
            }
        }
    }
    if let Some(value) = pod["metadata"]["annotations"].get("proxy-injection") {
        if value.as_str() == Some("disabled") {
            warn!("Automatic namespace injection is disabled in pod {}", pod);
            return Err("Automatic pod injection is disabled".to_string());
        }
    }

    //check if the pod has the ports open
    Ok(true)
}
