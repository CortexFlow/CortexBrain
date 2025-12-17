#![allow(warnings)]
use crate::enums::IpProtocols;
use crate::structs::{ PacketLog, VethLog, TcpPacketRegistry };
use anyhow::Error;
use aya::programs::tc::SchedClassifierLinkId;
use aya::{
    Bpf,
    maps::{ MapData, perf::PerfEventArrayBuffer },
    programs::{ SchedClassifier, TcAttachType },
};
use bytes::BytesMut;
use nix::net::if_::if_nameindex;
use tokio::time;
use tracing_subscriber::fmt::format;
use std::collections::HashMap;
use std::fs;
use std::hash::Hash;
use std::result::Result::Ok;
use std::sync::Mutex;
use std::path::PathBuf;
use std::time::Duration;
use std::{ borrow::BorrowMut, net::Ipv4Addr, sync::{ Arc, atomic::{ AtomicBool, Ordering } } };
use tracing::{ error, info, warn };
use kube::{ Api, Client };
use k8s_openapi::api::core::v1::Pod;
use cortexbrain_common::constants;

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
    mut buffers: Vec<BytesMut>
) {
    while running.load(Ordering::SeqCst) {
        for buf in perf_buffers.iter_mut() {
            match buf.read_events(&mut buffers) {
                std::result::Result::Ok(events) => {
                    for i in 0..events.read {
                        let data = &buffers[i];
                        if data.len() >= std::mem::size_of::<PacketLog>() {
                            let pl: PacketLog = unsafe {
                                std::ptr::read(data.as_ptr() as *const _)
                            };
                            let src = reverse_be_addr(pl.src_ip);
                            let dst = reverse_be_addr(pl.dst_ip);
                            let src_port = u16::from_be(pl.src_port);
                            let dst_port = u16::from_be(pl.dst_port);
                            let event_id = pl.pid;

                            match IpProtocols::try_from(pl.proto) {
                                std::result::Result::Ok(proto) => {
                                    info!(
                                        "Event Id: {} Protocol: {:?} SRC: {}:{} -> DST: {}:{}",
                                        event_id,
                                        proto,
                                        src,
                                        src_port,
                                        dst,
                                        dst_port
                                    );
                                }
                                Err(_) => {
                                    info!(
                                        "Event Id: {} Protocol: Unknown ({})",
                                        event_id,
                                        pl.proto
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
    mut link_ids: Arc<Mutex<HashMap<String, SchedClassifierLinkId>>>
) {
    while running.load(Ordering::SeqCst) {
        for buf in perf_buffers.iter_mut() {
            match buf.read_events(&mut buffers) {
                std::result::Result::Ok(events) => {
                    for i in 0..events.read {
                        let data = &buffers[i];
                        if data.len() >= std::mem::size_of::<VethLog>() {
                            let vethlog: VethLog = unsafe {
                                std::ptr::read(data.as_ptr() as *const _)
                            };

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
                                    match
                                        attach_detach_veth(
                                            bpf.clone(),
                                            vethlog.event_type,
                                            veth_name,
                                            link_ids.clone()
                                        ).await
                                    {
                                        std::result::Result::Ok(_) => {
                                            info!("Attach/Detach veth function attached correctly");
                                        }
                                        Err(e) =>
                                            error!("Error attaching Attach/Detach function. Error : {}", e),
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
    link_ids: Arc<Mutex<HashMap<String, SchedClassifierLinkId>>>
) -> Result<(), anyhow::Error> {
    info!("attach_detach_veth called: event_type={}, iface={}", event_type, iface);
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
                    info!("Program 'identity_classifier' attached to interface {}", iface);
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
    mut buffers: Vec<BytesMut>
) {
    while running.load(Ordering::SeqCst) {
        for buf in perf_buffers.iter_mut() {
            match buf.read_events(&mut buffers) {
                std::result::Result::Ok(events) => {
                    for i in 0..events.read {
                        let data = &buffers[i];
                        if data.len() >= std::mem::size_of::<TcpPacketRegistry>() {
                            let tcp_pl: TcpPacketRegistry = unsafe {
                                std::ptr::read(data.as_ptr() as *const _)
                            };
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
                                        cgroup_id
                                        //proc_content
                                    );
                                }
                                Err(_) => {
                                    info!(
                                        "Event Id: {} Protocol: Unknown ({})",
                                        event_id,
                                        tcp_pl.proto
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
            error!("Error reading cgroup directory {:?}: {}", &target_path.clone(), e);
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
        let scanned_paths = scan_cgroup_paths(
            "/sys/fs/cgroup/kubelet.slice".to_string()
        ).await.expect("An error occured during the cgroup scan");
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
        for subpath in scanned_subpaths_v2 {
            //info!("Scanned cgroup subpath: {}", subpath);
            let uids = extract_pod_uid(subpath.clone()).expect(
                "An error occured during the extraction of pod UIDs"
            );
            info!("Debugging extracted UIDS: {:?}",&uids);

            for uid in &uids {
                // store the UID in an HashMap for O(1) access
                discovered_pods.insert(uid.clone(), subpath.clone());
                info!("Extracted Pod UID: {}", uid);
            }

            // get pod information from UID
            let service_map = get_pod_info_from_uid(&uids).await.expect("An error occured during the execution of the get_pod_info_from_uid function");

            for (k,v) in service_map {
                info!("Service name : {:?} , Service UID: {:?}",k,v);
            }

        }

        info!("Cronjob completed a cgroup scan cycle. Next scan will be in {} seconds", time_delta);
        time::sleep(interval).await;
    }

    Ok(())
}

fn extract_pod_uid(cgroup_path: String) -> Result<Vec<String>, Error> {
    // example of cgroup path:
    // /sys/fs/cgroup/kubelet.slice/kubelet-kubepods.slice/kubelet-kubepods-besteffort.slice/kubelet-kubepods-besteffort-pod93580201_87d5_44e6_9779_f6153ca17637.slice
    // or
    // /sys/fs/cgroup/kubelet.slice/kubelet-kubepods.slice/kubelet-kubepods-burstable.slice/kubelet-kubepods-burstable-poddd3a1c6b_af40_41b1_8e1c_9e31fe8d96cb.slice

    // split the path by "/"
    let splits: Vec<&str> = cgroup_path.split("/").collect();
    let mut uid_vec = Vec::<String>::new();
    info!("Debugging splits: {:?}", &splits);

    if
        splits[7].starts_with("kubelet-kubepods-besteffort-pod") ||
        splits[7].starts_with("kubelet-kubepods-burstable-pod")
    {
        let uid = splits[7]
            .replace("kubelet-kubepods-besteffort-pod", "")
            .replace("kubelet-kubepods-burstable-pod", "")
            .replace("kubelet-kubepods-pod", "")
            .replace(".slice", "");
        if !uid.is_empty() {
            uid_vec.push(uid);
        }
    } else if
        splits[7].starts_with("kubelet-kubepods-besteffort-pod")||
        splits[7].starts_with("kubelet-kubepods-burstable-pod")
    {
        let uid = splits[7]
            .replace("kubepods-besteffort-pod", "")
            .replace("kubepods-burstable-pod", "")
            .replace("kubepods-pod", "")
            .replace(".slice", "");

        if !uid.is_empty() {
            uid_vec.push(uid);
        }
    }

    Ok(uid_vec)
}

async fn get_pod_info_from_uid(uids: &Vec<String>) -> Result<HashMap<String, String>, Error> {
    // use the kube client to get the pod information from the UID
    let client = Client::try_default().await.expect("Cannot connect to kubernetes client");
    let pods: Api<Pod> = Api::all(client);

    let mut service_map: HashMap<String, String> = HashMap::new();

    for uid in uids {
        let lp = kube::api::ListParams::default().fields(&format!("metadata.uid={}", uid));
        info!("Debugging kube::api::ListParams: {:?}", &lp);
        let pod_list = pods
            .list(&lp).await
            .expect("An error occured during the pod lits extraction");
        for p in pod_list {
            if let Some(name) = p.metadata.name {
                service_map.insert(name, uid.to_string());
            }
            if let Some(namespace) = p.metadata.namespace {
                info!("Pod Namespace: {} for UID: {}", namespace, uid);
            }
        }
    }
    Ok(service_map)
}
