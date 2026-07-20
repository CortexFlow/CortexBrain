#[cfg(feature = "experimental")]
use anyhow::Error;
#[cfg(feature = "experimental")]
use std::fs;

/// Supported runtime prefixes for extracting the container ID from a cgroup path.
const RUNTIME_PREFIXES: &[&str] = &["docker-", "cri-containerd-", "crio-"];

/// Extract the container ID (e.g. Docker runtime ID) from a cgroup filesystem path.
///
/// Supports multiple prefixes: docker-, cri-containerd-, crio-.
#[cfg(feature = "experimental")]
pub fn extract_container_id(cgroup_path: &str) -> Result<String, Error> {
    let splits: Vec<&str> = cgroup_path.split('/').collect();

    for prefix in RUNTIME_PREFIXES {
        if let Ok(index) = extract_target_from_splits(&splits, prefix) {
            let id = splits[index]
                .trim_start_matches(prefix)
                .trim_end_matches(".scope");
            return Ok(id.to_string());
        }
    }

    Err(Error::msg(format!(
        "No known runtime prefix found in cgroup path: {}",
        cgroup_path
    )))
}

/// Extract the Pod UID from a Kubernetes cgroup filesystem path.
///
/// Example inputs:
/// - `/sys/fs/cgroup/kubepods.slice/kubepods-besteffort.slice/kubepods-besteffort-pod231bd2d7_0f09_4781_a4e1_e4ea026342dd.slice`
/// - `/sys/fs/cgroup/kubelet.slice/kubelet-kubepods.slice/kubelet-kubepods-besteffort.slice/kubelet-kubepods-besteffort-pod231bd2d7_0f09_4781_a4e1_e4ea026342dd.slice`
#[cfg(feature = "experimental")]
pub fn extract_pod_uid(cgroup_path: &str) -> Result<String, Error> {
    let splits: Vec<&str> = cgroup_path.split('/').collect();

    let index = extract_target_from_splits(&splits, "-pod")?;

    let pod_split = splits[index]
        .trim_start_matches("kubelet-kubepods-besteffort-")
        .trim_start_matches("kubelet-kubepods-burstable-")
        .trim_start_matches("kubepods-besteffort-")
        .trim_start_matches("kubepods-burstable-");

    let uid_ = pod_split
        .trim_start_matches("pod")
        .trim_end_matches(".slice");

    let uid = uid_.replace('_', "-");
    Ok(uid)
}

/// Scan a given cgroup directory and return subdirectory paths.
///
/// If `path` does not exist or is empty, falls back to the default K8s kubepods.slice path.
#[cfg(feature = "experimental")]
pub fn scan_cgroup_paths(path: &str) -> Result<Vec<String>, Error> {
    let mut cgroup_paths: Vec<String> = Vec::new();
    let default_path = "/sys/fs/cgroup/kubepods.slice";

    let target_path = if path.is_empty() || fs::metadata(path).is_err() {
        default_path
    } else {
        path
    };

    let entries = match fs::read_dir(target_path) {
        Ok(entries) => entries,
        Err(e) => {
            tracing::error!("Error reading cgroup directory {:?}: {}", target_path, e);
            return Ok(cgroup_paths);
        }
    };

    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_dir() {
                if let Some(path_str) = path.to_str() {
                    cgroup_paths.push(path_str.to_string());
                }
            }
        }
    }

    Ok(cgroup_paths)
}

#[cfg(feature = "experimental")]
fn extract_target_from_splits(splits: &[&str], target: &str) -> Result<usize, Error> {
    for (index, split) in splits.iter().enumerate() {
        if split.contains(target) {
            return Ok(index);
        }
    }
    Err(Error::msg(format!("'{target}' word not found in split")))
}

#[cfg(feature = "experimental")]
mod tests {
    use super::*;

    #[test]
    fn extract_uid_from_string() {
        let cgroup_paths = vec![
            "/sys/fs/cgroup/kubepods.slice/kubepods-besteffort.slice/kubepods-besteffort-pod231bd2d7_0f09_4781_a4e1_e4ea026342dd.slice".to_string(),
            "/sys/fs/cgroup/kubelet.slice/kubelet-kubepods.slice/kubelet-kubepods-besteffort.slice/kubelet-kubepods-besteffort-pod231bd2d7_0f09_4781_a4e1_e4ea026342dd.slice".to_string(),
        ];

        let mut uid_vec = Vec::<String>::new();

        for cgroup_path in cgroup_paths {
            let uid = extract_pod_uid(&cgroup_path)
                .map_err(|e| format!("An error occurred {}", e))
                .unwrap();
            uid_vec.push(uid);
        }

        let check = vec![
            "231bd2d7-0f09-4781-a4e1-e4ea026342dd".to_string(),
            "231bd2d7-0f09-4781-a4e1-e4ea026342dd".to_string(),
        ];

        assert_eq!(uid_vec, check);
    }

    #[test]
    fn test_extract_target_index() {
        let cgroup_paths = vec![
            "/sys/fs/cgroup/kubepods.slice/kubepods-besteffort.slice/kubepods-besteffort-pod231bd2d7_0f09_4781_a4e1_e4ea026342dd.slice".to_string(),
            "/sys/fs/cgroup/kubelet.slice/kubelet-kubepods.slice/kubelet-kubepods-besteffort.slice/kubelet-kubepods-besteffort-pod231bd2d7_0f09_4781_a4e1_e4ea026342dd.slice".to_string(),
        ];

        let mut index_vec = Vec::<usize>::new();
        for cgroup_path in cgroup_paths {
            let splits: Vec<&str> = cgroup_path.split('/').collect();

            let target_index = extract_target_from_splits(&splits, "-pod").unwrap();
            index_vec.push(target_index);
        }
        let index_check = vec![6, 7];
        assert_eq!(index_vec, index_check);
    }

    #[test]
    fn extract_docker_id() {
        let cgroup_paths = vec![
            "/sys/fs/cgroup/kubepods.slice/kubepods-besteffort.slice/kubepods-besteffort-pod17fd3f7c_37e4_4009_8c38_e58b30691af3.slice/docker-13abd64c0ba349975a762476c9703b642d18077eabeb3aa1d941132048afc861.scope".to_string(),
            "/sys/fs/cgroup/kubelet.slice/kubelet-kubepods.slice/kubelet-kubepods-besteffort.slice/kubelet-kubepods-besteffort-pod17fd3f7c_37e4_4009_8c38_e58b30691af3.slice/docker-13abd64c0ba349975a762476c9703b642d18077eabeb3aa1d941132048afc861.scope".to_string(),
        ];

        let mut id_vec = Vec::<String>::new();
        for cgroup_path in cgroup_paths {
            let id = extract_container_id(&cgroup_path).unwrap();
            id_vec.push(id);
        }
        let id_check = vec![
            "13abd64c0ba349975a762476c9703b642d18077eabeb3aa1d941132048afc861".to_string(),
            "13abd64c0ba349975a762476c9703b642d18077eabeb3aa1d941132048afc861".to_string(),
        ];
        assert_eq!(id_vec, id_check);
    }
}
