use kube::Client;
use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::service_cache::ServiceCache;
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
    pub namespace: Option<String>,
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
            && self.try_resolve_from_cgroup_id(cgid, cache).await
        {
            return;
        } else {
            debug!("An error occured during cgroup_id resolution.Trying with docker and tgid");
        }
        if let Some(tgid) = self.tgid
            && self.try_resolve_docker(tgid, cache).await
        {
            return;
        } else {
            debug!("Cannot resolve tgid with Docker");
        }
    }

    /// Resolve pod/container from a kernel cgroup_id by walking /sys/fs/cgroup (v2).
    /// Returns true if resolution succeeded (pod_uid extracted), false to allow caller fallback.
    ///
    /// only for k8s
    async fn try_resolve_from_cgroup_id(
        &mut self,
        cgroup_id: u64,
        cache: &Arc<RwLock<ServiceCache>>,
    ) -> bool {
        let Some(cgroup_path) = cached_cgroup_path_by_id(cgroup_id) else {
            debug!(
                "cgroup_id {} not found under {}",
                cgroup_id, CGROUP_ROOT
            );
            return false;
        };

        if let Some(id) = extract_pod_uid(cgroup_path.to_string_lossy().to_string())
            && kubernetes_available().await
        {
            self.container_id = Some(id.clone());
            debug!(
                "Resolving container id {:?} using the pod UID",
                self.container_id
            );
            self.runtime = ContainerRuntime::Kubernetes;

            // get pod from cache
            // acquire cache lock
            let cache_lock = cache.read().await;
            match cache_lock.get_from_cache(&id).await {
                Some(cached_data) => {
                    self.pod_name = Some(cached_data[0].clone());
                    self.namespace = Some(cached_data[1].clone());
                    return true;
                }
                None => {
                    return false;
                }
            }
        }
        false
    }

    /// Docker resolution via local filesystem.
    /// This part is triggered when the container is already detected
    // TODO: this is working for Linux, can anyone check if this works on macOs systems ?
    pub async fn try_resolve_docker(
        &mut self,
        tgid: u32,
        cache: &Arc<RwLock<ServiceCache>>,
    ) -> bool {
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
                return false;
            }

            // Step 2: extract container ID from the path
            if let Some(id) = extract_container_id_from_path(&cgroup_path.to_string()) {
                self.container_id = Some(id.clone());
                debug!(
                    "Resolving container id {:?} using the docker fs",
                    self.container_id
                );
                self.runtime = ContainerRuntime::Docker;

                let cache_lock = cache.read().await;
                match cache_lock.get_from_cache(&id).await {
                    Some(cached_data) => {
                        self.container_name = Some(cached_data[0].clone());
                        self.namespace = Some(cached_data[1].clone());
                        return true;
                    }
                    None => match resolve_docker_name(&id) {
                        Some(name) => {
                            self.container_name = Some(name);
                            self.namespace = None;
                            return true;
                        }
                        None => return false,
                    },
                }
            }
        };
        false
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

fn extract_pod_uid(cgroup_path: String) -> Option<String> {
    // Pod cgroups come in two shapes, depending on the kubelet cgroup driver:
    //
    // systemd driver (one slice per pod; Guaranteed pods have no QoS level):
    //   /kubepods.slice/kubepods-besteffort.slice/kubepods-besteffort-pod93580201_87d5_44e6_9779_f6153ca17637.slice
    //   /kubepods.slice/kubepods-burstable.slice/kubepods-burstable-poddd3a1c6b_af40_41b1_8e1c_9e31fe8d96cb.slice
    //   /kubepods.slice/kubepods-pod93580201_87d5_44e6_9779_f6153ca17637.slice
    //   (kind/minikube nest the same names under kubelet.slice/kubelet-kubepods.slice)
    //
    // cgroupfs driver:
    //   /kubepods/besteffort/pod93580201-87d5-44e6-9779-f6153ca17637/<container id>
    //   /kubepods/pod93580201-87d5-44e6-9779-f6153ca17637/<container id>
    //
    // Static pods use the 32-hex-digit config hash instead of a UID; ServiceCache keys
    // them by that hash, so it is returned as is.
    for segment in cgroup_path.split('/') {
        let candidate = if let Some(slice) = segment.strip_suffix(".slice") {
            match slice.rfind("-pod") {
                Some(idx) => &slice[idx + "-pod".len()..],
                None => continue,
            }
        } else if let Some(rest) = segment.strip_prefix("pod") {
            rest
        } else {
            continue;
        };

        let uid = candidate.replace('_', "-");
        if is_pod_uid(&uid) {
            debug!("Extracted pod uid {} from cgroup path {}", uid, cgroup_path);
            return Some(uid);
        }
    }
    debug!("No pod uid in cgroup path {}", cgroup_path);
    None
}

/// A pod UID (8-4-4-4-12 hex digits) or a static pod config hash (32 hex digits).
fn is_pod_uid(candidate: &str) -> bool {
    let is_hex = |part: &str| !part.is_empty() && part.chars().all(|c| c.is_ascii_hexdigit());
    let groups: Vec<&str> = candidate.split('-').collect();
    match groups.as_slice() {
        [hash] => hash.len() == 32 && is_hex(hash),
        [a, b, c, d, e] => {
            [a.len(), b.len(), c.len(), d.len(), e.len()] == [8, 4, 4, 4, 12]
                && groups.iter().all(|g| is_hex(g))
        }
        _ => false,
    }
}

/// Where the host cgroup hierarchy is mounted inside the agent container.
const CGROUP_ROOT: &str = "/host/sys/fs/cgroup";

/// Maximum number of memoized cgroup_id -> path lookups before the memo is reset.
const CGROUP_PATH_MEMO_CAPACITY: usize = 16_384;

/// How long a failed cgroup_id lookup is remembered before the hierarchy is walked again.
const CGROUP_PATH_MISS_TTL: Duration = Duration::from_secs(30);

/// Memoized [`find_cgroup_path_by_id`].
///
/// The lookup walks the whole cgroup hierarchy (one `stat` per cgroup) and used to run
/// for every eBPF event. Hits are kept until the memo is full; misses are retried
/// after [`CGROUP_PATH_MISS_TTL`] so cgroups created later are still found.
fn cached_cgroup_path_by_id(cgroup_id: u64) -> Option<PathBuf> {
    static MEMO: OnceLock<Mutex<HashMap<u64, (Option<PathBuf>, Instant)>>> = OnceLock::new();
    let memo = MEMO.get_or_init(|| Mutex::new(HashMap::new()));

    if let Ok(entries) = memo.lock()
        && let Some((path, looked_up_at)) = entries.get(&cgroup_id)
        && (path.is_some() || looked_up_at.elapsed() < CGROUP_PATH_MISS_TTL)
    {
        return path.clone();
    }

    // walk without holding the lock
    let path = find_cgroup_path_by_id(cgroup_id);

    if let Ok(mut entries) = memo.lock() {
        if entries.len() >= CGROUP_PATH_MEMO_CAPACITY {
            entries.clear();
        }
        entries.insert(cgroup_id, (path.clone(), Instant::now()));
    }
    path
}

/// Whether a Kubernetes client configuration is available.
///
/// Building a `kube::Client` reads the kubeconfig or the in-cluster service account
/// and sets up TLS; it used to happen once per eBPF event. The answer does not change
/// while the agent runs, so it is computed once.
async fn kubernetes_available() -> bool {
    static AVAILABLE: tokio::sync::OnceCell<bool> = tokio::sync::OnceCell::const_new();
    *AVAILABLE
        .get_or_init(|| async { Client::try_default().await.is_ok() })
        .await
}

/// Scans /sys/fs/cgroup recursively and return the path of the directory that
/// matches cgroup_id. This lookup is guaranteed because of the structure of the cgroup v2 in linux
fn find_cgroup_path_by_id(cgroup_id: u64) -> Option<PathBuf> {
    find_cgroup_path_by_id_in(Path::new(CGROUP_ROOT), cgroup_id)
}

fn find_cgroup_path_by_id_in(root: &Path, cgroup_id: u64) -> Option<PathBuf> {
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
    fs::read_to_string(format!("/proc/{}/cgroup", tgid))
        .map_err(|e| debug!("Cannot read /proc/{}/cgroup: {}", tgid, e))
        .ok()
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
    #[ignore = "needs a running Docker daemon and read access to /var/lib/docker"]
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
    #[test]
    fn extract_pod_uid_systemd_qos_slices() {
        for (path, uid) in [
            (
                "/sys/fs/cgroup/kubelet.slice/kubelet-kubepods.slice/kubelet-kubepods-besteffort.slice/kubelet-kubepods-besteffort-pod93580201_87d5_44e6_9779_f6153ca17637.slice",
                "93580201-87d5-44e6-9779-f6153ca17637",
            ),
            (
                "/sys/fs/cgroup/kubelet.slice/kubelet-kubepods.slice/kubelet-kubepods-burstable.slice/kubelet-kubepods-burstable-poddd3a1c6b_af40_41b1_8e1c_9e31fe8d96cb.slice",
                "dd3a1c6b-af40-41b1-8e1c-9e31fe8d96cb",
            ),
            (
                "/host/sys/fs/cgroup/kubepods.slice/kubepods-besteffort.slice/kubepods-besteffort-podb8701d38_3791_422d_ad15_890ad1a0844b.slice/cri-containerd-abc123.scope",
                "b8701d38-3791-422d-ad15-890ad1a0844b",
            ),
        ] {
            assert_eq!(extract_pod_uid(path.to_string()).as_deref(), Some(uid), "{path}");
        }
    }

    // Guaranteed pods have no QoS slice: kubepods-pod<uid>.slice sits directly under kubepods.slice.
    #[test]
    fn extract_pod_uid_guaranteed_pod() {
        assert_eq!(
            extract_pod_uid(
                "/host/sys/fs/cgroup/kubepods.slice/kubepods-pod93580201_87d5_44e6_9779_f6153ca17637.slice/cri-containerd-abc.scope".to_string()
            )
            .as_deref(),
            Some("93580201-87d5-44e6-9779-f6153ca17637")
        );
        assert_eq!(
            extract_pod_uid(
                "/sys/fs/cgroup/kubelet.slice/kubelet-kubepods.slice/kubelet-kubepods-pod93580201_87d5_44e6_9779_f6153ca17637.slice".to_string()
            )
            .as_deref(),
            Some("93580201-87d5-44e6-9779-f6153ca17637")
        );
    }

    #[test]
    fn extract_pod_uid_cgroupfs_driver() {
        assert_eq!(
            extract_pod_uid(
                "/host/sys/fs/cgroup/kubepods/besteffort/pod93580201-87d5-44e6-9779-f6153ca17637/0123abcd".to_string()
            )
            .as_deref(),
            Some("93580201-87d5-44e6-9779-f6153ca17637")
        );
        assert_eq!(
            extract_pod_uid(
                "/host/sys/fs/cgroup/kubepods/pod93580201-87d5-44e6-9779-f6153ca17637".to_string()
            )
            .as_deref(),
            Some("93580201-87d5-44e6-9779-f6153ca17637")
        );
    }

    // ServiceCache keys static pods by their kubernetes.io/config.hash.
    #[test]
    fn extract_pod_uid_static_pod_config_hash() {
        assert_eq!(
            extract_pod_uid(
                "/host/sys/fs/cgroup/kubepods.slice/kubepods-burstable.slice/kubepods-burstable-pod2d6c8d3e9b0f4a1c8e7d6c5b4a3f2e1d.slice".to_string()
            )
            .as_deref(),
            Some("2d6c8d3e9b0f4a1c8e7d6c5b4a3f2e1d")
        );
    }

    #[test]
    fn extract_pod_uid_rejects_non_pod_paths() {
        for path in [
            "/sys/fs/cgroup/system.slice/docker-13abd64c0ba349975a762476c9703b642d18077eabeb3aa1d941132048afc861.scope",
            "/sys/fs/cgroup/kubepods.slice/kubepods-besteffort.slice",
            "/sys/fs/cgroup/system.slice/podman-1234.scope",
            "/sys/fs/cgroup/user.slice/user-1000.slice/user@1000.service/app.slice/pod-manager.slice",
            "/sys/fs/cgroup/podcast/pod123",
            "",
            "/",
        ] {
            assert_eq!(extract_pod_uid(path.to_string()), None, "{path}");
        }
    }

    #[test]
    fn find_cgroup_path_by_id_walks_nested_directories() {
        let root = std::env::temp_dir().join(format!("cortexbrain-cgroup-test-{}", std::process::id()));
        let target = root.join("kubepods.slice/kubepods-pod93580201_87d5_44e6_9779_f6153ca17637.slice/cri-containerd-abc.scope");
        fs::create_dir_all(&target).unwrap();
        fs::create_dir_all(root.join("system.slice/docker-1.scope")).unwrap();
        let ino = fs::metadata(&target).unwrap().ino();

        assert_eq!(find_cgroup_path_by_id_in(&root, ino), Some(target.clone()));
        assert_eq!(
            extract_pod_uid(target.to_string_lossy().to_string()).as_deref(),
            Some("93580201-87d5-44e6-9779-f6153ca17637")
        );
        assert_eq!(find_cgroup_path_by_id_in(&root, u64::MAX), None);
        assert_eq!(find_cgroup_path_by_id_in(&root.join("missing"), ino), None);

        fs::remove_dir_all(&root).unwrap();
    }
}
