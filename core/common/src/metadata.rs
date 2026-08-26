use anyhow::Error;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::service_cache::ServiceCache;

// TODO: add containerd tracing and mutual exclusion for docker environments

/// Detected container runtime.
#[derive(Debug, Clone, PartialEq)]
pub enum ContainerRuntime {
    Docker,
    Kubernetes,
    Unknown,
}

/// Process / container metadata enriched by service discovery.
/// Built from raw eBPF data and enriched with a cgroup -> Docker -> K8s lookup.
#[derive(Debug, Clone)]
pub struct Metadata {
    pub tgid: Option<u32>,
    pub cgroup_id: Option<u64>,
    pub command: String,
    pub runtime: ContainerRuntime,
    pub container_name: Option<String>,
    pub container_id: Option<String>,
    pub pod_name: Option<String>,
    pub namespace: Option<String>, // TODO: implement this one
}

impl Metadata {
    /// Build base metadata from eBPF data.
    pub fn from_ebpf(tgid: Option<u32>, cgroup_id: Option<u64>, command: &[u8]) -> Self {
        let command = String::from_utf8_lossy(command)
            .trim_end_matches('\0')
            .to_string();
        Self {
            tgid,
            cgroup_id,
            command,
            runtime: ContainerRuntime::Unknown,
            container_name: None,
            container_id: None,
            pod_name: None,
            namespace: None,
        }
    }

    /// Lookup rules: prefer cgroup_id (v2), fall back to /proc/<tgid>/cgroup (v1).
    ///
    /// 1. If cgroup_id is set and non-zero, resolve it via /sys/fs/cgroup (cgroup v2).
    /// 2. Otherwise (cgroup v1, or eBPF didn't provide cgroup_id), fall back to reading
    ///    /host/proc/<tgid>/cgroup.
    pub async fn enrich(&mut self, cache: &Arc<RwLock<ServiceCache>>) {
        if let Some(cgid) = self.cgroup_id
            && cgid != 0
            && detect_cgroup_v2()
            && self.try_resolve_from_cgroup_id(cgid, cache).await
        {
            return;
        } else {
            debug!("cgroup_v2 not detected. Using cgroup v1");
            self.try_resolve_docker();
        }
        self.try_resolve_k8s(cache).await;
    }

    /// Resolve pod/container from a kernel cgroup_id by walking /sys/fs/cgroup (v2).
    /// Returns true if resolution succeeded (pod_uid extracted), false to allow caller fallback.
    async fn try_resolve_from_cgroup_id(
        &mut self,
        cgroup_id: u64,
        cache: &Arc<RwLock<ServiceCache>>,
    ) -> bool {
        let Some(cgroup_path) = find_cgroup_path_by_id(cgroup_id) else {
            debug!(
                "cgroup_id {} not found under {}",
                cgroup_id, "/host/sys/fs/cgroup"
            );
            return false;
        };

        if let Some(id) = extract_pod_uid(cgroup_path.to_string_lossy().to_string()) {
            self.container_id = Some(id.clone());
            self.runtime = ContainerRuntime::Kubernetes;

            // get pod from cache
            // acquire cache lock
            let cache_lock = cache.read().await;
            match cache_lock.get_from_cache(&id).await {
                Some(name) => self.pod_name = Some(name),
                None => {
                    // fallback to the container_id if the k8s API cannot resolve the name
                    self.pod_name = Some(id.clone());
                }
            }
            return true;
        }

        // not a k8s pod cgroup — try docker/containerd/crio container id
        if let Some(id) = extract_container_id_from_path(&cgroup_path.to_string_lossy()) {
            self.container_id = Some(id.clone());
            self.runtime = ContainerRuntime::Docker;
            match resolve_docker_name(&id) {
                Some(name) => self.container_name = Some(name),
                None => self.container_name = self.container_id.clone(),
            }
            return true;
        }

        false
    }

    /// Docker resolution via local filesystem.
    /// This part is triggered when the container is already detected
    // TODO: this is working for Linux, can anyone check if this works on macOs systems ?
    fn try_resolve_docker(&mut self) {
        let Some(tgid) = self.tgid else { return };

        // Step 1: read the cgroup path from procfs
        if let Some(cgroup_info) = get_cgroup_info(tgid) {
            // Extract the actual path from the cgroup file (format: hierarchy:id:path)
            let cgroup_path = cgroup_info
                .lines()
                .filter_map(|line| line.split(':').nth(2))
                .next()
                .unwrap_or("");

            if cgroup_path.is_empty() {
                info!("cgroup_path is empty");
                return;
            }

            // Step 2: extract container ID from the path
            if let Some(id) = extract_container_id_from_path(cgroup_path) {
                self.container_id = Some(id.clone());
                self.runtime = ContainerRuntime::Docker;

                // Step 3: resolve container name from Docker metadata JSON
                match resolve_docker_name(&id) {
                    Some(name) => self.container_name = Some(name),
                    None => {
                        self.container_name = self.container_id.clone(); // fallback to the container_id if the system cannot resolve the name after the 2 steps
                    }
                }
            }
        };
    }

    async fn try_resolve_k8s(&mut self, cache: &Arc<RwLock<ServiceCache>>) {
        let Some(tgid) = self.tgid else { return };

        // Step 1: read the cgroup path from procfs
        if let Some(cgroup_info) = get_cgroup_info(tgid) {
            // Extract the actual path from the cgroup file (format: hierarchy:id:path)
            let cgroup_path = cgroup_info
                .lines()
                .filter_map(|line| line.split(':').nth(2))
                .next()
                .unwrap_or("");

            if cgroup_path.is_empty() {
                info!("cgroup_path is empty");
                return;
            }

            // Step 2: extract container ID from the path
            if let Some(id) = extract_pod_uid(cgroup_path.to_string()) {
                self.container_id = Some(id.clone());
                self.runtime = ContainerRuntime::Kubernetes;

                // Step 3: resolve container name from the k8s API

                //acquire cache lock
                let cache_lock = cache.read().await;

                match cache_lock.get_from_cache(&id).await {
                    Some(name) => self.pod_name = Some(name),
                    None => {
                        // fallback to the container_id if the k8s API cannot resolve the name
                        self.pod_name = Some(id.clone());
                    }
                }
            }
        }
    }
}

/// Helpers

/// Helper function to extract the container ID from a cgroup path, supporting multiple prefixes.
/// Supports Docker, Containerd, CRI-O, Docker cgroup V1.
/// Used in try_resolve_docker function
fn extract_container_id_from_path(cgroup_path: &str) -> Option<String> {
    let parts: Vec<&str> = cgroup_path.split('/').collect();

    for part in &parts {
        // Docker with systemd (docker-<id>.scope)
        if let Some(id) = part.strip_prefix("docker-") {
            return Some(id.strip_suffix(".scope").unwrap_or(id).to_string());
        }
        // containerd (cri-containerd-<id>.scope)
        if let Some(id) = part.strip_prefix("cri-containerd-") {
            return Some(id.strip_suffix(".scope").unwrap_or(id).to_string());
        }
        // CRI-O (crio-<id>.scope)
        if let Some(id) = part.strip_prefix("crio-") {
            return Some(id.strip_suffix(".scope").unwrap_or(id).to_string());
        }
        // Docker cgroup v1 plain (/docker/<id>)
        if *part == "docker" {
            if let Some(next) =
                parts.get(parts.iter().position(|p| *p == "docker").unwrap_or(0) + 1)
            {
                return Some(next.to_string());
            }
        }
    }

    None
}

/// Resolve a Docker container name by reading config.v2.json.
fn resolve_docker_name(container_id: &str) -> Option<String> {
    // TODO: Does this work on macOs?
    let path = format!("/var/lib/docker/containers/{}/config.v2.json", container_id);
    let json_str = fs::read_to_string(&path).ok()?;
    let parsed: serde_json::Value = serde_json::from_str(&json_str).ok()?;
    let image_name = parsed
        .get("Config")?
        .get("Image")?
        .as_str()?
        .trim_start_matches('/');
    if image_name.is_empty() {
        return None; // if images is empty returns None
    }

    Some(image_name.to_string())
}

/// Helper function to extract the pod name, called 'target' from a vector of splits ['','','']
fn extract_target_from_splits(splits: Vec<&str>, target: &str) -> Result<usize, Error> {
    for (index, split) in splits.iter().enumerate() {
        // find the split that contains the word 'pod'
        if split.contains(target) {
            debug!("Target index; {}", index);
            return Ok(index);
        }
    }
    Err(Error::msg("'-pod' word not found in split"))
}

fn extract_pod_uid(cgroup_path: String) -> Option<String> {
    // example of cgroup path:
    // /sys/fs/cgroup/kubelet.slice/kubelet-kubepods.slice/kubelet-kubepods-besteffort.slice/kubelet-kubepods-besteffort-pod93580201_87d5_44e6_9779_f6153ca17637.slice
    // or
    // /sys/fs/cgroup/kubelet.slice/kubelet-kubepods.slice/kubelet-kubepods-burstable.slice/kubelet-kubepods-burstable-poddd3a1c6b_af40_41b1_8e1c_9e31fe8d96cb.slice

    // split the path by "/"
    let splits: Vec<&str> = cgroup_path.split("/").collect();
    debug!("Debugging splits: {:?}", &splits);

    let index = extract_target_from_splits(splits.clone(), "-pod").ok();

    match index {
        Some(idx) => {
            let pod_split = splits[idx]
                .trim_start_matches("kubelet-kubepods-besteffort-")
                .trim_start_matches("kubelet-kubepods-burstable-")
                .trim_start_matches("kubepods-besteffort-")
                .trim_start_matches("kubepods-burstable-");

            let uid_ = pod_split
                .trim_start_matches("pod")
                .trim_end_matches(".slice"); //return uids with underscore (_) [ex.dd3a1c6b_af40_41b1_8e1c_9e31fe8d96cb]

            let uid = Some(uid_.replace("_", "-"));
            uid
        }
        None => {
            debug!("Index returned a value of None");
            None
        }
    }
}

/// Detect if the host is running cgroup v2.
/// On cgroup v2 the file /sys/fs/cgroup/cgroup.controllers exists; on v1 it
/// does not (the controllers are split across /sys/fs/cgroup/<controller>).
/// this is a way to detect if we can attach the try_resolve_k8s functions
fn detect_cgroup_v2() -> bool {
    Path::new("/host/sys/fs/cgroup")
        .join("cgroup.controllers")
        .is_file()
}

/// Scans /sys/fs/cgroup recursively and return the path of the directory that
/// matches cgroup_id. This lookup is guaranteed because of the structure of the cgroup v2 in linux
fn find_cgroup_path_by_id(cgroup_id: u64) -> Option<PathBuf> {
    let root = Path::new("/host/sys/fs/cgroup");
    let mut stack = vec![root.to_path_buf()];

    while let Some(dir) = stack.pop() {
        let meta = match fs::metadata(&dir) {
            Ok(m) => m,
            Err(_) => continue,
        };

        if meta.ino() == cgroup_id {
            return Some(dir);
        }

        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                }
            }
        }
    }

    None
}

fn get_cgroup_info(tgid: u32) -> Option<String> {
    let cgroup_info = match fs::read_to_string(format!("/proc/{}/cgroup", tgid)) {
        Ok(s) => return Some(s),
        Err(e) => {
            debug!("Cannot read /proc/{}/cgroup: {}", tgid, e);
            return None;
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_container_id_docker_systemd() {
        let path = "/sys/fs/cgroup/system.slice/docker-13abd64c0ba349975a762476c9703b642d18077eabeb3aa1d941132048afc861.scope";
        assert_eq!(
            extract_container_id_from_path(path),
            Some("13abd64c0ba349975a762476c9703b642d18077eabeb3aa1d941132048afc861".to_string())
        );
    }

    #[test]
    fn test_extract_container_id_containerd() {
        let path = "/sys/fs/cgroup/kubepods.slice/kubepods-besteffort.slice/kubepods-besteffort-podb8701d38_3791_422d_ad15_890ad1a0844b.slice/cri-containerd-abc123.scope";
        assert_eq!(
            extract_container_id_from_path(path),
            Some("abc123".to_string())
        );
    }

    #[test]
    fn test_extract_container_id_docker_v1() {
        let path = "/docker/13abd64c0ba349975a762476c9703b642d18077eabeb3aa1d941132048afc861";
        assert_eq!(
            extract_container_id_from_path(path),
            Some("13abd64c0ba349975a762476c9703b642d18077eabeb3aa1d941132048afc861".to_string())
        );
    }

    #[test]
    fn test_extract_container_id_not_found() {
        let path = "/sys/fs/cgroup/system.slice/systemd-journald.service";
        assert_eq!(extract_container_id_from_path(path), None);
    }
    #[test]
    fn resolve_docker_name_test() {
        use std::process::Command;
        // create a docker container from a testing image
        let create_test_container_command = Command::new("/usr/bin/docker")
            .args(["run", "--rm", "-d", "busybox:latest"])
            .output()
            .expect("Cannot create test container");
        let output = create_test_container_command.stdout;
        let container_id = String::from_utf8(output)
            .expect("Cannot extract container id")
            .trim()
            .to_string();
        println!("{}", &container_id);
        let docker_container_name = match resolve_docker_name(&container_id) {
            Some(name) => Some(name),
            None => None,
        };

        assert_eq!(docker_container_name, Some("busybox:latest".to_string()))
    }
    // TODO: missing tests for extract_pod_uid, extract_target_from_splits, try_resolve_docker, try_resolve_k8s, enrich.
}
