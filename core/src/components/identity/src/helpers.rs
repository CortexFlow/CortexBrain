use crate::enums::IpProtocols;
use crate::structs::{PacketLog, TcpPacketRegistry, VethLog};

use aya::Ebpf;
use aya::programs::tc::SchedClassifierLinkId;
use aya::{
    maps::{MapData, perf::PerfEventArrayBuffer},
    programs::{SchedClassifier, TcAttachType},
};
use bytes::BytesMut;
use nix::net::if_::if_nameindex;
use std::{
    borrow::BorrowMut, collections::HashMap, net::Ipv4Addr, result::Result::Ok, sync::Arc,
    sync::Mutex,
};
use tracing::{debug, error, event, info, span, warn};

//
// TryFrom Trait implementation for IpProtocols enum
// This is used to reconstruct the packet protocol based on the
// IPV4 Header Protocol code
//

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
    mut buffers: Vec<BytesMut>,
) {
    //  FIXME: here maybe we need to use a loop with tokio::select
    loop {
        for buf in perf_buffers.iter_mut() {
            match buf.read_events(&mut buffers) {
                std::result::Result::Ok(events) => {
                    let offset = 0 as usize;
                    if events.read > 0 {
                        debug!("Read {} events", events.read);
                    }
                    if events.lost > 0 {
                        debug!("Lost events: {}", events.lost);
                    }
                    for i in offset..events.read {
                        let data = &buffers[i];
                        if data.len() < std::mem::size_of::<PacketLog>() {
                            let failed_events_span =
                                span!(tracing::Level::INFO, "corrupted_packets_events");
                            let _enter: span::Entered<'_> = failed_events_span.enter();
                            event!(
                                tracing::Level::WARN,
                                "Corrupted data. data_len = {} data_ptr = {}. Min size required: {} bytes",
                                data.len(),
                                data.as_ptr() as usize,
                                std::mem::size_of::<PacketLog>()
                            );
                            continue;
                        }
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
                                    let packets_events_span = span!(tracing::Level::INFO, "packets_event",event_id=%event_id, protocol = %format!("{:?}", proto));
                                    let _enter = packets_events_span.enter();
                                    event!(
                                        tracing::Level::INFO,
                                        "Event Id: {} Protocol: {:?} SRC: {}:{} -> DST: {}:{}",
                                        event_id,
                                        proto,
                                        src,
                                        src_port,
                                        dst,
                                        dst_port
                                    );
                                }
                                Err(e) => {
                                    let failed_packets_events_span = span!(tracing::Level::INFO, "failed_packets_event", event_id=%event_id, protocol = %pl.proto);
                                    let _enter = failed_packets_events_span.enter();
                                    event!(
                                        tracing::Level::INFO,
                                        "Event Id: {} Protocol: Unknown ({}). Error: {:?}",
                                        event_id,
                                        pl.proto,
                                        e
                                    )
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

// docs:
// This function perform a byte swap from little-endian to big-endian
// It's used to reconstruct the correct IPv4 address from the u32 representation
//
// Takes a u32 address in big-endian format and returns a Ipv4Addr with reversed octets
//
pub fn reverse_be_addr(addr: u32) -> Ipv4Addr {
    let octects = addr.to_be_bytes();
    let [a, b, c, d] = [octects[3], octects[2], octects[1], octects[0]];
    let reversed_ip = Ipv4Addr::new(a, b, c, d);
    reversed_ip
}

pub async fn display_veth_events<T: BorrowMut<MapData>>(
    bpf: Arc<Mutex<Ebpf>>,
    mut perf_buffers: Vec<PerfEventArrayBuffer<T>>,
    mut buffers: Vec<BytesMut>,
    link_ids: Arc<Mutex<HashMap<String, SchedClassifierLinkId>>>,
) {
    //  FIXME: here maybe we need to use a loop with tokio::select
    loop {
        for buf in perf_buffers.iter_mut() {
            match buf.read_events(&mut buffers) {
                std::result::Result::Ok(events) => {
                    // debug: log the readed events
                    if events.read > 0 {
                        debug!("Read {} veth events", events.read);
                    }
                    // debug: log the lost events
                    if events.lost > 0 {
                        debug!("Lost {} veth events", events.lost);
                    }
                    let offset = 0 as usize;
                    for i in offset..events.read {
                        let data = &buffers[i];
                        let veth_events_span = span!(tracing::Level::INFO, "corrupted_veth_events");
                        // error: data is smaller that the vethlog structure
                        let _enter = veth_events_span.enter();
                        if data.len() < std::mem::size_of::<VethLog>() {
                            warn!(
                                "Corrupted data. data_len = {} data_ptr = {}. Min size required: {} bytes",
                                data.len(),
                                data.as_ptr() as usize,
                                std::mem::size_of::<VethLog>()
                            );
                            continue;
                        }
                        // correct size: data is logged correctly
                        if data.len() >= std::mem::size_of::<VethLog>() {
                            let vethlog: VethLog =
                                unsafe { std::ptr::read_unaligned(data.as_ptr() as *const _) };
                            //TODO: can this pattern be safe instead of using unsafe?

                            let name_bytes = vethlog.name;

                            let dev_addr_bytes = vethlog.dev_addr;
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
                                    let veth_events_span = span!(tracing::Level::INFO, "veth_event", veth_name = %veth_name.trim_end_matches("\0"), event_type = %event_type.as_str());
                                    let _enter = veth_events_span.enter();
                                    event!(
                                        tracing::Level::INFO,
                                        "[{}] Veth Event: Type: {} Name: {} Dev_addr: {:x?} State: {}",
                                        netns,
                                        event_type,
                                        veth_name.trim_end_matches("\0"),
                                        dev_addr,
                                        state
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
                                            event!(
                                                tracing::Level::INFO,
                                                "[{}] Successfully attached Attach/Detach function for veth: {}",
                                                netns,
                                                veth_name.trim_end_matches("\0")
                                            );
                                        }
                                        Err(e) => {
                                            let failed_veth_events_span = span!(tracing::Level::ERROR, "failed_veth_event_attach_detach", veth_name = %veth_name.trim_end_matches("\0"));
                                            let _enter = failed_veth_events_span.enter();
                                            event!(
                                                tracing::Level::ERROR,
                                                "[{}] Error attaching Attach/Detach function. Error : {}",
                                                netns,
                                                e
                                            )
                                        }
                                    }
                                }
                                Err(e) => {
                                    event!(
                                        tracing::Level::WARN,
                                        "Corrupted veth name field. Error: {:?}",
                                        e
                                    );
                                }
                            }
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

// docs:
// This function checks if the given interface name is in the list of ignored interfaces
// Takes a interface name (iface) as &str and returns true if the interface should be ignored
// Typically we want to ignore eth0,docker0,tunl0,lo interfaces because they are not relevant for the internal monitoring
//
pub fn ignore_iface(iface: &str) -> bool {
    let ignored_interfaces = ["eth0", "docker0", "tunl0", "lo"];
    ignored_interfaces.contains(&iface)
}

// docs:
// This function retrieves the list of veth interfaces on the system, filtering out ignored interfaces with
// the ignore_iface function.
//
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
    bpf: Arc<Mutex<Ebpf>>,
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
            //
            // EVENT_TYPE 1: Attach the program to the veth inferfaces
            //

            let mut bpf = bpf
                .lock()
                .map_err(|e| anyhow::anyhow!("Cannot get value from lock : {}", e))?;
            let program: &mut SchedClassifier = bpf
                .program_mut("identity_classifier")
                .ok_or_else(|| anyhow::anyhow!("program 'identity_classifier' not found"))?
                .try_into()?;

            let iface = iface.trim_end_matches('\0');

            if ignore_iface(iface) {
                info!("Skipping ignored interface: {}", iface);
                return Ok(());
            }

            let mut link_ids = link_ids
                .lock()
                .map_err(|e| anyhow::anyhow!("Cannot get value from lock when attaching: {}", e))?;
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
            //
            // EVENT_TYPE 2: Detach the program from the veth interfaces
            // INFO: Detaching occurs automatically when veth is deleted by kernel itself
            //

            let mut link_ids = link_ids
                .lock()
                .map_err(|e| anyhow::anyhow!("Cannot get value from lock when detaching: {}", e))?;
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

/* helper functions to display events from the TcpPacketRegistry structure */
pub async fn display_tcp_registry_events<T: BorrowMut<MapData>>(
    mut perf_buffers: Vec<PerfEventArrayBuffer<T>>,
    mut buffers: Vec<BytesMut>,
) {
    //  FIXME: here maybe we need to use a loop with tokio::select
    loop {
        for buf in perf_buffers.iter_mut() {
            match buf.read_events(&mut buffers) {
                std::result::Result::Ok(events) => {
                    let offset = 0;
                    for i in offset..events.read {
                        let data = &buffers[i];
                        if data.len() < std::mem::size_of::<TcpPacketRegistry>() {
                            let failed_tcp_events_span =
                                span!(tracing::Level::INFO, "failed_tcp_registry_event");
                            let _enter: span::Entered<'_> = failed_tcp_events_span.enter();
                            event!(
                                tracing::Level::WARN,
                                "Corrupted data. data_len = {} data_ptr = {}. Min size required: {} bytes",
                                data.len(),
                                data.as_ptr() as usize,
                                std::mem::size_of::<TcpPacketRegistry>()
                            );
                            continue;
                        }
                        if data.len() >= std::mem::size_of::<TcpPacketRegistry>() {
                            let tcp_pl: TcpPacketRegistry =
                                unsafe { std::ptr::read(data.as_ptr() as *const _) };
                            //TODO: can this pattern be safe?
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

                            match IpProtocols::try_from(tcp_pl.proto) {
                                std::result::Result::Ok(proto) => {
                                    let tcp_events_span = span!(tracing::Level::INFO, "tcp_registry_event", command = %command_str.as_str(), cgroup_id = %cgroup_id);
                                    let _enter = tcp_events_span.enter();
                                    event!(
                                        tracing::Level::INFO,
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
                                Err(e) => {
                                    event!(
                                        tracing::Level::INFO,
                                        "Event Id: {} Protocol: Unknown ({}) Command: {} Cgroup_id: {} Error: {:?}",
                                        event_id,
                                        tcp_pl.proto,
                                        command_str,
                                        cgroup_id,
                                        e
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
