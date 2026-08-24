use std::collections::HashMap;
use std::fs;
use anyhow::Error;
use k8s_openapi::api::core::v1::Pod;
use kube::api::ObjectList;
use kube::{Api, Client};
use tracing::debug;


/// Detected container runtime.
#[derive(Debug, Clone, PartialEq)]
pub enum ContainerRuntime {
    Docker,
    Kubernetes,
    Unknown,
}

/// Process / container metadata enriched by service discovery.
///
/// Built from raw eBPF data and enriched with a cgroup -> Docker -> K8s lookup.
#[derive(Debug, Clone)]
pub struct Metadata {
    pub tgid: Option<u32>,
    pub command: String,
    pub runtime: ContainerRuntime,
    pub container_name: Option<String>,
    pub container_id: Option<String>,
    pub pod_name: Option<String>,
    pub namespace: Option<String>,
}

impl Metadata {
    /// Build base metadata from eBPF data.
    pub fn from_ebpf(tgid: Option<u32>, command: &[u8]) -> Self {
        let command = String::from_utf8_lossy(command)
            .trim_end_matches('\0')
            .to_string();
        Self {
            tgid,
            command,
            runtime: ContainerRuntime::Unknown,
            container_name: None,
            container_id: None,
            pod_name: None,
            namespace: None,
        }
    }

    /// Lookup rules: first Docker (filesystem), then Kubernetes (API).
    ///
    /// 1. Reads `/proc/<tgid>/cgroup`.
    /// 2. Extracts the container ID from the cgroup path.
    /// 3. Tries to resolve the container name from `/var/lib/docker/containers/<id>/config.v2.json`.
    /// 4. If Docker is not found, attempts K8s lookup.
    pub fn enrich(&mut self) {
        self.try_resolve_docker();
        self.try_resolve_k8s();
    }

    /// Docker resolution via local filesystem.
    /// This part is triggered when the container is already detected
    // TODO: this is working for Linux, can anyone check if this works on macOs systems ?
    fn try_resolve_docker(&mut self) {
        let Some(tgid) = self.tgid else { return };

        // Step 1: read the cgroup path from procfs
        let cgroup_info = match fs::read_to_string(format!("/proc/{}/cgroup", tgid)) {
            Ok(s) => s,
            Err(e) => {
                tracing::debug!("Cannot read /proc/{}/cgroup: {}", tgid, e);
                return;
            }
        };

        // Extract the actual path from the cgroup file (format: hierarchy:id:path)
        let cgroup_path = cgroup_info
            .lines()
            .filter_map(|line| line.split(':').nth(2))
            .next()
            .unwrap_or("");

        if cgroup_path.is_empty() {
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
                    self.container_name =
                        Some(self.container_id.clone()).expect("Cannot resolve container name")
                } // fallback to the container_id if the system cannot resolve the name after the 2 steps
            }
        }
    }

    fn try_resolve_k8s(&mut self){
        let Some(tgid) = self.tgid else { return };

        // Step 1: read the cgroup path from procfs
        let cgroup_info = match fs::read_to_string(format!("/proc/{}/cgroup", tgid)) {
            Ok(s) => s,
            Err(e) => {
                tracing::debug!("Cannot read /proc/{}/cgroup: {}", tgid, e);
                return;
            }
        };

        // Extract the actual path from the cgroup file (format: hierarchy:id:path)
        let cgroup_path = cgroup_info
            .lines()
            .filter_map(|line| line.split(':').nth(2))
            .next()
            .unwrap_or("");

        if cgroup_path.is_empty() {
            return;
        }

        // Step 2: extract container ID from the path
        if let Some(id) = extract_container_id_from_path(cgroup_path) {
            self.container_id = Some(id.clone());
            self.runtime = ContainerRuntime::Kubernetes;

            // Step 3: resolve container name from Docker metadata JSON
            match resolve_k8s_name(&id) {
                Some(name) => self.container_name = Some(name),
                None => {
                    self.container_name =
                        Some(self.container_id.clone()).expect("Cannot resolve container name from k8s api ")
                } // fallback to the container_id if the system cannot resolve the name after the 2 steps
            }
        }

        //Step 3: call the k8s API to solve the container nam e
    }

    // TODO: dead code 
    // Enrichment from Kubernetes (for external use, e.g. identity service).
    //pub fn enrich_from_k8s(&mut self, pod_name: impl Into<String>, namespace: impl Into<String>) {
    //    self.runtime = ContainerRuntime::Kubernetes;
    //    self.pod_name = Some(pod_name.into());
    //    self.namespace = Some(namespace.into());
    //}
}

/// Extract the container ID from a cgroup path, supporting multiple prefixes.
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

/// Resolve a Docker container name by reading `config.v2.json`.
///
// TODO: Does this work on macOs?
fn resolve_docker_name(container_id: &str) -> Option<String> {
    let path = format!("/var/lib/docker/containers/{}/config.v2.json", container_id);
    let json_str = fs::read_to_string(&path).ok()?;
    let parsed: serde_json::Value = serde_json::from_str(&json_str).ok()?;
    let mut image_name = parsed
        .get("Config")?
        .get("Image")?
        .as_str()?
        .trim_start_matches('/');
    if image_name.is_empty() {
        image_name = parsed
            .get("Config")?
            .get("WorkingDir")?
            .as_str()?
            .trim_start_matches('/'); // fallback if the image_name is empty
    }

    Some(image_name.to_string())
}


fn resolve_k8s_name(container_id: &str) -> Option<String> {
    // pass a container_id and returns the object meta infos:
    /* pub struct ObjectMeta {

        pub annotations: Option<BTreeMap<String, String>>,
        pub creation_timestamp: Option<Time>,
        pub deletion_grace_period_seconds: Option<i64>,
        pub deletion_timestamp: Option<Time>,
        pub finalizers: Option<Vec<String>>,
        pub generate_name: Option<String>,
        pub generation: Option<i64>,
        pub labels: Option<BTreeMap<String, String>>,
        pub managed_fields: Option<Vec<ManagedFieldsEntry>>,
        pub name: Option<String>,
        pub namespace: Option<String>,
        pub owner_references: Option<Vec<OwnerReference>>,
        pub resource_version: Option<String>,
        pub self_link: Option<String>,
        pub uid: Option<String>,
    } */

    todo!()
}


async fn query_all_pods() -> Result<ObjectList<Pod>, Error> {
    let client = Client::try_default()
        .await
        .expect("Cannot connect to kubernetes client");
    let pods: Api<Pod> = Api::all(client);
    let lp = kube::api::ListParams::default(); // default list params
    let pod_list = pods
        .list(&lp)
        .await
        .expect("An error occured during the pod list extraction");

    Ok(pod_list)
}

async fn get_pod_info() -> Result<HashMap<String, String>, Error> {
    let all_pods = query_all_pods().await?;

    let mut service_map = HashMap::<String, String>::new();

    for pod in all_pods {
        if let (Some(name), Some(uid)) = (pod.metadata.name, pod.metadata.uid) {
            service_map.insert(uid, name);
        }
    } // insert the pod name and uid from the KubeAPI

    Ok(service_map)
}

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


fn extract_pod_uid(cgroup_path: String) -> Result<String, Error> {
    // example of cgroup path:
    // /sys/fs/cgroup/kubelet.slice/kubelet-kubepods.slice/kubelet-kubepods-besteffort.slice/kubelet-kubepods-besteffort-pod93580201_87d5_44e6_9779_f6153ca17637.slice
    // or
    // /sys/fs/cgroup/kubelet.slice/kubelet-kubepods.slice/kubelet-kubepods-burstable.slice/kubelet-kubepods-burstable-poddd3a1c6b_af40_41b1_8e1c_9e31fe8d96cb.slice

    // split the path by "/"
    let splits: Vec<&str> = cgroup_path.split("/").collect();
    debug!("Debugging splits: {:?}", &splits);

    let index = extract_target_from_splits(splits.clone(), "-pod")?;

    let pod_split = splits[index]
        .trim_start_matches("kubelet-kubepods-besteffort-")
        .trim_start_matches("kubelet-kubepods-burstable-")
        .trim_start_matches("kubepods-besteffort-")
        .trim_start_matches("kubepods-burstable-");

    let uid_ = pod_split
        .trim_start_matches("pod")
        .trim_end_matches(".slice"); //return uids with underscore (_) [ex.dd3a1c6b_af40_41b1_8e1c_9e31fe8d96cb]

    let uid = uid_.replace("_", "-");
    Ok(uid.to_string())
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
}
