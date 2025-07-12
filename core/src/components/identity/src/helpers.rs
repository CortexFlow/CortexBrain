use crate::enums::IpProtocols;
use crate::structs::{PacketLog, VethLog};
use aya::programs::tc::SchedClassifierLinkId;
use aya::{
    Bpf,
    maps::{
        MapData,
        perf::{PerfEventArray, PerfEventArrayBuffer},
    },
    programs::{SchedClassifier, TcAttachType},
    util::online_cpus,
};
use bytes::BytesMut;
use nix::net::if_::if_nameindex;
use std::collections::HashMap;
use std::sync::Mutex;
use std::{
    ascii,
    borrow::BorrowMut,
    net::Ipv4Addr,
    string,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use tracing::{error, event, info, warn};

use anyhow::Context;
use std::path::Path;
use tokio::{fs, signal};
/*
 * decleare bpf path env variable
 */
const BPF_PATH: &str = "BPF_PATH";
const IFACE: &str = "IFACE";
use std::result::Result::Ok as Okk;

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

pub async fn display_events<T: BorrowMut<MapData>>(
    mut perf_buffers: Vec<PerfEventArrayBuffer<T>>,
    running: Arc<AtomicBool>,
    mut buffers: Vec<BytesMut>,
) {
    while running.load(Ordering::SeqCst) {
        for buf in perf_buffers.iter_mut() {
            match buf.read_events(&mut buffers) {
                Ok(events) => {
                    for i in 0..events.read {
                        let data = &buffers[i];
                        if data.len() >= std::mem::size_of::<PacketLog>() {
                            let pl: PacketLog =
                                unsafe { std::ptr::read(data.as_ptr() as *const _) };
                            let src = Ipv4Addr::from(u32::from_be(pl.src_ip));
                            let dst = Ipv4Addr::from(u32::from_be(pl.dst_ip));
                            let src_port = u16::from_be(pl.src_port as u16);
                            let dst_port = u16::from_be(pl.dst_port as u16);
                            let event_id = pl.event_id;

                            match IpProtocols::try_from(pl.proto) {
                                Ok(proto) => {
                                    info!(
                                        "Event Id: {} Protocol: {:?} SRC: {}:{} -> DST: {}:{}",
                                        event_id, proto, src, src_port, dst, dst_port
                                    );
                                }
                                Err(_) => {
                                    info!("Event Id: {} Protocol: Unknown ({})", event_id, pl.proto)
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
                Ok(events) => {
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
                                Ok(veth_name) => {
                                    info!(
                                        "Triggered action: register_netdevice event_type:{:?} Manipulated veth: {:?} state:{:?} dev_addr:{:?}",
                                        event_type,
                                        veth_name.trim_end_matches("\0").to_string(),
                                        state,
                                        dev_addr
                                    );
                                    attach_detach_veth(bpf.clone(), vethlog.event_type, veth_name, link_ids.clone()).await;
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
            if !ignore_iface(&iface_name)
            {
                interfaces.push(iface_name);
            } else {
                info!("skipping interface {:?}", iface_name);
            }
        }
    }

    interfaces
}

async fn attach_detach_veth(bpf: Arc<Mutex<Bpf>>, event_type: u8, iface: &str, link_ids: Arc<Mutex<HashMap<String, SchedClassifierLinkId>>>) -> Result<(), anyhow::Error> {
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
                Ok(link_id) => {
                    info!("Program 'identity_classifier' attached to interface {}", iface);
                    link_ids.insert(iface.to_string(), link_id);

                },
                Err(e) => error!("Error attaching program to interface {}: {:?}", iface, e),
            }
        }
        2 => {
            // INFO: Detaching occurs automatically when veth is deleted by kernel itsel
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