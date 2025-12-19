/*
 * CortexBrain Identity Service
 * Open Issues: #105 #107
 * Features:
 *   1. TCP, UDP , ICMP events tracker
 *   2. Track Connections using a PerfEventArray named ConnArray
 *   3. Track veth creation and deletion events
 *
 */
#![allow(unused_mut)]
#![allow(warnings)]

mod enums;
mod helpers;
mod structs;
mod map_handlers;

use aya::{
    Ebpf, maps::{ Map, MapData, perf::{ PerfEventArray, PerfEventArrayBuffer } }, programs::{ KProbe, SchedClassifier, TcAttachType, tc::SchedClassifierLinkId }, util::online_cpus
};

use crate::helpers::{
    display_events,
    display_veth_events,
    get_veth_channels,
    display_tcp_registry_events,
    scan_cgroup_cronjob
};
use crate::map_handlers::{ init_bpf_maps, map_pinner, populate_blocklist };

use bytes::BytesMut;
use std::{ convert::TryInto, path::Path, sync::{ Arc, Mutex, atomic::{ AtomicBool, Ordering } } };

use anyhow::{ Context, Ok };
use tokio::{ fs, signal };
use tracing::{ error, info };
use cortexbrain_common::{ constants, logger };

use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    //init tracing subscriber
    logger::init_default_logger();

    info!("Starting identity service...");
    info!("fetching data");

    // To Store link_ids they can be used to detach tc
    let link_ids = Arc::new(Mutex::new(HashMap::<String, SchedClassifierLinkId>::new()));

    //init conntracker data path
    let bpf_path = std::env
        ::var(constants::BPF_PATH)
        .context("BPF_PATH environment variable required")?;
    let data = fs::read(Path::new(&bpf_path)).await.context("failed to load file from path")?;

    //init bpf data
    let bpf = Arc::new(Mutex::new(Ebpf::load(&data)?));
    let bpf_map_save_path = std::env
        ::var(constants::PIN_MAP_PATH)
        .context("PIN_MAP_PATH environment variable required")?;

    match init_bpf_maps(bpf.clone()) {
        std::result::Result::Ok(mut bpf_maps) => {
            info!("Successfully loaded bpf maps");
            let pin_path = std::path::PathBuf::from(&bpf_map_save_path);
            info!("About to call map_pinner with path: {:?}", pin_path);
            match map_pinner(&bpf_maps, &pin_path) {
                std::result::Result::Ok(_) => {
                    info!("maps pinned successfully");
                    //load veth_trace program ref veth_trace.rs
                    {
                        init_veth_tracer(bpf.clone()).await?;
                    }

                    let interfaces = get_veth_channels();

                    info!("Found interfaces: {:?}", interfaces);

                    {
                        populate_blocklist(&mut bpf_maps.2).await;
                    }

                    //{
                    //    init_tc_classifier(bpf.clone(), interfaces, link_ids.clone()).await.context(
                    //        "An error occured during the execution of attach_bpf_program function"
                    //    )?;
                    //}
                    {
                        init_tcp_registry(bpf.clone()).await.context(
                            "An error occured during the execution of init_tcp_registry function"
                        )?;
                    }

                    event_listener(bpf_maps, link_ids.clone(), bpf.clone()).await.context(
                        "Error initializing event_listener"
                    )?;
                }
                Err(e) => {
                    error!("Error while pinning bpf_maps: {}", e);
                }
            }
        }
        Err(e) => {
            error!("Error while loading bpf maps {}", e);
            signal::ctrl_c();
        }
    }

    Ok(())
}

//attach the tc classifier program to a vector of interfaces
async fn init_tc_classifier(
    bpf: Arc<Mutex<Ebpf>>,
    ifaces: Vec<String>,
    link_ids: Arc<Mutex<HashMap<String, SchedClassifierLinkId>>>
) -> Result<(), anyhow::Error> {
    //this funtion initialize the tc classifier program
    info!("Loading programs");

    let mut bpf_new = bpf.lock().unwrap();

    let program: &mut SchedClassifier = bpf_new
        .program_mut("identity_classifier")
        .ok_or_else(|| anyhow::anyhow!("program 'identity_classifier' not found"))?
        .try_into()
        .context("Failed to init SchedClassifier program")?;

    program.load().context("Failed to load identity_classifier program")?;

    for interface in ifaces {
        match program.attach(&interface, TcAttachType::Ingress) {
            std::result::Result::Ok(link_id) => {
                info!("Program 'identity_classifier' attached to interface {}", interface);
                let mut map = link_ids.lock().unwrap();
                map.insert(interface.clone(), link_id);
            }
            Err(e) => error!("Error attaching program to interface {}: {:?}", interface, e),
        }
    }

    Ok(())
}

async fn init_veth_tracer(bpf: Arc<Mutex<Ebpf>>) -> Result<(), anyhow::Error> {
    //this functions init the veth_tracer used to make the InterfacesRegistry

    let mut bpf_new = bpf.lock().unwrap();

    //creation tracer
    let veth_creation_tracer: &mut KProbe = bpf_new
        .program_mut("veth_creation_trace")
        .ok_or_else(|| anyhow::anyhow!("program 'veth_creation_trace' not found"))?
        .try_into()?;
    veth_creation_tracer.load()?;

    match veth_creation_tracer.attach("register_netdevice", 0) {
        std::result::Result::Ok(_) => info!("veth_creation_tracer program attached successfully"),
        Err(e) => error!("Error attaching veth_creation_tracer program {:?}", e),
    }

    //deletion tracer
    let veth_deletion_tracer: &mut KProbe = bpf_new
        .program_mut("veth_deletion_trace")
        .ok_or_else(|| anyhow::anyhow!("program 'veth_deletion_trace' not found"))?
        .try_into()?;
    veth_deletion_tracer.load().context("Failed to load deletetion_tracer program")?;

    match veth_deletion_tracer.attach("unregister_netdevice_queue", 0) {
        std::result::Result::Ok(_) => info!("veth_deletion_trace program attached successfully"),
        Err(e) => error!("Error attaching veth_deletetion_trace program {:?}", e),
    }

    Ok(())
}

async fn init_tcp_registry(bpf: Arc<Mutex<Ebpf>>) -> Result<(), anyhow::Error> {
    let mut bpf_new = bpf.lock().unwrap();

    // init tcp registry
    let tcp_analyzer: &mut KProbe = bpf_new
        .program_mut("tcp_message_tracer")
        .ok_or_else(|| anyhow::anyhow!("program 'tcp_message_tracer' not found"))?
        .try_into()?;

    tcp_analyzer.load().context("Failed to load tcp_message_tracer")?;

    info!("initializing tcp tracing functions");

    match tcp_analyzer.attach("tcp_v4_rcv", 0) {
        std::result::Result::Ok(_) =>
            info!("tcp_message_tracer attached successfully to the tcp_v4_rcv function "),
        Err(e) =>
            error!("Error attaching tcp_message_tracer to the tcp_v4_rcv function. Error: {:?}", e),
    }

    match tcp_analyzer.attach("tcp_v4_connect", 0) {
        std::result::Result::Ok(_) =>
            info!("tcp_message_tracer attached successfully to the tcp_v4_connect function "),
        Err(e) =>
            error!(
                "Error attaching tcp_message_tracer to the tcp_v4_connect function. Error: {:?}",
                e
            ),
    }

    Ok(())
}

async fn event_listener(
    bpf_maps: (Map, Map, Map, Map),
    link_ids: Arc<Mutex<HashMap<String, SchedClassifierLinkId>>>,
    bpf: Arc<Mutex<Ebpf>>
) -> Result<(), anyhow::Error> {
    // this function init the event listener. Listens for veth events (creation/deletion) and network events (pod to pod communications)
    /* Doc:

       perf_net_events_array: contains is associated with the network events stored in the events_map (EventsMap)
       perf_veth_array: contains is associated with the network events stored in the veth_map (veth_identity_map)

    */

    info!("Preparing perf_buffers and perf_arrays");

    //TODO: try to change from PerfEventArray to a RingBuffer data structure
    //let m0=bpf_maps[0];
    //let m1 = bpf_maps[1];
    //let mut ring1=RingBuf::try_from(m0)?;
    //let mut ring2=RingBuf::try_from(m1)?;

    //TODO:create an helper function that initialize the data structures and the running
    // init PerfEventArrays
    let mut perf_veth_array: PerfEventArray<MapData> = PerfEventArray::try_from(bpf_maps.1)?;
    let mut perf_net_events_array: PerfEventArray<MapData> = PerfEventArray::try_from(bpf_maps.0)?;
    let mut tcp_registry_array: PerfEventArray<MapData> = PerfEventArray::try_from(bpf_maps.3)?;

    // init PerfEventArrays buffers
    let mut perf_veth_buffer: Vec<PerfEventArrayBuffer<MapData>> = Vec::new();
    let mut perf_net_events_buffer: Vec<PerfEventArrayBuffer<MapData>> = Vec::new();
    let mut tcp_registry_buffer: Vec<PerfEventArrayBuffer<MapData>> = Vec::new();

    // fill the input buffers

    for cpu_id in online_cpus().map_err(|e| anyhow::anyhow!("Error {:?}", e))? {
        let veth_buf: PerfEventArrayBuffer<MapData> = perf_veth_array.open(cpu_id, None)?;
        perf_veth_buffer.push(veth_buf);
    }
    for cpu_id in online_cpus().map_err(|e| anyhow::anyhow!("Error {:?}", e))? {
        let events_buf: PerfEventArrayBuffer<MapData> = perf_net_events_array.open(cpu_id, None)?;
        perf_net_events_buffer.push(events_buf);
    }
    for cpu_id in online_cpus().map_err(|e| anyhow::anyhow!("Error {:?}", e))? {
        let tcp_registry_buf: PerfEventArrayBuffer<MapData> = tcp_registry_array.open(
            cpu_id,
            None
        )?;
        tcp_registry_buffer.push(tcp_registry_buf);
    }

    info!("Listening for events...");

    // init runnings
    let veth_running = Arc::new(AtomicBool::new(true));
    let net_events_running = Arc::new(AtomicBool::new(true));
    let tcp_registry_running = Arc::new(AtomicBool::new(true));

    // init output buffers
    let mut veth_buffers = vec![BytesMut::with_capacity(1024); 10];
    let mut events_buffers = vec![BytesMut::with_capacity(1024); online_cpus().iter().len()];
    let mut tcp_buffers = vec![BytesMut::with_capacity(1024); online_cpus().iter().len()];

    // init running signals
    let veth_running_signal = veth_running.clone();
    let net_events_running_signal = net_events_running.clone();
    let tcp_registry_running_signal = tcp_registry_running.clone();

    let veth_link_ids = link_ids.clone();

    let veth_events_displayer = tokio::spawn(async move {
        display_veth_events(
            bpf.clone(),
            perf_veth_buffer,
            veth_running,
            veth_buffers,
            veth_link_ids
        ).await;
    });

    //let net_events_displayer = tokio::spawn(async move {
    //    display_events(perf_net_events_buffer, net_events_running, events_buffers).await;
    //});

    let tcp_registry_events_displayer: tokio::task::JoinHandle<()> = tokio::spawn(async move {
        display_tcp_registry_events(tcp_registry_buffer, tcp_registry_running, tcp_buffers).await;
    });

    let scan_cgroup_cronjob = tokio::spawn(async move {
        let _ = scan_cgroup_cronjob(180).await;
    });

    tokio::select! {
        result = scan_cgroup_cronjob=>{
            match result{
                Err(e)=>error!("scan_cgroup_cronjob panicked {:?}",e),
                std::result::Result::Ok(_) => info!("cgroup scan cronjob exited"),
                }
            }
        result = veth_events_displayer=>{
            match result{
                Err(e)=>error!("veth_event_displayer panicked {:?}",e),
                std::result::Result::Ok(_) => info!("Found new veth_event"),
                }
        }

        //result = net_events_displayer=>{
        //    match result{
        //        Err(e)=>error!("net_event_displayer panicked {:?}",e),
        //        std::result::Result::Ok(_)  => info!("Found new net_event"),
        //    }
        //}
        
        result = tcp_registry_events_displayer => {
            match result{
                Err(e)=>error!("tcp_registry_events_displayer panicked {:?}",e),
                std::result::Result::Ok(_)=>info!("Found new tcp_register event")
            }
        }

        _= signal::ctrl_c()=>{
            info!("Triggered Exiting...");
            veth_running_signal.store(false, Ordering::SeqCst);
            net_events_running_signal.store(false, Ordering::SeqCst);
            tcp_registry_running_signal.store(false, Ordering::SeqCst);
        }

    }

    Ok(())
}
