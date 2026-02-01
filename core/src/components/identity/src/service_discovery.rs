#[cfg(feature = "experimental")]
use anyhow::Error;
#[cfg(feature = "experimental")]
use k8s_openapi::api::core::v1::Pod;
#[cfg(feature = "experimental")]
use kube::api::ObjectList;
#[cfg(feature = "experimental")]
use kube::{Api, Client};
#[cfg(feature = "experimental")]
use std::fs;
#[cfg(feature = "experimental")]
use tokio::time;

#[cfg(feature = "experimental")]
pub async fn scan_cgroup_paths(path: String) -> Result<Vec<String>, Error> {
    let mut cgroup_paths: Vec<String> = Vec::new();
    let default_path = "/sys/fs/cgroup/kubepods.slice".to_string();

    let target_path = if fs::metadata(&path).is_err() {
        error!("Using default path: {}", &default_path);
        default_path
    } else {
        path
    };
    let entries = match fs::read_dir(&target_path) {
        Ok(entries) => entries,
        Err(e) => {
            error!(
                "Error reading cgroup directory {:?}: {}",
                &target_path.clone(),
                e
            );
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
struct ServiceIdentity {
    uid: String,
    container_id: String,
}

#[cfg(feature = "experimental")]
pub async fn scan_cgroup_cronjob(time_delta: u64) -> Result<(), Error> {
    let interval = std::time::Duration::from_secs(time_delta);
    loop {
        let scanned_paths = scan_cgroup_paths("/sys/fs/cgroup/kubelet.slice".to_string())
            .await
            .expect("An error occured during the cgroup scan");
        //--> this should return :
        //  /sys/fs/cgroup/kubelet.slice/kubelet-kubepods.slice
        //  /sys/fs/cgroup/kubelet.slice/kubelet.service
        let mut scanned_subpaths = Vec::<String>::new();
        for path in scanned_paths {
            //info!("Scanned cgroup path: {}", path);
            // scan the subgroups
            let subpaths = scan_cgroup_paths(path.to_string()).await;
            match subpaths {
                Ok(paths) => {
                    for subpath in paths {
                        scanned_subpaths.push(subpath);
                    }
                    // ---> this should return the cgroups files and also :
                    // kubelet-kubepods-burstable.slice
                    // kubelet-kubepods-besteffort.slice

                    // this directories needs to be scanned again to get further information about the pods
                    // for example:
                    // kubelet-kubepods-besteffort-pod088f8704_24f0_4636_a8e2_13f75646f370.slice
                    // where pod088f8704_24f0_4636_a8e2_13f75646f370 is the pod UID
                }
                Err(e) => {
                    error!("An error occured during the cgroup subpath scan: {}", e);
                    continue;
                }
            }
        }

        let mut scanned_subpaths_v2 = Vec::<String>::new();
        // second cgroup scan level to get the pod UIDs
        for scanned_subpath in &scanned_subpaths {
            let subpaths_v2 = scan_cgroup_paths(scanned_subpath.to_string()).await;
            match subpaths_v2 {
                Ok(paths) => {
                    for sub2 in paths {
                        info!("Debugging sub2: {}", &sub2); //return e.g. /sys/fs/cgroup/kubepods.slice/kubepods-besteffort.slice/kubepods-besteffort-podb8701d38_3791_422d_ad15_890ad1a0844b.slice/docker-f2e265659293676231ecb38fafccc97b1a42b75be192c32a602bc8ea579dc866.scope
                        scanned_subpaths_v2.push(sub2);
                        // this contains the addressed like this
                        //kubelet-kubepods-besteffort-pod088f8704_24f0_4636_a8e2_13f75646f370.slice
                    }
                }
                Err(e) => {
                    error!("An error occured during the cgroup subpath v2 scan: {}", e);
                    continue;
                }
            }
        }

        let mut uids = Vec::<String>::new();
        let mut identites = Vec::<ServiceIdentity>::new();

        //read the subpaths to extract the pod uid
        for subpath in scanned_subpaths_v2 {
            let uid = extract_pod_uid(subpath.clone())
                .expect("An error occured during the extraction of pod UIDs");
            let container_id = extract_container_id(subpath.clone())
                .expect("An error occured during the extraction of the docker container id");
            debug!("Debugging extracted UID: {:?}", &uid);
            // create a linked list for each service
            let service_identity = ServiceIdentity { uid, container_id };
            identites.push(service_identity); //push the linked list in a vector of ServiceIdentity structure. Each struct contains the uid and the container id
        }

        // get pod information from UID and store the info in an HashMqp for O(1) access
        let service_map = get_pod_info().await?;

        //info!("Debugging Identites vector: {:?}", identites);
        for service in identites {
            let name = service_cache(service_map.clone(), service.uid.clone());
            let uid = service.uid;
            let id = service.container_id;
            info!(
                "[Identity]: name: {:?} uid: {:?} docker container id {:?} ",
                name, uid, id
            );
        }

        info!(
            "Cronjob completed a cgroup scan cycle. Next scan will be in {} seconds",
            time_delta
        );
        time::sleep(interval).await;
    }
}
#[cfg(feature = "experimental")]
fn service_cache(service_map: HashMap<String, String>, uid: String) -> String {
    service_map.get(&uid).cloned().unwrap_or_else(|| {
        error!("Service not found for uid: {}", uid);
        "unknown".to_string()
    })
}
#[cfg(feature = "experimental")]
fn extract_container_id(cgroup_path: String) -> Result<String, Error> {
    let splits: Vec<&str> = cgroup_path.split("/").collect();

    let index = extract_target_from_splits(splits.clone(), "docker-")?;
    let docker_id_split = splits[index]
        .trim_start_matches("docker-")
        .trim_end_matches(".scope");
    Ok(docker_id_split.to_string())
}

// IDEA: add cgroup docker process mapping in ServiceIdentity structure
#[cfg(feature = "experimental")]
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
#[cfg(feature = "experimental")]
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

/* unfortunately you cannot query the pods using the uids directly from ListParams */
#[cfg(feature = "experimental")]
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

// fast pod caching system
#[cfg(feature = "experimental")]
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

#[cfg(feature = "experimental")]
mod tests {
    use tracing_subscriber::fmt::format;

    use crate::helpers::{extract_container_id, extract_pod_uid, extract_target_from_splits};

    #[test]
    fn extract_uid_from_string() {
        let cgroup_paths = vec!["/sys/fs/cgroup/kubepods.slice/kubepods-besteffort.slice/kubepods-besteffort-pod231bd2d7_0f09_4781_a4e1_e4ea026342dd.slice".to_string(),
                                             "/sys/fs/cgroup/kubelet.slice/kubelet-kubepods.slice/kubelet-kubepods-besteffort.slice/kubelet-kubepods-besteffort-pod231bd2d7_0f09_4781_a4e1_e4ea026342dd.slice".to_string()];

        let mut uid_vec = Vec::<String>::new();

        for cgroup_path in cgroup_paths {
            let uid = extract_pod_uid(cgroup_path)
                .map_err(|e| format!("An error occured {}", e))
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
        let cgroup_paths = vec!["/sys/fs/cgroup/kubepods.slice/kubepods-besteffort.slice/kubepods-besteffort-pod231bd2d7_0f09_4781_a4e1_e4ea026342dd.slice".to_string(),
                                             "/sys/fs/cgroup/kubelet.slice/kubelet-kubepods.slice/kubelet-kubepods-besteffort.slice/kubelet-kubepods-besteffort-pod231bd2d7_0f09_4781_a4e1_e4ea026342dd.slice".to_string()];

        let mut index_vec = Vec::<usize>::new();
        for cgroup_path in cgroup_paths {
            let splits: Vec<&str> = cgroup_path.split("/").collect();

            let target_index = extract_target_from_splits(splits, "-pod").unwrap();
            index_vec.push(target_index);
        }
        let index_check = vec![6, 7];
        assert_eq!(index_vec, index_check);
    }

    #[test]
    fn extract_docker_id() {
        let cgroup_paths = vec!["/sys/fs/cgroup/kubepods.slice/kubepods-besteffort.slice/kubepods-besteffort-pod17fd3f7c_37e4_4009_8c38_e58b30691af3.slice/docker-13abd64c0ba349975a762476c9703b642d18077eabeb3aa1d941132048afc861.scope".to_string(),
                                             "/sys/fs/cgroup/kubelet.slice/kubelet-kubepods.slice/kubelet-kubepods-besteffort.slice/kubelet-kubepods-besteffort-pod17fd3f7c_37e4_4009_8c38_e58b30691af3.slice/docker-13abd64c0ba349975a762476c9703b642d18077eabeb3aa1d941132048afc861.scope".to_string()];

        let mut id_vec = Vec::<String>::new();
        for cgroup_path in cgroup_paths {
            let id = extract_container_id(cgroup_path).unwrap();
            id_vec.push(id);
        }
        let id_check = vec![
            "13abd64c0ba349975a762476c9703b642d18077eabeb3aa1d941132048afc861".to_string(),
            "13abd64c0ba349975a762476c9703b642d18077eabeb3aa1d941132048afc861".to_string(),
        ];
        assert_eq!(id_vec, id_check);
    }
}
