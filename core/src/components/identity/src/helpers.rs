#![allow(warnings)]
use crate::enums::IpProtocols;
use crate::structs::{PacketLog, TcpPacketRegistry, VethLog};
use anyhow::Error;
use aya::programs::tc::SchedClassifierLinkId;
use aya::{
    Bpf,
    maps::{MapData, perf::PerfEventArrayBuffer},
    programs::{SchedClassifier, TcAttachType},
};
use bytes::BytesMut;
use cortexbrain_common::constants;
use k8s_openapi::api::core::v1::Pod;
use kube::api::ObjectList;
use kube::{Api, Client};
use nix::net::if_::if_nameindex;
use std::collections::HashMap;
use std::fs;
use std::hash::Hash;
use std::path::PathBuf;
use std::result::Result::Ok;
use std::sync::Mutex;
use std::time::Duration;
use std::{
    borrow::BorrowMut,
    net::Ipv4Addr,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::time;
use tracing::{debug, error, info, warn};
use tracing_subscriber::fmt::format;

/*
 * TryFrom Trait implementation for IpProtocols enum
 * This is used to reconstruct the packet protocol based on the
 * IPV4 Header Protocol code
 */

impl TryFrom<u8> for IpProtocols {
    type Error = ();
    fn try_from(proto: u8) -> Result<Self, Self::Error> {
        match proto {
            1 => Ok(IpProtocols::ICMP),
            6 => Ok(IpProtocols::TCP),
            17 => Ok(IpProtocols::UDP),
            _ => Err(()),
        }
    }
}

/* helper functions to read and log net events in the container */
pub async fn display_events<T: BorrowMut<MapData>>(
    mut perf_buffers: Vec<PerfEventArrayBuffer<T>>,
    running: Arc<AtomicBool>,
    mut buffers: Vec<BytesMut>,
) {
    while running.load(Ordering::SeqCst) {
        for buf in perf_buffers.iter_mut() {
            match buf.read_events(&mut buffers) {
                std::result::Result::Ok(events) => {
                    for i in 0..events.read {
                        let data = &buffers[i];
                        if data.len() >= std::mem::size_of::<PacketLog>() {
                            let pl: PacketLog =
                                unsafe { std::ptr::read(data.as_ptr() as *const _) };
                            let src = reverse_be_addr(pl.src_ip);
                            let dst = reverse_be_addr(pl.dst_ip);
                            let src_port = u16::from_be(pl.src_port);
                            let dst_port = u16::from_be(pl.dst_port);
                            let event_id = pl.pid;

                            match IpProtocols::try_from(pl.proto) {
                                std::result::Result::Ok(proto) => {
                                    info!(
                                        "Event Id: {} Protocol: {:?} SRC: {}:{} -> DST: {}:{}",
                                        event_id, proto, src, src_port, dst, dst_port
                                    );
                                }
                                Err(_) => {
                                    info!(
                                        "Event Id: {} Protocol: Unknown ({})",
                                        event_id, pl.proto
                                    );
                                }
                            };
                        } else {
                            warn!("Received packet data too small: {} bytes", data.len());
                        }
                    }
                }
                Err(e) => {
                    error!("Error reading events: {:?}", e);
                }
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

pub fn reverse_be_addr(addr: u32) -> Ipv4Addr {
    let mut octects = addr.to_be_bytes();
    let [a, b, c, d] = [octects[3], octects[2], octects[1], octects[0]];
    let reversed_ip = Ipv4Addr::new(a, b, c, d);
    reversed_ip
}

pub async fn display_veth_events<T: BorrowMut<MapData>>(
    bpf: Arc<Mutex<Bpf>>,
    mut perf_buffers: Vec<PerfEventArrayBuffer<T>>,
    running: Arc<AtomicBool>,
    mut buffers: Vec<BytesMut>,
    mut link_ids: Arc<Mutex<HashMap<String, SchedClassifierLinkId>>>,
) {
    while running.load(Ordering::SeqCst) {
        for buf in perf_buffers.iter_mut() {
            match buf.read_events(&mut buffers) {
                std::result::Result::Ok(events) => {
                    for i in 0..events.read {
                        let data = &buffers[i];
                        if data.len() >= std::mem::size_of::<VethLog>() {
                            let vethlog: VethLog =
                                unsafe { std::ptr::read(data.as_ptr() as *const _) };

                            let name_bytes = vethlog.name;

                            let dev_addr_bytes = vethlog.dev_addr.to_vec();
                            let name = std::str::from_utf8(&name_bytes);
                            let state = vethlog.state;

                            let dev_addr = dev_addr_bytes;
                            let netns = vethlog.netns;
                            let mut event_type = String::new();
                            match vethlog.event_type {
                                1 => {
                                    event_type = "creation".to_string();
                                }
                                2 => {
                                    event_type = "deletion".to_string();
                                }
                                _ => warn!("unknown event_type"),
                            }
                            match name {
                                std::result::Result::Ok(veth_name) => {
                                    info!(
                                        "[{}] Triggered action: register_netdevice event_type:{:?} Manipulated veth: {:?} state:{:?} dev_addr:{:?}",
                                        netns,
                                        event_type,
                                        veth_name.trim_end_matches("\0").to_string(),
                                        state,
                                        dev_addr
                                    );
                                    match attach_detach_veth(
                                        bpf.clone(),
                                        vethlog.event_type,
                                        veth_name,
                                        link_ids.clone(),
                                    )
                                    .await
                                    {
                                        std::result::Result::Ok(_) => {
                                            info!("Attach/Detach veth function attached correctly");
                                        }
                                        Err(e) => error!(
                                            "Error attaching Attach/Detach function. Error : {}",
                                            e
                                        ),
                                    }
                                }
                                Err(_) => info!("Unknown name or corrupted field"),
                            }
                        } else {
                            warn!("Corrupted data");
                        }
                    }
                }
                Err(e) => {
                    error!("Error reading veth events: {:?}", e);
                }
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

pub fn ignore_iface(iface: &str) -> bool {
    let ignored_interfaces = ["eth0", "docker0", "tunl0", "lo"];
    ignored_interfaces.contains(&iface)
}

//filter the interfaces,exclude docker0,eth0,lo interfaces
pub fn get_veth_channels() -> Vec<String> {
    //filter interfaces and save the output in the
    let mut interfaces: Vec<String> = Vec::new();

    if let Ok(ifaces) = if_nameindex() {
        for iface in &ifaces {
            let iface_name = iface.name().to_str().unwrap().to_owned();
            if !ignore_iface(&iface_name) {
                interfaces.push(iface_name);
            } else {
                info!("skipping interface {:?}", iface_name);
            }
        }
    }

    interfaces
}

async fn attach_detach_veth(
    bpf: Arc<Mutex<Bpf>>,
    event_type: u8,
    iface: &str,
    link_ids: Arc<Mutex<HashMap<String, SchedClassifierLinkId>>>,
) -> Result<(), anyhow::Error> {
    info!(
        "attach_detach_veth called: event_type={}, iface={}",
        event_type, iface
    );
    match event_type {
        1 => {
            let mut bpf = bpf.lock().unwrap();
            let program: &mut SchedClassifier = bpf
                .program_mut("identity_classifier")
                .ok_or_else(|| anyhow::anyhow!("program 'identity_classifier' not found"))?
                .try_into()?;

            let iface = iface.trim_end_matches('\0');

            if ignore_iface(iface) {
                info!("Skipping ignored interface: {}", iface);
                return Ok(());
            }

            let mut link_ids = link_ids.lock().unwrap();
            match program.attach(iface, TcAttachType::Ingress) {
                std::result::Result::Ok(link_id) => {
                    info!(
                        "Program 'identity_classifier' attached to interface {}",
                        iface
                    );
                    link_ids.insert(iface.to_string(), link_id);
                }
                Err(e) => error!("Error attaching program to interface {}: {:?}", iface, e),
            }
        }
        2 => {
            // INFO: Detaching occurs automatically when veth is deleted by kernel itself
            let mut link_ids = link_ids.lock().unwrap();
            match link_ids.remove(iface) {
                Some(_) => {
                    info!("Successfully detached program from interface {}", iface);
                }
                None => {
                    error!("Interface {} not found in link_ids", iface);
                    return Err(anyhow::anyhow!("Interface {} not found in link_ids", iface));
                }
            }
        }
        _ => {
            error!("Unknown event type: {}", event_type);
        }
    }
    Ok(())
}

// CHECK THIS DIR: /sys/fs/cgroup/kubelet.slice/kubelet-kubepods.slice/kubelet-kubepods-besteffort.slice
/* helper functions to display events from the TcpPacketRegistry structure */
pub async fn display_tcp_registry_events<T: BorrowMut<MapData>>(
    mut perf_buffers: Vec<PerfEventArrayBuffer<T>>,
    running: Arc<AtomicBool>,
    mut buffers: Vec<BytesMut>,
) {
    while running.load(Ordering::SeqCst) {
        for buf in perf_buffers.iter_mut() {
            match buf.read_events(&mut buffers) {
                std::result::Result::Ok(events) => {
                    for i in 0..events.read {
                        let data = &buffers[i];
                        if data.len() >= std::mem::size_of::<TcpPacketRegistry>() {
                            let tcp_pl: TcpPacketRegistry =
                                unsafe { std::ptr::read(data.as_ptr() as *const _) };
                            let src = reverse_be_addr(tcp_pl.src_ip);
                            let dst = reverse_be_addr(tcp_pl.dst_ip);
                            let src_port = u16::from_be(tcp_pl.src_port);
                            let dst_port = u16::from_be(tcp_pl.dst_port);
                            let event_id = tcp_pl.pid;
                            let command = tcp_pl.command.to_vec();
                            let end = command
                                .iter()
                                .position(|&x| x == 0)
                                .unwrap_or(command.len());
                            let command_str = String::from_utf8_lossy(&command[..end]).to_string();
                            let cgroup_id = tcp_pl.cgroup_id;

                            // construct the parent path
                            //let proc_path = PathBuf::from("/proc")
                            //    .join(event_id.to_string())
                            //    .join("cgroup");

                            //let proc_content = fs::read_to_string(&proc_path);
                            //match proc_content {
                            //    Ok(proc_content) => {
                            match IpProtocols::try_from(tcp_pl.proto) {
                                std::result::Result::Ok(proto) => {
                                    info!(
                                        "Event Id: {} Protocol: {:?} SRC: {}:{} -> DST: {}:{} Command: {} Cgroup_id: {}",
                                        event_id,
                                        proto,
                                        src,
                                        src_port,
                                        dst,
                                        dst_port,
                                        command_str,
                                        cgroup_id //proc_content
                                    );
                                }
                                Err(_) => {
                                    info!(
                                        "Event Id: {} Protocol: Unknown ({})",
                                        event_id, tcp_pl.proto
                                    );
                                }
                            };
                            //}
                            //Err(e) =>
                            //    eprintln!(
                            //        "An error occured while accessing the content from the {:?} path: {}",
                            //        &proc_path,
                            //       e
                            //    ),
                        } else {
                            warn!("Received packet data too small: {} bytes", data.len());
                        }
                    }
                }
                Err(e) => {
                    error!("Error reading events: {:?}", e);
                }
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

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

pub async fn scan_cgroup_cronjob(time_delta: u64) -> Result<(), Error> {
    let interval = std::time::Duration::from_secs(time_delta);
    let mut discovered_pods = HashMap::<String, String>::new();
    while true {
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

        //read the subpaths
        let mut uids = Vec::<String>::new();
        for subpath in scanned_subpaths_v2 {
            let uid = extract_pod_uid(subpath.clone())
                .expect("An error occured during the extraction of pod UIDs");
            debug!("Debugging extracted UID: {:?}", &uid);
            uids.push(uid);
        }
        // get pod information from UID and store the info in an HashMqp for O(1) access
        let service_map = get_pod_info().await?;

        for (uid) in uids {
            if let Some(name) = service_map.get(&uid) {
                info!("UID (from eBPF): {} name:(from K8s): {}", &uid, name);
            }
        }

        info!(
            "Cronjob completed a cgroup scan cycle. Next scan will be in {} seconds",
            time_delta
        );
        time::sleep(interval).await;
    }

    Ok(())
}

fn extract_pod_uid(cgroup_path: String) -> Result<String, Error> {
    // example of cgroup path:
    // /sys/fs/cgroup/kubelet.slice/kubelet-kubepods.slice/kubelet-kubepods-besteffort.slice/kubelet-kubepods-besteffort-pod93580201_87d5_44e6_9779_f6153ca17637.slice
    // or
    // /sys/fs/cgroup/kubelet.slice/kubelet-kubepods.slice/kubelet-kubepods-burstable.slice/kubelet-kubepods-burstable-poddd3a1c6b_af40_41b1_8e1c_9e31fe8d96cb.slice

    // split the path by "/"
    let splits: Vec<&str> = cgroup_path.split("/").collect();
    let mut uid_vec = Vec::<String>::new();
    debug!("Debugging splits: {:?}", &splits);

    let mut pod_split_vec = Vec::<String>::new();

    let index = extract_target_from_splits(splits.clone())?;

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

fn extract_target_from_splits(splits: Vec<&str>) -> Result<usize, Error> {
    for (index, split) in splits.iter().enumerate() {
        // find the split that contains the word 'pod'
        if split.contains("-pod") {
            debug!("Target index; {}", index);
            return Ok(index);
        }
    }
    Err(Error::msg("'-pod' word not found in split"))
}

/* unfortunately you cannot query the pods using the uids directly from ListParams */
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
async fn get_pod_info() -> Result<HashMap<String, String>, Error> {
    let all_pods = query_all_pods().await?;

    let mut service_map = HashMap::<String, String>::new();

    for pod in all_pods {
        if let (Some(name), Some(uid)) = (pod.metadata.name, pod.metadata.uid) {
            service_map.insert(uid, name);
        }
    }

    Ok(service_map)
}

mod tests {
    use tracing_subscriber::fmt::format;

    use crate::helpers::{extract_pod_uid, extract_target_from_splits};

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
            let mut splits: Vec<&str> = cgroup_path.split("/").collect();

            let target_index = extract_target_from_splits(splits).unwrap();
            index_vec.push(target_index);
        }
        let index_check = vec![6, 7];
        assert_eq!(index_vec, index_check);
    }
}
