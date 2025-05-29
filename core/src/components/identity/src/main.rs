/* 
    * CortexBrain Identity Service
    * Features: 
    *   1. TCP, UDP , ICMP events tracker
    *   2. Track Connections using a PerfEventArray named ConnArray
    *
*/

use aya::Bpf;
use aya::programs::{SchedClassifier, TcAttachType};
use aya::maps::perf::PerfEventArray;
use aya::maps::PerCpuHashMap;
use aya::maps::MapData;
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

/* 
    * Structure PacketLog
    * This structure is used to store the packet information
*/
#[repr(C)]
#[derive(Clone, Copy)]
struct PacketLog {
    proto: u8,
    src_ip: u32,
    src_port: u16,
    dst_ip: u32,
    dst_port: u16,
    event_id: u16,
}

/* 
    * Connection Array that contains the hash_id associated with an active connection
*/
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ConnArray {
    pub event_id: u16,
    pub connection_id: u16,
}

unsafe impl aya::Pod for ConnArray {}

/* 
    * IpProtocols enum to reconstruct the packet protocol based on the 
    * IPV4 Header Protocol code 
*/
#[derive(Debug)]
#[repr(u8)]
enum IpProtocols {
    ICMP = 1,
    TCP = 6,
    UDP = 17,
}

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

/* 
    * decleare bpf path env variable
*/
const BPF_PATH : &str = "BPF_PATH";

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    //init tracing subscriber
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
    
    //init conntracker data path 
    let bpf_path= std::env::var(BPF_PATH).context("BPF_PATH environment variable required")?;
    let data = fs::read(Path::new(&bpf_path)).await.context("failed to load file from path")?;
    
    //init bpf data 
    let mut bpf = Bpf::load(&data)?;
    
    info!("Loading programs");
    {
        let program: &mut SchedClassifier = bpf
            .program_mut("identity_classifier")
            .ok_or_else(|| anyhow::anyhow!("program 'identity_classifier' not found"))?
            .try_into()?;
        program.load()?;
        program.attach("eth0", TcAttachType::Ingress)?; //check with ipconfig a the source of traffic
    }

    //init events map 
    let events_map = bpf
        .take_map("EventsMap")
        .ok_or_else(|| anyhow::anyhow!("EventsMap map not found"))?;
    
    info!("loading bpf connections map");
    
    //init connection map 
    let connections_map_raw = bpf
        .take_map("ConnectionMap")
        .context("failed to take connections map")?;
    
    //pinning connections map 
    if Path::new("/sys/fs/bpf/connections").exists() {
        warn!("map already pinned, skipping process");
    } else {
        connections_map_raw.pin("/sys/fs/bpf/connections")
            .context("failed to pin map")?;
    }

    // init PerfEventArrays 
    let mut perf_array = PerfEventArray::try_from(events_map)?;
/*     let mut connections_perf_array = PerCpuHashMap::<&mut MapData,u8,ConnArray>::try_from(connections_map_raw)?; //change with lru hash map
 */
    //init PerfEventArrays buffers
    let mut perf_buffers = Vec::new();
/*     let mut connections_perf_buffers = Vec::new(); */

    for cpu_id in online_cpus().map_err(|e| anyhow::anyhow!("Error {:?}", e))? {
        let buf = perf_array.open(cpu_id, None)?;
        perf_buffers.push(buf);
        
/*         let conn_buf = connections_perf_array.open(cpu_id, None)?;
        connections_perf_buffers.push(conn_buf); */
    }

    info!("Listening for events...");
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    
    //waiting for signint (CTRL+C) to stop the main program
    tokio::spawn(async move {
        signal::ctrl_c().await.unwrap();
        r.store(false, Ordering::SeqCst);
    });

    let mut buffers = vec![BytesMut::with_capacity(1024); 10]; 
/*     let mut connections_buffers = vec![BytesMut::with_capacity(1024); 10]; */

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
                            let event_id = pl.event_id;

                            match IpProtocols::try_from(pl.proto) {
                                Ok(proto) => {
                                    info!("Hash: {} Protocol: {:?} SRC: {}:{} -> DST: {}:{}", 
                                    event_id, proto, src, src_port, dst, dst_port);
                                },
                                Err(_) => info!("Hash: {} Protocol: Unknown ({})", event_id, pl.proto),
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
    //print out Connection Events
/*     while running.load(Ordering::SeqCst) {
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
    } */

    info!("Exiting...");
    Ok(())
}