use aya::{ maps::{ perf::{ PerfEventArrayBuffer }, MapData } };
use crate::structs::PacketLog;
use bytes::BytesMut;
use std::{ borrow::BorrowMut, net::Ipv4Addr, sync::{ atomic::{ AtomicBool, Ordering }, Arc } };
use crate::enums::IpProtocols;
use tracing::{ info, error, warn };

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
                                        "Hash: {} Protocol: {:?} SRC: {}:{} -> DST: {}:{}",
                                        event_id,
                                        proto,
                                        src,
                                        src_port,
                                        dst,
                                        dst_port
                                    );
                                }
                                Err(_) =>
                                    info!("Hash: {} Protocol: Unknown ({})", event_id, pl.proto),
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
