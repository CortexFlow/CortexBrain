/*
 * CortexBrain Identity Service
 * Features:
 *   1. TCP events tracker
 *   2. veth creation and deletion tracker
 *   3. TC (traffic control) tracker
 *   4. [Experimental]: cgroup scanner
 *
 */

mod enums;
mod helpers;
mod structs;

use crate::helpers::{
    display_events, display_tcp_registry_events, display_veth_events, get_veth_channels,
};
use aya::{
    Ebpf,
    maps::{Map, perf::PerfEventArray},
    programs::{SchedClassifier, TcAttachType, tc::SchedClassifierLinkId},
    util::online_cpus,
};

#[cfg(feature = "experimental")]
use crate::helpers::scan_cgroup_cronjob;

use bytes::BytesMut;
use cortexbrain_common::map_handlers::{init_bpf_maps, map_pinner, populate_blocklist};
use cortexbrain_common::program_handlers::load_program;
use std::{
    convert::TryInto,
    path::Path,
    sync::{Arc, Mutex},
};

use anyhow::{Context, Ok};
use cortexbrain_common::{constants, logger};
use tokio::{fs, signal};
use tracing::{debug, error, info};

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
    let bpf_path =
        std::env::var(constants::BPF_PATH).context("BPF_PATH environment variable required")?;
    let data = fs::read(Path::new(&bpf_path))
        .await
        .context("failed to load file from path")?;

    //init bpf data
    let bpf = Arc::new(Mutex::new(Ebpf::load(&data)?));
    let bpf_map_save_path = std::env::var(constants::PIN_MAP_PATH)
        .context("PIN_MAP_PATH environment variable required")?;
    let data = vec![
        "events_map".to_string(),
        "veth_identity_map".to_string(),
        "TcpPacketRegistry".to_string(),
    ];
    match init_bpf_maps(bpf.clone(), data) {
        std::result::Result::Ok(bpf_maps) => {
            info!("Successfully loaded bpf maps");
            let pin_path = std::path::PathBuf::from(&bpf_map_save_path);
            info!("About to call map_pinner with path: {:?}", pin_path);
            match map_pinner(bpf_maps, &pin_path) {
                std::result::Result::Ok(maps) => {
                    info!("maps pinned successfully");
                    //load veth_trace program ref veth_trace.rs
                    {
                        init_veth_tracer(bpf.clone()).await?;
                    }

                    let interfaces = get_veth_channels();

                    info!("Found interfaces: {:?}", interfaces);

                    //{ FIXME: paused for testing the other features
                    //    populate_blocklist(&mut maps.2).await?;
                    //}

                    {
                        init_tc_classifier(bpf.clone(), interfaces, link_ids.clone()).await.context(
                            "An error occured during the execution of attach_bpf_program function"
                        )?;
                    }
                    {
                        init_tcp_registry(bpf.clone()).await.context(
                            "An error occured during the execution of init_tcp_registry function",
                        )?;
                    }

                    event_listener(maps, link_ids.clone(), bpf.clone())
                        .await
                        .map_err(|e| {
                            anyhow::anyhow!("Error inizializing event_listener. Reason: {}", e)
                        })?;
                }
                Err(e) => {
                    error!("Error while pinning bpf_maps: {}", e);
                }
            }
        }
        Err(e) => {
            error!("Error while loading bpf maps {}", e);
            let _ = signal::ctrl_c().await;
        }
    }

    Ok(())
}

//attach the tc classifier program to a vector of interfaces
async fn init_tc_classifier(
    bpf: Arc<Mutex<Ebpf>>,
    ifaces: Vec<String>,
    link_ids: Arc<Mutex<HashMap<String, SchedClassifierLinkId>>>,
) -> Result<(), anyhow::Error> {
    //this funtion initialize the tc classifier program
    info!("Loading programs");

    let mut bpf_new = bpf
        .lock()
        .map_err(|e| anyhow::anyhow!("Cannot get value from lock. Reason: {}", e))?;

    let program: &mut SchedClassifier = bpf_new
        .program_mut("identity_classifier")
        .ok_or_else(|| anyhow::anyhow!("program 'identity_classifier' not found"))?
        .try_into()
        .context("Failed to init SchedClassifier program")?;

    program
        .load()
        .context("Failed to load identity_classifier program")?;

    for interface in ifaces {
        match program.attach(&interface, TcAttachType::Ingress) {
            std::result::Result::Ok(link_id) => {
                info!(
                    "Program 'identity_classifier' attached to interface {}",
                    interface
                );
                let mut map = link_ids
                    .lock()
                    .map_err(|e| anyhow::anyhow!("Cannot get value from lock. Reason: {}", e))?;
                map.insert(interface.clone(), link_id);
            }
            Err(e) => error!(
                "Error attaching program to interface {}: {:?}",
                interface, e
            ),
        }
    }

    Ok(())
}

async fn init_veth_tracer(bpf: Arc<Mutex<Ebpf>>) -> Result<(), anyhow::Error> {
    //this functions init the veth_tracer used to make the InterfacesRegistry
    //creation tracer

    load_program(bpf.clone(), "veth_creation_trace", "register_netdevice")?;

    //deletion tracer
    load_program(bpf, "veth_deletion_trace", "unregister_netdevice_queue")?;

    Ok(())
}

async fn init_tcp_registry(bpf: Arc<Mutex<Ebpf>>) -> Result<(), anyhow::Error> {
    // init tcp registry

    // .clone() increments the reference count of the shared Ebpf instance.
    load_program(bpf.clone(), "tcp_message_tracer_rcv", "tcp_v4_rcv")?;

    info!("initializing tcp tracing functions");

    load_program(bpf, "tcp_message_tracer_connect", "tcp_v4_connect")?;

    Ok(())
}

// this function init the event listener. Listens for veth events (creation/deletion) and network events (pod to pod communications)
// Doc:
//
//   perf_net_events_array: contains is associated with the network events stored in the events_map (EventsMap)
//   perf_veth_array: contains is associated with the network events stored in the veth_map (veth_identity_map)
//
//
async fn event_listener(
    bpf_maps: Vec<Map>,
    link_ids: Arc<Mutex<HashMap<String, SchedClassifierLinkId>>>,
    bpf: Arc<Mutex<Ebpf>>,
) -> Result<(), anyhow::Error> {
    info!("Preparing perf_buffers and perf_arrays");

    //TODO: try to change from PerfEventArray to a RingBuffer data structure

    let mut perf_event_arrays = Vec::new(); // contains a vector of PerfEventArrays
    let mut event_buffers = Vec::new(); // contains a vector of buffers

    // create the PerfEventArrays and the buffers
    for map in bpf_maps {
        debug!("Debugging map type:{:?}", map);
        let perf_event_array = PerfEventArray::try_from(map).map_err(|e| {
            error!("Cannot create perf_event_array for map.Reason: {}", e);
            anyhow::anyhow!("Cannot create perf_event_array for map.Reason: {}", e)
        })?;
        perf_event_arrays.push(perf_event_array); // this is step 1
        let perf_event_array_buffer = Vec::new();
        event_buffers.push(perf_event_array_buffer); //this is step 2 
    }

    // fill the input buffers with data from the PerfEventArrays
    let cpus = online_cpus().map_err(|e| anyhow::anyhow!("Error {:?}", e))?;

    for (perf_evt_array, perf_evt_array_buffer) in
        perf_event_arrays.iter_mut().zip(event_buffers.iter_mut())
    {
        for cpu_id in &cpus {
            let single_buffer = perf_evt_array.open(*cpu_id, None)?;
            perf_evt_array_buffer.push(single_buffer);
        }
    }

    info!("Listening for events...");

    let mut event_buffers = event_buffers.into_iter();
    let perf_veth_buffer = event_buffers
        .next()
        .expect("Cannot create perf_veth buffer");
    let perf_net_events_buffer = event_buffers
        .next()
        .expect("Cannot create perf_net_events buffer");
    let tcp_registry_buffer = event_buffers
        .next()
        .expect("Cannot create tcp_registry buffer");

    // init output buffers
    let veth_buffers = vec![BytesMut::with_capacity(1024); 10];
    let events_buffers = vec![BytesMut::with_capacity(1024); online_cpus().iter().len()];
    let tcp_buffers = vec![BytesMut::with_capacity(1024); online_cpus().iter().len()];

    // init veth link ids
    let veth_link_ids = link_ids;

    // spawn async tasks
    let veth_events_displayer = tokio::spawn(async move {
        display_veth_events(bpf.clone(), perf_veth_buffer, veth_buffers, veth_link_ids).await;
    });

    let net_events_displayer = tokio::spawn(async move {
        display_events(perf_net_events_buffer, events_buffers).await;
    });

    let tcp_registry_events_displayer: tokio::task::JoinHandle<()> = tokio::spawn(async move {
        display_tcp_registry_events(tcp_registry_buffer, tcp_buffers).await;
    });

    #[cfg(feature = "experimental")]
    let scan_cgroup_cronjob = tokio::spawn(async move {
        let _ = scan_cgroup_cronjob(180).await;
    });

    #[cfg(not(feature = "experimental"))]
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

        result = tcp_registry_events_displayer => {
            match result{
                Err(e)=>error!("tcp_registry_events_displayer panicked {:?}",e),
                std::result::Result::Ok(_)=>info!("Found new tcp_register event")
            }
        }

        _= signal::ctrl_c()=>{
            info!("Triggered Exiting...");
        }

    }
    #[cfg(feature = "experimental")]
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

        result = net_events_displayer=>{
           match result{
                Err(e)=>error!("net_event_displayer panicked {:?}",e),
               std::result::Result::Ok(_)  => info!("Found new net_event"),
            }
        }

        result = tcp_registry_events_displayer => {
            match result{
                Err(e)=>error!("tcp_registry_events_displayer panicked {:?}",e),
                std::result::Result::Ok(_)=>info!("Found new tcp_register event")
            }
        }

        _= signal::ctrl_c()=>{
            info!("Triggered Exiting...");
        }

    }

    Ok(())
}
