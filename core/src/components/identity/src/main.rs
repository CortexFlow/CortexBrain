use aya::Bpf;
use aya::programs::{SchedClassifier, TcAttachType};
use aya::maps::perf::PerfEventArray;
use aya::util::online_cpus;
use bytes::BytesMut;
use std::convert::TryInto;
use std::net::Ipv4Addr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::signal;
use anyhow::Context;
use tokio::fs;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::EnvFilter;
use tracing::{info, error, warn};
use std::path::Path;

#[repr(C)]
#[derive(Clone, Copy)]
struct PacketLog {
    proto: u8,
    src_ip: u32,
    src_port: u32,
    dst_ip: u32,
    dst_port: u32,
    hash_id: u16,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ConnArray {
    pub hash_id: u16,
}

unsafe impl aya::Pod for ConnArray {}

#[derive(Debug)]
#[repr(u8)]
enum IpProtocols {
    ICMP = 1,
    TCP = 6,
    UDP = 17,
}

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

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_level(true)
        .with_span_events(FmtSpan::NONE)
        .without_time()
        .with_file(false)
        .pretty()
        .with_env_filter(EnvFilter::new("info"))
        .with_line_number(false)
        .init();

    info!("Starting identity service...");
    info!("fetching data");
    
    let data = fs::read("../../../target/bpfel-unknown-none/release/conntracker")
        .await
        .context("Failed to load BPF object data")?;
    
    let mut bpf = Bpf::load(&data)?;
    
    info!("Loading programs");
    {
        let program: &mut SchedClassifier = bpf
            .program_mut("identity_classifier")
            .ok_or_else(|| anyhow::anyhow!("program 'identity_classifier' not found"))?
            .try_into()?;
        program.load()?;
        program.attach("enp0s25", TcAttachType::Ingress)?;
    }

    let events_map = bpf
        .take_map("EVENTS")
        .ok_or_else(|| anyhow::anyhow!("EVENTS map not found"))?;
    
    info!("loading bpf connections map");
    let connections_map_raw = bpf
        .take_map("ConnectionArray")
        .context("failed to take connections map")?;
    
    if Path::new("/sys/fs/bpf/connections").exists() {
        warn!("map already pinned, skipping process");
    } else {
        connections_map_raw.pin("/sys/fs/bpf/connections")
            .context("failed to pin map")?;
    }

    let mut perf_array = PerfEventArray::try_from(events_map)?;
    let mut connections_perf_array = PerfEventArray::try_from(connections_map_raw)?;

    let mut perf_buffers = Vec::new();
    let mut connections_perf_buffers = Vec::new();

    for cpu_id in online_cpus().map_err(|e| anyhow::anyhow!("Error {:?}", e))? {
        let buf = perf_array.open(cpu_id, None)?;
        perf_buffers.push(buf);
        
        let conn_buf = connections_perf_array.open(cpu_id, None)?;
        connections_perf_buffers.push(conn_buf);
    }

    info!("Listening for events...");
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    
    tokio::spawn(async move {
        signal::ctrl_c().await.unwrap();
        r.store(false, Ordering::SeqCst);
    });

    let mut buffers = vec![BytesMut::with_capacity(1024); 10];
    let mut connections_buffers = vec![BytesMut::with_capacity(1024); 10];

    /* 
    while running.load(Ordering::SeqCst) {
        for buf in &mut perf_buffers {
            match buf.read_events(&mut buffers) {
                Ok(events) => {
                    for i in 0..events.read {
                        let data = &buffers[i];
                        if data.len() >= std::mem::size_of::<PacketLog>() {
                            let pl: PacketLog = unsafe { std::ptr::read(data.as_ptr() as *const _) };
                            let src = Ipv4Addr::from(u32::from_be(pl.src_ip));
                            let dst = Ipv4Addr::from(u32::from_be(pl.dst_ip));
                            let src_port = u16::from_be(pl.src_port as u16);
                            let dst_port = u16::from_be(pl.dst_port as u16);
                            let hash = pl.hash_id;
                            
                            match IpProtocols::try_from(pl.proto) {
                                Ok(proto) => {
                                    info!("Hash: {} Protocol: {:?} SRC: {}:{} -> DST: {}:{}", 
                                          hash, proto, src, src_port, dst, dst_port);
                                },
                                Err(_) => info!("Hash: {} Protocol: Unknown ({})", hash, pl.proto),
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
    */ 

    while running.load(Ordering::SeqCst) {
        for buf in &mut connections_perf_buffers {
            match buf.read_events(&mut connections_buffers) {
                Ok(events) => {
                    for i in 0..events.read {
                        let data = &connections_buffers[i];
                        if data.len() >= std::mem::size_of::<ConnArray>() {
                            let conn: ConnArray = unsafe { std::ptr::read(data.as_ptr() as *const _) };
                            info!("Connection Hash ID from map: {}", conn.hash_id);
                        } else {
                            warn!("Received connection data too small: {} bytes", data.len());
                        }
                    }
                }
                Err(e) => {
                    error!("Error reading connection events: {:?}", e);
                }
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    info!("Exiting...");
    Ok(())
}