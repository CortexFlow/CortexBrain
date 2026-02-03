use cortexbrain_common::buffer_type::{reverse_be_addr,IpProtocols};
use cortexbrain_common::buffer_type::{PacketLog, TcpPacketRegistry, VethLog};

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

/*
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

 */
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
/*
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

 */
// enum BuffersType
pub enum BufferType {
    PacketLog,
    TcpPacketRegistry,
    VethLog,
}

// TODO: add variant for OTEL log exporters
impl BufferType {
    async fn read_packet_log(buffers: &mut [BytesMut], tot_events: i32, offset: i32) {
        for i in offset..tot_events {
            let vec_bytes = &buffers[i as usize];
            if vec_bytes.len() < std::mem::size_of::<PacketLog>() {
                error!(
                    "Corrupted data. Readed {:?} bytes expected {} bytes",
                    vec_bytes,
                    std::mem::size_of::<PacketLog>()
                )
            }
            if vec_bytes.len() >= std::mem::size_of::<PacketLog>() {
                let pl: PacketLog =
                    unsafe { std::ptr::read_unaligned(vec_bytes.as_ptr() as *const _) }; // reading raw bytes

                // extracting struct info from bytes
                let src_ip = reverse_be_addr(pl.src_ip);
                let dst_ip = reverse_be_addr(pl.dst_ip);
                let src_port = u16::from_be(pl.src_port);
                let dst_port = u16::from_be(pl.dst_port);
                let event_id = pl.pid;
                let protocol = pl.proto;

                // protocol extraction
                match IpProtocols::try_from(protocol) {
                    Ok(proto) => {
                        info!(
                            "Event Id: {} Protocol: {:?} SRC: {}:{} -> DST: {}:{}",
                            event_id, proto, src_ip, src_port, dst_ip, dst_port
                        );
                    }
                    Err(e) => {
                        error!("Unknown protocol. Data maybe corrupted. Reason:{:?}", e);
                    }
                }
            }
        }
    }
    async fn read_tcp_registry_log(buffers: &mut [BytesMut], tot_events: i32, offset: i32) {
        for i in offset..tot_events {
            let vec_bytes = &buffers[i as usize];
            if vec_bytes.len() < std::mem::size_of::<TcpPacketRegistry>() {
                error!(
                    "Corrupted data. Readed {:?} bytes expected {} bytes",
                    vec_bytes,
                    std::mem::size_of::<TcpPacketRegistry>()
                )
            }
            if vec_bytes.len() >= std::mem::size_of::<TcpPacketRegistry>() {
                let pl: TcpPacketRegistry =
                    unsafe { std::ptr::read_unaligned(vec_bytes.as_ptr() as *const _) }; // reading raw bytes

                // extracting struct info from bytes
                let src = reverse_be_addr(pl.src_ip);
                let dst = reverse_be_addr(pl.dst_ip);
                let src_port = u16::from_be(pl.src_port);
                let dst_port = u16::from_be(pl.dst_port);
                let event_id = pl.pid;
                let command = pl.command.to_vec();
                let end = command
                    .iter()
                    .position(|&x| x == 0)
                    .unwrap_or(command.len());
                let command_str = String::from_utf8_lossy(&command[..end]).to_string();
                let cgroup_id = pl.cgroup_id;
                let protocol = pl.proto;

                // protocol extraction
                match IpProtocols::try_from(protocol) {
                    Ok(proto) => {
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
                    Err(e) => {
                        error!("Unknown protocol. Data maybe corrupted. Reason:{:?}", e);
                    }
                }
            }
        }
    }
    async fn read_and_handle_veth_log(
        //link_ids: Arc<Mutex<HashMap<String, SchedClassifierLinkId>>>,
        //bpf: Arc<Mutex<Ebpf>>,
        buffers: &mut [BytesMut],
        tot_events: i32,
        offset: i32,
    ) {
        for i in offset..tot_events {
            let vec_bytes = &buffers[i as usize];
            if vec_bytes.len() < std::mem::size_of::<VethLog>() {
                error!(
                    "Corrupted data. Readed {:?} bytes expected {} bytes",
                    vec_bytes,
                    std::mem::size_of::<VethLog>()
                )
            }
            if vec_bytes.len() >= std::mem::size_of::<VethLog>() {
                let pl: VethLog =
                    unsafe { std::ptr::read_unaligned(vec_bytes.as_ptr() as *const _) }; // reading raw bytes

                // extracting struct info from bytes
                let name_bytes = pl.name;

                let dev_addr_bytes = pl.dev_addr;
                let name = std::str::from_utf8(&name_bytes);
                let state = pl.state;

                let dev_addr = dev_addr_bytes;
                let netns = pl.netns;
                let mut event_type = String::new();

                // event_type extraction
                match pl.event_type {
                    1 => {
                        event_type = "creation".to_string();
                        match name {
                            Ok(veth_name) => {
                                info!(
                                    "[{}] Veth Event: Type: {} Name: {} Dev_addr: {:x?} State: {}",
                                    netns,
                                    event_type,
                                    veth_name.trim_end_matches("\0"),
                                    dev_addr,
                                    state
                                );
                                // TODO: this logic needs to live in a separate space
                                //match attach_detach_veth(
                                //    bpf.clone(),
                                //    1,
                                //    veth_name,
                                //    link_ids.clone(),
                                //)
                                //.await
                                //{
                                //    Ok(_) => {
                                //        info!(
                                //            "[{}] Successfully attached Attach/Detach function for veth: {}",
                                //            netns,
                                //            veth_name.trim_end_matches("\0")
                                //        );
                                //    }
                                //    Err(e) => {
                                //        info!(
                                //            "[{}] Error attaching Attach/Detach function. Error : {}",
                                //            netns, e
                                //        );
                                //    }
                                //}
                            }
                            Err(e) => {
                                error!(
                                    "Failed to extract veth name during event_type = creation (1).Reason:{}",
                                    e
                                );
                            }
                        }
                    }
                    2 => {
                        event_type = "deletion".to_string();
                        match name {
                            Ok(veth_name) => {
                                info!(
                                    "[{}] Veth Event: Type: {} Name: {} Dev_addr: {:x?} State: {}",
                                    netns,
                                    event_type,
                                    veth_name.trim_end_matches("\0"),
                                    dev_addr,
                                    state
                                );
                                // TODO: this logic needs to live in a separate space
                                //match attach_detach_veth(
                                //    bpf.clone(),
                                //    2,
                                //    veth_name,
                                //    link_ids.clone(),
                                //)
                                //.await
                                //{
                                //    Ok(_) => {
                                //        info!(
                                //            "[{}] Successfully attached Attach/Detach function for veth: {}",
                                //            netns,
                                //            veth_name.trim_end_matches("\0")
                                //        );
                                //    }
                                //    Err(e) => {
                                //        info!(
                                //            "[{}] Error attaching Attach/Detach function. Error : {}",
                                //            netns, e
                                //        );
                                //    }
                                // }
                            }
                            Err(e) => {
                                error!(
                                    "Failed to extract veth name during event_type = deletion (2).Reason:{}",
                                    e
                                );
                            }
                        }
                    }
                    _ => {
                        warn!("Unknown event type")
                    }
                }
            }
        }
    }
}

// docs: read buffer function:
// template function that take a mut perf_event_array_buffer of type T and a mutable buffer of Vec<BytesMut>

pub async fn read_perf_buffer<T: std::borrow::BorrowMut<aya::maps::MapData>>(
    //bpf: Arc<Mutex<Ebpf>>, // this is only for read_and_handle_veth_logs fn
    //link_ids: Arc<Mutex<HashMap<String, SchedClassifierLinkId>>>, // this is only for read_and_handle_veth_logs fn
    mut array_buffers: Vec<PerfEventArrayBuffer<T>>,
    mut buffers: Vec<bytes::BytesMut>,
    buffer_type: BufferType,
) {
    // loop over the buffers
    loop {
        for buf in array_buffers.iter_mut() {
            match buf.read_events(&mut buffers) {
                Ok(events) => {
                    // triggered if some events are lost
                    if events.lost > 0 {
                        tracing::debug!("Lost events: {} ", events.lost);
                    }
                    // triggered if some events are readed
                    if events.read > 0 {
                        tracing::debug!("Readed events: {}", events.read);
                        let offset = 0;
                        let tot_events = events.read as i32;

                        //read the events in the buffer
                        match buffer_type {
                            BufferType::PacketLog => {
                                BufferType::read_packet_log(&mut buffers, tot_events, offset).await
                            }
                            BufferType::TcpPacketRegistry => {
                                BufferType::read_tcp_registry_log(&mut buffers, tot_events, offset)
                                    .await
                            }
                            BufferType::VethLog => {
                                BufferType::read_and_handle_veth_log(
                                    //link_ids.clone(),
                                    //bpf.clone(),
                                    &mut buffers,
                                    tot_events,
                                    offset,
                                )
                                .await
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Cannot read events from buffer. Reason: {} ", e);
                }
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await; // small sleep 
    }
}
