use aya::{
    maps::{ perf::{ PerfEventArray, PerfEventArrayBuffer }, MapData },
    programs::{ SchedClassifier, TcAttachType },
    util::online_cpus,
    Bpf,
};
use crate::structs::PacketLog;
use bytes::BytesMut;
use std::{
    borrow::BorrowMut,
    net::Ipv4Addr,
    string,
    sync::{ atomic::{ AtomicBool, Ordering }, Arc },
};
use crate::enums::IpProtocols;
use tracing::{ info, error, warn };
use nix::net::if_::if_nameindex;

use tokio::{ fs, signal };
use std::path::Path;
use anyhow::Context;
/*
 * decleare bpf path env variable
 */
const BPF_PATH: &str = "BPF_PATH";
const IFACE: &str = "IFACE";

pub async fn display_events<T: BorrowMut<MapData>>(
    mut perf_buffers: Vec<PerfEventArrayBuffer<T>>,
    running: Arc<AtomicBool>,
    mut buffers: Vec<BytesMut>
) {
    while running.load(Ordering::SeqCst) {
        for buf in perf_buffers.iter_mut() {
            match buf.read_events(&mut buffers) {
                Ok(events) => {
                    for i in 0..events.read {
                        let data = &buffers[i];
                        if data.len() >= std::mem::size_of::<PacketLog>() {
                            let pl: PacketLog = unsafe {
                                std::ptr::read(data.as_ptr() as *const _)
                            };
                            let src = Ipv4Addr::from(u32::from_be(pl.src_ip));
                            let dst = Ipv4Addr::from(u32::from_be(pl.dst_ip));
                            let src_port = u16::from_be(pl.src_port as u16);
                            let dst_port = u16::from_be(pl.dst_port as u16);
                            let event_id = pl.event_id;

                            match IpProtocols::try_from(pl.proto) {
                                Ok(proto) => {
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
                                Err(_) =>
                                    info!("Event Id: {} Protocol: Unknown ({})", event_id, pl.proto),
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

//filter the interfaces,exclude docker0,eth0,lo interfaces
pub fn get_veth_channels() -> Vec<String> {
    //filter interfaces and save the output in the
    let mut interfaces: Vec<String> = Vec::new();

    if let Ok(ifaces) = if_nameindex() {
        for iface in &ifaces {
            let iface_name = iface.name().to_str().unwrap().to_owned();
            if
                iface_name != "eth0" &&
                iface_name != "docker0" &&
                iface_name != "tunl0" &&
                iface_name != "lo"
            {
                interfaces.push(iface_name);
            } else {
                info!("skipping interface");
            }
        }
    }

    interfaces
}
