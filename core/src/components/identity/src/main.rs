/*
 * CortexBrain Identity Service
 * Features:
 *   1. TCP, UDP , ICMP events tracker
 *   2. Track Connections using a PerfEventArray named ConnArray
 *
 */
#![allow(warnings)]
#![allow(unused_mut)]

mod helpers;
mod structs;
mod enums;
use aya::{
    maps::{ perf::{ PerfEventArray, PerfEventArrayBuffer }, MapData },
    programs::{ SchedClassifier, TcAttachType },
    util::online_cpus,
    Bpf,
};

use bytes::BytesMut;
use std::{
    convert::TryInto,
    sync::{ atomic::{ AtomicBool, Ordering }, Arc },
    path::Path,
};
use crate::helpers::display_events;
use crate::enums::IpProtocols;

use tokio::{ signal, fs };
use anyhow::Context;
use tracing_subscriber::{ fmt::format::FmtSpan, EnvFilter };
use tracing::{ info, error, warn };


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
const BPF_PATH: &str = "BPF_PATH";
const IFACE: &str = "IFACE";

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    //init tracing subscriber
    tracing_subscriber
        ::fmt()
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
    let bpf_path = std::env::var(BPF_PATH).context("BPF_PATH environment variable required")?;
    let data = fs::read(Path::new(&bpf_path)).await.context("failed to load file from path")?;

    //init bpf data
    let mut bpf = Bpf::load(&data)?;

    info!("Loading programs");

    let program: &mut SchedClassifier = bpf
        .program_mut("identity_classifier")
        .ok_or_else(|| anyhow::anyhow!("program 'identity_classifier' not found"))?
        .try_into()?;
    program.load()?;
    let iface = std::env::var(IFACE).context("Interface var 'IFACE' not found")?;
    program.attach(&iface, TcAttachType::Ingress)?;

    //init events map
    let events_map = bpf
        .take_map("EventsMap")
        .ok_or_else(|| anyhow::anyhow!("EventsMap map not found"))?;

    info!("loading bpf connections map");

    //init connection map
    let connections_map_raw = bpf
        .take_map("ConnectionMap")
        .context("failed to take connections map")?;

    let connection_tracker_map = bpf
        .take_map("ConnectionTrackerMap")
        .context("failed to take ConnectionTrackerMap map")?;

    //pinning connections map
    //  if Path::new("/sys/fs/bpf/connections").exists() {
    //  warn!("map already pinned, skipping process");
    //} else {
    //  connections_map_raw.pin("/sys/fs/bpf/connections").context("failed to pin map")?;
    //}

    // init PerfEventArrays
    let mut perf_array: PerfEventArray<MapData> = PerfEventArray::try_from(events_map)?;
    /*     let mut connections_perf_array = PerCpuHashMap::<&mut MapData,u8,ConnArray>::try_from(connections_map_raw)?; //change with lru hash map*/
    //init PerfEventArrays buffers
    let mut perf_buffers: Vec<PerfEventArrayBuffer<MapData>> = Vec::new();
    /*     let mut connections_perf_buffers = Vec::new(); */

    for cpu_id in online_cpus().map_err(|e| anyhow::anyhow!("Error {:?}", e))? {
        let buf: PerfEventArrayBuffer<MapData> = perf_array.open(cpu_id, None)?;
        perf_buffers.push(buf);

        //   conn_buf = connections_perf_array.open(cpu_id, None)?;
        //connections_perf_buffers.push(conn_buf);
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
    //   let mut connections_buffers = vec![BytesMut::with_capacity(1024); 10];

    display_events(perf_buffers, running, buffers).await;

    info!("Exiting...");
    Ok(())
}
