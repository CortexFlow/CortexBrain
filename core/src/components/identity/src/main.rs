/* EDIT: TODO: this part needs an update
 * CortexBrain Identity Service
 * Features:
 *   1. TCP, UDP , ICMP events tracker
 *   2. Track Connections using a PerfEventArray named ConnArray
 *   3. Track veth creation and deletion events
 *
 */
#![allow(warnings)]
#![allow(unused_mut)]

mod enums;
mod helpers;
mod structs;
use aya::{
    Bpf, Ebpf,
    maps::{
        Map, MapData,
        perf::{PerfEventArray, PerfEventArrayBuffer},
    },
    programs::{KProbe, SchedClassifier, TcAttachType},
    util::online_cpus,
};

use crate::enums::IpProtocols;
use crate::helpers::{display_events, display_veth_events, get_veth_channels};
use bytes::BytesMut;
use std::{
    convert::TryInto,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use anyhow::{Context, Ok};
use tokio::{fs, signal, sync::broadcast::error};
use tracing::{error, info, warn};
use tracing_subscriber::{EnvFilter, fmt::format::FmtSpan};

const BPF_PATH: &str = "BPF_PATH"; //BPF env path

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    //init tracing subscriber
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_level(true)
        .with_span_events(FmtSpan::NONE)
        .with_file(false)
        .pretty()
        .with_env_filter(EnvFilter::new("info"))
        .with_line_number(false)
        .init();

    info!("Starting identity service...");
    info!("fetching data");

    //init conntracker data path
    let bpf_path = std::env::var(BPF_PATH).context("BPF_PATH environment variable required")?;
    let data = fs::read(Path::new(&bpf_path))
        .await
        .context("failed to load file from path")?;

    //init bpf data
    let mut bpf = Bpf::load(&data)?;

    //load veth_trace program ref veth_trace.rs
    init_veth_tracer(&mut bpf);
    let bpf_maps = init_bpf_maps(&mut bpf).unwrap();

    let interfaces = get_veth_channels();

    //TODO: store the results from the veth_tracer in a hashmap (InterfacesRegistry) to make sure that the creation and deletion of veth are up to date
    // everytime a new interface enters the InterfacesRegistry attach a bpf program with the attach_bpf_program function below

    info!("Found interfaces: {:?}", interfaces);
    init_tc_classifier(&mut bpf, interfaces)
        .context("An error occured during the execution of attach_bpf_program function")?;

    event_listener(bpf_maps)
        .await
        .context("Error initializing event_listener")?;

    Ok(())
}

//attach the tc classifier program to a vector of interfaces
pub fn init_tc_classifier(bpf: &mut Ebpf, ifaces: Vec<String>) -> Result<(), anyhow::Error> {
    //this funtion initialize the tc classifier program
    info!("Loading programs");

    let program: &mut SchedClassifier = bpf
        .program_mut("identity_classifier")
        .ok_or_else(|| anyhow::anyhow!("program 'identity_classifier' not found"))?
        .try_into()
        .context("Failed to init SchedClassifier program")?;

    program
        .load()
        .context("Failed to load identity_classifier program")?;

    for interface in ifaces {
        match program.attach(&interface, TcAttachType::Ingress) {
            std::result::Result::Ok(_) => info!(
                "Program 'identity_classifier' attached to interface {}",
                interface
            ),
            Err(e) => error!(
                "Error attaching program to interface {}: {:?}",
                interface, e
            ),
        }
    }

    Ok(())
}
fn init_veth_tracer(bpf: &mut Ebpf) -> Result<(), anyhow::Error> {
    //this functions init the veth_tracer used to make the InterfacesRegistry

    //creation tracer
    let veth_creation_tracer: &mut KProbe = bpf
        .program_mut("veth_creation_trace")
        .ok_or_else(|| anyhow::anyhow!("program 'veth_creation_trace' not found"))?
        .try_into()?;
    veth_creation_tracer.load()?;

    veth_creation_tracer.attach("register_netdevice", 0)?;

    //deletion tracer
    let veth_deletion_tracer: &mut KProbe = bpf
        .program_mut("veth_deletion_trace")
        .ok_or_else(|| anyhow::anyhow!("program 'veth_deletion_trace' not found"))?
        .try_into()?;
    veth_deletion_tracer
        .load()
        .context("Failed to load deletetion_tracer program")?;

    match veth_deletion_tracer.attach("unregister_netdevice_queue", 0) {
        std::result::Result::Ok(_) => info!("veth_deletion_trace program attached successfully"),
        Err(e) => error!("Error attaching veth_deletetion_trace program {:?}", e),
    }

    Ok(())
}

fn init_bpf_maps(bpf: &mut Ebpf) -> Result<(Map, Map), anyhow::Error> {
    // this function init the bpfs maps used in the main program
    /*
       index 0: events_map
       index 1: veth_map
    */
    let events_map = bpf
        .take_map("EventsMap")
        .ok_or_else(|| anyhow::anyhow!("EventsMap map not found"))?;

    let veth_map = bpf
        .take_map("veth_identity_map")
        .ok_or_else(|| anyhow::anyhow!("veth_identity_map map not found"))?;

    /* EDIT: this part is paused right now
       info!("loading bpf connections map");

       //init connection map
       let connections_map_raw = bpf
           .take_map("ConnectionMap")
           .context("failed to take connections map")?;

       let connection_tracker_map = bpf
           .take_map("ConnectionTrackerMap")
           .context("failed to take ConnectionTrackerMap map")?;
    */
    Ok((events_map, veth_map))
}

async fn event_listener(bpf_maps: (Map, Map)) -> Result<(), anyhow::Error> {
    // this function init the event listener. Listens for veth events (creation/deletion) and network events (pod to pod communications)
    /* Doc:

       perf_net_events_array: contains is associated with the network events stored in the events_map (EventsMap)
       perf_veth_array: contains is associated with the network events stored in the veth_map (veth_identity_map)

    */

    info!("Preparing perf_buffers and perf_arrays");
    // init PerfEventArrays
    let mut perf_veth_array: PerfEventArray<MapData> = PerfEventArray::try_from(bpf_maps.1)?;
    let mut perf_net_events_array: PerfEventArray<MapData> = PerfEventArray::try_from(bpf_maps.0)?;
    /*     let mut connections_perf_array = PerCpuHashMap::<&mut MapData,u8,ConnArray>::try_from(connections_map_raw)?; //change with lru hash map*/
    //init PerfEventArrays buffers
    let mut perf_veth_buffer: Vec<PerfEventArrayBuffer<MapData>> = Vec::new();
    let mut perf_net_events_buffer: Vec<PerfEventArrayBuffer<MapData>> = Vec::new();
    /*     let mut connections_perf_buffers = Vec::new(); */

    for cpu_id in online_cpus().map_err(|e| anyhow::anyhow!("Error {:?}", e))? {
        let veth_buf: PerfEventArrayBuffer<MapData> = perf_veth_array.open(cpu_id, None)?;
        perf_veth_buffer.push(veth_buf);
    }
    for cpu_id in online_cpus().map_err(|e| anyhow::anyhow!("Error {:?}", e))? {
        let events_buf: PerfEventArrayBuffer<MapData> = perf_net_events_array.open(cpu_id, None)?;
        perf_net_events_buffer.push(events_buf);
    }
    info!("Listening for events...");

    // FIXME: There seem to be a concurrency error that is causing the pod to pod logs to not work at all
    let veth_running = Arc::new(AtomicBool::new(true));
    let net_events_running = Arc::new(AtomicBool::new(true));

    let mut veth_buffers = vec![BytesMut::with_capacity(1024); 10];
    let mut events_buffers = vec![BytesMut::with_capacity(1024); online_cpus().iter().len()];
    //   let mut connections_buffers = vec![BytesMut::with_capacity(1024); 10];

    let veth_running_signal = veth_running.clone();
    let net_events_running_signal = net_events_running.clone();

    //display_events(perf_buffers, running, buffers).await;
    let veth_events_displayer = tokio::spawn(async move {
        display_veth_events(perf_veth_buffer, veth_running, veth_buffers).await;
    });
    let net_events_displayer = tokio::spawn(async move {
        display_events(perf_net_events_buffer, net_events_running, events_buffers).await;
    });

    tokio::select! {
        result = veth_events_displayer=>{
            match result{
                Err(e)=>error!("veth_event_displayer panicked {:?}",e),
                std::result::Result::Ok(_) => info!("Found new veth_event"),
                }
        }

        result = net_events_displayer=>{
            match result{
                Err(e)=>error!("net_event_displayer panicked {:?}",e),
                std::result::Result::Ok(_)  => info!("Found new net_event"),
            }
        }
        _= signal::ctrl_c()=>{
            info!("Triggered Exiting...");
            veth_running_signal.store(false, Ordering::SeqCst);
            net_events_running_signal.store(false, Ordering::SeqCst);
        }

    }

    Ok(())
}
