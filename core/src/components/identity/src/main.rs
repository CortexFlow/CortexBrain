/*
 * CortexBrain Identity Service
 * Features:
 *   1. TCP events tracker
 *   2. veth creation and deletion tracker
 *   3. TC (traffic control) tracker
 *   4. [Experimental]: cgroup scanner
 *
 */

mod helpers;
mod service_discovery;

use crate::helpers::{attach_and_register, get_veth_channels};
use aya::{
    Ebpf,
    maps::{Map, MapData},
    programs::{SchedClassifier, TcAttachType},
    util::online_cpus,
};

#[cfg(feature = "experimental")]
use crate::helpers::scan_cgroup_cronjob;

use cortexbrain_common::{
    buffer_type::BufferSize,
    constants,
    consumer::Consumer,
    consumer::read_perf_buffer,
    logger,
    map_handlers::BpfMapsData,
    map_handlers::{init_bpf_maps, map_manager, map_pinner, populate_blocklist},
    program_handlers::load_program,
};
use std::{
    collections::HashMap,
    convert::TryInto,
    path::Path,
    sync::{Arc, Mutex},
};
#[cfg(not(feature = "experimental"))]
use std::future::Future;

use anyhow::{Context, Ok, anyhow};

//use std::collections::HashMap;
use tokio::{fs, signal};
use tracing::{error, info};

// maps created by init_bpf_maps. event_listener requires veth_identity_map, events_map and TcpPacketRegistry
const REQUIRED_MAPS: [&str; 5] = [
    "events_map",
    "veth_identity_map",
    "TcpPacketRegistry",
    "Blocklist",
    "tracked_veth",
];

#[derive(Debug, PartialEq)]
enum Outcome {
    Finished,
    // bpf maps could not be created: main waits for ctrl_c before exiting
    MapsInitFailed,
}

// reads the bpf object path from the env variable named env_key and loads the eBPF object
async fn load_bpf_object(env_key: &str) -> Result<Arc<Mutex<Ebpf>>, anyhow::Error> {
    let bpf_path =
        std::env::var(env_key).context(format!("{} environment variable required", env_key))?;
    let data = fs::read(Path::new(&bpf_path))
        .await
        .context("failed to load file from path")?;
    Ok(Arc::new(Mutex::new(Ebpf::load(&data)?)))
}

fn bpf_map_save_path(env_key: &str) -> Result<std::path::PathBuf, anyhow::Error> {
    let path =
        std::env::var(env_key).context(format!("{} environment variable required", env_key))?;
    Ok(std::path::PathBuf::from(path))
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    //init otlè tracing subscriber
    let otlp_provider = logger::otlp_logger_init("identity_service-OTLP".to_string());

    info!("Starting identity service...");
    info!("fetching data");

    //init conntracker data path and bpf data
    let bpf = load_bpf_object(constants::BPF_PATH).await?;
    let pin_path = bpf_map_save_path(constants::PIN_MAP_PATH)?;

    if run(bpf, pin_path, &IdentityOperations).await? == Outcome::MapsInitFailed {
        let _ = signal::ctrl_c().await;
        let _ = otlp_provider.shutdown();
    }

    Ok(())
}

trait StartupOperations {
    fn init_maps(
        &self,
        bpf: Arc<Mutex<Ebpf>>,
        names: Vec<String>,
    ) -> Result<BpfMapsData, anyhow::Error>;
    fn pin_maps(
        &self,
        maps: BpfMapsData,
        path: &std::path::PathBuf,
    ) -> Result<BpfMapsData, anyhow::Error>;
    async fn init_veth(&self, bpf: Arc<Mutex<Ebpf>>) -> Result<(), anyhow::Error>;
    fn interfaces(&self) -> Vec<String>;
    async fn blocklist(&self) -> Result<(), anyhow::Error>;
    async fn init_classifier(
        &self,
        bpf: Arc<Mutex<Ebpf>>,
        ifaces: Vec<String>,
    ) -> Result<(), anyhow::Error>;
    async fn init_tcp(&self, bpf: Arc<Mutex<Ebpf>>) -> Result<(), anyhow::Error>;
    async fn listen(&self, maps: BpfMapsData) -> Result<(), anyhow::Error>;
}

struct IdentityOperations;

impl StartupOperations for IdentityOperations {
    fn init_maps(
        &self,
        bpf: Arc<Mutex<Ebpf>>,
        names: Vec<String>,
    ) -> Result<BpfMapsData, anyhow::Error> {
        init_bpf_maps(bpf, names)
    }

    fn pin_maps(
        &self,
        maps: BpfMapsData,
        path: &std::path::PathBuf,
    ) -> Result<BpfMapsData, anyhow::Error> {
        map_pinner(maps, path)
    }

    async fn init_veth(&self, bpf: Arc<Mutex<Ebpf>>) -> Result<(), anyhow::Error> {
        init_veth_tracer(bpf).await
    }

    fn interfaces(&self) -> Vec<String> {
        get_veth_channels()
    }

    async fn blocklist(&self) -> Result<(), anyhow::Error> {
        populate_blocklist().await
    }

    async fn init_classifier(
        &self,
        bpf: Arc<Mutex<Ebpf>>,
        ifaces: Vec<String>,
    ) -> Result<(), anyhow::Error> {
        init_tc_classifier(bpf, ifaces).await
    }

    async fn init_tcp(&self, bpf: Arc<Mutex<Ebpf>>) -> Result<(), anyhow::Error> {
        init_tcp_registry(bpf).await
    }

    async fn listen(&self, maps: BpfMapsData) -> Result<(), anyhow::Error> {
        event_listener(maps).await
    }
}

async fn run(
    bpf: Arc<Mutex<Ebpf>>,
    pin_path: std::path::PathBuf,
    ops: &impl StartupOperations,
) -> Result<Outcome, anyhow::Error> {
    let map_data = REQUIRED_MAPS.iter().map(|m| m.to_string()).collect();
    match ops.init_maps(bpf.clone(), map_data) {
        std::result::Result::Ok(bpf_maps) => {
            info!("Successfully loaded bpf maps");
            info!("About to call map_pinner with path: {:?}", pin_path);
            match ops.pin_maps(bpf_maps, &pin_path) {
                std::result::Result::Ok(maps) => {
                    info!("maps pinned successfully");
                    //load veth_trace program ref veth_trace.rs
                    {
                        ops.init_veth(bpf.clone()).await?;
                    }

                    let interfaces = ops.interfaces();

                    info!("Found interfaces: {:?}", interfaces);

                    {
                        ops.blocklist().await?;
                    }

                    {
                        ops.init_classifier(bpf.clone(), interfaces).await.context(
                            "An error occured during the execution of attach_bpf_program function",
                        )?;
                    }
                    {
                        ops.init_tcp(bpf.clone()).await.context(
                            "An error occured during the execution of init_tcp_registry function",
                        )?;
                    }

                    ops.listen(maps).await.map_err(|e| {
                        anyhow::anyhow!("Error inizializing event_listener. Reason: {}", e)
                    })?;
                }
                Err(e) => {
                    error!("Error while pinning bpf_maps: {}", e);
                }
            }
            Ok(Outcome::Finished)
        }
        Err(e) => {
            error!("Error while loading bpf maps {}", e);
            Ok(Outcome::MapsInitFailed)
        }
    }
}

//attach the tc classifier program to a vector of interfaces
// TODO: consider to create a load schedule classifier in the common functions
async fn init_tc_classifier(
    bpf: Arc<Mutex<Ebpf>>,
    ifaces: Vec<String>,
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

    init_tc_classifier_with(
        program,
        ifaces,
        |program| {
            program
                .load()
                .context("Failed to load identity_classifier program")
        },
        || {
            let mapdata = MapData::from_pin("/sys/fs/bpf/maps/tracked_veth")
                .map_err(|e| anyhow!("Cannot return link_ids_mapdata. Reason: {}", e))?;
            aya::maps::HashMap::<MapData, [u8; 16], [u8; 8]>::try_from(Map::HashMap(mapdata))
                .map_err(|e| {
                    anyhow!(
                        "Cannot create link_ids HashMap from link_ids_map. Reason:{}",
                        e
                    )
                })
        },
        |program, interface| {
            program
                .attach(interface, TcAttachType::Ingress)
                .map(|_| ())
                .map_err(|e| anyhow!("{:?}", e))
        },
        |link_ids, key, value| link_ids.insert(key, value, 0).map_err(|e| anyhow!("{}", e)),
    )
}

fn init_tc_classifier_with<P, M>(
    program: &mut P,
    ifaces: Vec<String>,
    load: impl FnOnce(&mut P) -> Result<(), anyhow::Error>,
    open_map: impl FnOnce() -> Result<M, anyhow::Error>,
    mut attach: impl FnMut(&mut P, &str) -> Result<(), anyhow::Error>,
    mut insert: impl FnMut(&mut M, [u8; 16], [u8; 8]) -> Result<(), anyhow::Error>,
) -> Result<(), anyhow::Error> {
    load(program)?;
    let mut map = open_map()?;
    attach_and_register(
        ifaces,
        |interface| attach(program, interface),
        |key, value| insert(&mut map, key, value),
    );
    Ok(())
}

async fn init_veth_tracer(bpf: Arc<Mutex<Ebpf>>) -> Result<(), anyhow::Error> {
    init_veth_tracer_with(bpf, load_program).await
}

async fn init_veth_tracer_with<L>(bpf: Arc<Mutex<Ebpf>>, loader: L) -> Result<(), anyhow::Error>
where
    L: Fn(Arc<Mutex<Ebpf>>, &str, &str) -> Result<(), anyhow::Error>,
{
    //this functions init the veth_tracer used to make the InterfacesRegistry
    //creation tracer

    loader(bpf.clone(), "veth_creation_trace", "register_netdevice")?;

    //deletion tracer
    loader(bpf, "veth_deletion_trace", "unregister_netdevice_queue")?;

    Ok(())
}

async fn init_tcp_registry(bpf: Arc<Mutex<Ebpf>>) -> Result<(), anyhow::Error> {
    init_tcp_registry_with(bpf, load_program).await
}

async fn init_tcp_registry_with<L>(bpf: Arc<Mutex<Ebpf>>, loader: L) -> Result<(), anyhow::Error>
where
    L: Fn(Arc<Mutex<Ebpf>>, &str, &str) -> Result<(), anyhow::Error>,
{
    // init tcp registry

    // .clone() increments the reference count of the shared Ebpf instance.
    loader(bpf.clone(), "tcp_message_tracer_rcv", "tcp_v4_rcv")?;

    info!("initializing tcp tracing functions");

    loader(bpf, "tcp_message_tracer_connect", "tcp_v4_connect")?;

    Ok(())
}

fn open_perf_buffers<A, B>(
    maps: &mut HashMap<String, (A, Vec<B>)>,
    cpus: impl IntoIterator<Item = u32>,
    mut open: impl FnMut(&str, &mut A, u32) -> Result<B, anyhow::Error>,
) -> Result<(), anyhow::Error> {
    for cpu in cpus {
        for (name, (array, buffers)) in maps.iter_mut() {
            buffers.push(open(name, array, cpu)?);
        }
    }
    Ok(())
}

fn take_listener_maps<T>(maps: &mut HashMap<String, T>) -> Result<(T, T, T), anyhow::Error> {
    let veth = maps.remove("veth_identity_map").ok_or_else(|| {
        anyhow!("Cannot create perf_veth buffer: map veth_identity_map not found")
    })?;
    let network = maps
        .remove("events_map")
        .ok_or_else(|| anyhow!("Cannot create perf_net_events buffer: map events_map not found"))?;
    let tcp = maps.remove("TcpPacketRegistry").ok_or_else(|| {
        anyhow!("Cannot create tcp_registry buffer: map TcpPacketRegistry not found")
    })?;
    Ok((veth, network, tcp))
}

fn start_event_consumers<T, H>(
    veth: T,
    network: T,
    tcp: T,
    mut start: impl FnMut(T, BufferSize, Consumer) -> H,
) -> (H, H, H) {
    (
        start(veth, BufferSize::VethEvents, Consumer::VethLog),
        start(
            network,
            BufferSize::ClassifierNetEvents,
            Consumer::PacketLog,
        ),
        start(tcp, BufferSize::TcpEvents, Consumer::TcpPacketRegistry),
    )
}

#[cfg(not(feature = "experimental"))]
enum ListenerExit {
    Veth(Result<(), tokio::task::JoinError>),
    Network(Result<(), tokio::task::JoinError>),
    Tcp(Result<(), tokio::task::JoinError>),
    Shutdown,
}

#[cfg(not(feature = "experimental"))]
async fn wait_for_event_listener(
    veth: tokio::task::JoinHandle<()>,
    network: tokio::task::JoinHandle<()>,
    tcp: tokio::task::JoinHandle<()>,
    shutdown: impl Future<Output = std::io::Result<()>>,
) -> ListenerExit {
    tokio::select! {
        result = veth => ListenerExit::Veth(result),
        result = network => ListenerExit::Network(result),
        result = tcp => ListenerExit::Tcp(result),
        _ = shutdown => ListenerExit::Shutdown,
    }
}

// this function init the event listener. Listens for veth events (creation/deletion) and network events (pod to pod communications)
// Doc:
//
//   perf_net_events_array: contains is associated with the network events stored in the events_map (EventsMap)
//   perf_veth_array: contains is associated with the network events stored in the veth_map (veth_identity_map)
//
//
async fn event_listener(bpf_maps: BpfMapsData) -> Result<(), anyhow::Error> {
    info!("Preparing perf_buffers and perf_arrays");

    //TODO: try to change from PerfEventArray to a RingBuffer data structure

    let mut maps = map_manager(bpf_maps)?;

    // fill the input buffers with data from the PerfEventArrays
    open_perf_buffers(
        &mut maps,
        online_cpus().map_err(|e| anyhow::anyhow!("Error {:?}", e))?,
        |name, perf_evt_array, cpu_id| {
            let buf = perf_evt_array.open(cpu_id, None).map_err(|e| {
                anyhow!(
                    "Cannot create perf_event_array buffer from perf_event_array. Reason: {}",
                    e
                )
            })?;
            info!(
                "Buffer created for map {:?} on cpu_id {:?}. Buffer size: {}",
                name,
                cpu_id,
                std::mem::size_of_val(&buf)
            );
            Ok(buf)
        },
    )?;

    info!("Listening for events...");

    // i need to use remove to move the values from the Map Manager to the the async tasks
    let (
        (_perf_veth_array, perf_veth_buffers),
        (_perf_net_events_array, perf_net_events_buffers),
        (_tcp_registry_array, tcp_registry_buffers),
    ) = take_listener_maps(&mut maps)?;

    let (veth_events_displayer, net_events_displayer, tcp_registry_events_displayer) =
        start_event_consumers(
            perf_veth_buffers,
            perf_net_events_buffers,
            tcp_registry_buffers,
            |buffers, size, consumer| {
                let output = size.set_buffer();
                tokio::spawn(async move {
                    read_perf_buffer(buffers, output, consumer, None, None).await;
                })
            },
        );

    #[cfg(feature = "experimental")]
    let scan_cgroup_cronjob = tokio::spawn(async move {
        let _ = scan_cgroup_cronjob(180).await;
    });

    #[cfg(not(feature = "experimental"))]
    match wait_for_event_listener(
        veth_events_displayer,
        net_events_displayer,
        tcp_registry_events_displayer,
        signal::ctrl_c(),
    )
    .await
    {
        ListenerExit::Veth(result) => match result {
            Err(e) => error!("veth_event_displayer panicked {:?}", e),
            std::result::Result::Ok(_) => info!("Found new veth_event"),
        },

        ListenerExit::Network(result) => match result {
            Err(e) => error!("net_event_displayer panicked {:?}", e),
            std::result::Result::Ok(_) => info!("Found new net_event"),
        },

        ListenerExit::Tcp(result) => match result {
            Err(e) => error!("tcp_registry_events_displayer panicked {:?}", e),
            std::result::Result::Ok(_) => info!("Found new tcp_register event"),
        },

        ListenerExit::Shutdown => {
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

#[cfg(test)]
mod tests {
    use super::*;

    // Minimal valid eBPF ELF64 object (header only, no programs).
    fn minimal_elf() -> Vec<u8> {
        let mut elf = vec![0u8; 64];
        elf[..4].copy_from_slice(b"\x7fELF");
        elf[4] = 2; // 64-bit
        elf[5] = 1; // little endian
        elf[6] = 1; // version
        elf[16..18].copy_from_slice(&1u16.to_le_bytes()); // ET_REL
        elf[18..20].copy_from_slice(&247u16.to_le_bytes()); // EM_BPF
        elf[20..24].copy_from_slice(&1u32.to_le_bytes());
        elf[52..54].copy_from_slice(&64u16.to_le_bytes()); // e_ehsize
        elf[58..60].copy_from_slice(&64u16.to_le_bytes()); // e_shentsize
        elf
    }

    fn empty_bpf() -> Option<Arc<Mutex<Ebpf>>> {
        Ebpf::load(&minimal_elf())
            .ok()
            .map(|b| Arc::new(Mutex::new(b)))
    }

    #[tokio::test]
    async fn init_veth_tracer_fails_when_program_missing() {
        let bpf = empty_bpf().expect("minimal eBPF ELF should load");
        let err = init_veth_tracer(bpf).await.unwrap_err();
        assert!(err.to_string().contains("veth_creation_trace"));
    }

    #[tokio::test]
    async fn init_tcp_registry_fails_when_program_missing() {
        let bpf = empty_bpf().expect("minimal eBPF ELF should load");
        let err = init_tcp_registry(bpf).await.unwrap_err();
        assert!(err.to_string().contains("tcp_message_tracer_rcv"));
    }

    #[tokio::test]
    async fn init_functions_fail_on_poisoned_lock() {
        let bpf = empty_bpf().expect("minimal eBPF ELF should load");
        let b = bpf.clone();
        let _ = std::thread::spawn(move || {
            let _g = b.lock().unwrap();
            panic!("poison");
        })
        .join();
        assert!(init_veth_tracer(bpf.clone()).await.is_err());
        assert!(init_tcp_registry(bpf).await.is_err());
    }

    #[tokio::test]
    async fn init_tc_classifier_fails_when_program_missing() {
        let bpf = empty_bpf().expect("minimal eBPF ELF should load");
        let err = init_tc_classifier(bpf, vec!["veth0".to_string()])
            .await
            .unwrap_err();
        assert!(err.to_string().contains("identity_classifier"));
    }

    #[tokio::test]
    async fn init_tc_classifier_fails_on_poisoned_lock() {
        let bpf = empty_bpf().expect("minimal eBPF ELF should load");
        let b = bpf.clone();
        let _ = std::thread::spawn(move || {
            let _g = b.lock().unwrap();
            panic!("poison");
        })
        .join();
        let err = init_tc_classifier(bpf, vec![]).await.unwrap_err();
        assert!(err.to_string().contains("Cannot get value from lock"));
    }

    #[test]
    fn classifier_loads_opens_map_and_registers_attached_interfaces() {
        let calls = std::cell::RefCell::new(Vec::new());
        let mut program = ();
        init_tc_classifier_with(
            &mut program,
            vec!["veth0".into(), "bad".into(), "veth1".into()],
            |_| {
                calls.borrow_mut().push("load".to_string());
                Ok(())
            },
            || {
                calls.borrow_mut().push("open".to_string());
                Ok(())
            },
            |_, iface| {
                calls.borrow_mut().push(format!("attach:{iface}"));
                if iface == "bad" {
                    Err(anyhow!("attach failed"))
                } else {
                    Ok(())
                }
            },
            |_, key, value| {
                assert_eq!(value, *b"attached");
                let name = std::str::from_utf8(&key).unwrap().trim_end_matches('\0');
                calls.borrow_mut().push(format!("insert:{name}"));
                if name == "veth0" {
                    Err(anyhow!("insert failed"))
                } else {
                    Ok(())
                }
            },
        )
        .unwrap();
        assert_eq!(
            *calls.borrow(),
            [
                "load",
                "open",
                "attach:veth0",
                "insert:veth0",
                "attach:bad",
                "attach:veth1",
                "insert:veth1"
            ]
        );
    }

    #[test]
    fn classifier_stops_if_load_or_map_open_fails() {
        for failing in ["load", "open"] {
            let calls = std::cell::RefCell::new(Vec::new());
            let result = init_tc_classifier_with(
                &mut (),
                vec!["veth0".into()],
                |_| {
                    calls.borrow_mut().push("load");
                    if failing == "load" {
                        Err(anyhow!("load failed"))
                    } else {
                        Ok(())
                    }
                },
                || {
                    calls.borrow_mut().push("open");
                    if failing == "open" {
                        Err(anyhow!("open failed"))
                    } else {
                        Ok(())
                    }
                },
                |_, _| {
                    calls.borrow_mut().push("attach");
                    Ok(())
                },
                |_, _, _| {
                    calls.borrow_mut().push("insert");
                    Ok(())
                },
            );
            assert_eq!(result.unwrap_err().to_string(), format!("{failing} failed"));
            let expected = if failing == "load" {
                vec!["load"]
            } else {
                vec!["load", "open"]
            };
            assert_eq!(*calls.borrow(), expected);
        }
    }

    #[test]
    fn classifier_with_no_interfaces_still_loads_and_opens_map() {
        let calls = std::cell::RefCell::new(Vec::new());
        init_tc_classifier_with(
            &mut (),
            vec![],
            |_| {
                calls.borrow_mut().push("load");
                Ok(())
            },
            || {
                calls.borrow_mut().push("open");
                Ok(())
            },
            |_, _| {
                calls.borrow_mut().push("attach");
                Ok(())
            },
            |_, _, _| {
                calls.borrow_mut().push("insert");
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(*calls.borrow(), ["load", "open"]);
    }

    // With no usable maps, map_manager returns an empty registry and
    // event_listener returns an error for the first required map.
    #[tokio::test]
    async fn event_listener_errors_without_maps() {
        let maps = BpfMapsData {
            bpf_obj_names: vec![],
            bpf_obj_map: vec![],
        };
        let err = event_listener(maps).await.unwrap_err();
        assert!(err.to_string().contains("veth_identity_map"));
    }

    // zip() truncates to the shorter vector, so names without maps are ignored.
    #[tokio::test]
    async fn event_listener_ignores_names_without_maps() {
        let maps = BpfMapsData {
            bpf_obj_names: vec!["veth_identity_map".to_string()],
            bpf_obj_map: vec![],
        };
        let err = event_listener(maps).await.unwrap_err();
        assert!(err.to_string().contains("veth_identity_map"));
    }

    #[test]
    fn event_listener_requires_each_map_in_order() {
        let names = ["veth_identity_map", "events_map", "TcpPacketRegistry"];
        for missing in 0..names.len() {
            let mut maps: HashMap<String, usize> = names
                .iter()
                .enumerate()
                .filter(|(index, _)| *index != missing)
                .map(|(index, name)| (name.to_string(), index))
                .collect();
            let error = take_listener_maps(&mut maps).unwrap_err();
            assert!(error.to_string().contains(names[missing]));
        }
        let mut maps: HashMap<String, usize> = names
            .iter()
            .enumerate()
            .map(|(index, name)| (name.to_string(), index))
            .chain(std::iter::once(("extra".to_string(), 3)))
            .collect();
        assert_eq!(take_listener_maps(&mut maps).unwrap(), (0, 1, 2));
        assert_eq!(maps.get("extra"), Some(&3));
    }

    #[test]
    fn event_listener_opens_one_buffer_per_map_and_cpu() {
        let mut maps = HashMap::from([
            ("veth_identity_map".to_string(), (1, Vec::new())),
            ("events_map".to_string(), (2, Vec::new())),
        ]);
        open_perf_buffers(&mut maps, [0, 3], |_, array, cpu| Ok((*array, cpu))).unwrap();
        assert_eq!(maps["veth_identity_map"].1, [(1, 0), (1, 3)]);
        assert_eq!(maps["events_map"].1, [(2, 0), (2, 3)]);
    }

    #[test]
    fn event_listener_stops_opening_buffers_on_failure() {
        let mut maps = HashMap::from([("events_map".to_string(), ((), Vec::<u32>::new()))]);
        let mut calls = Vec::new();
        let error = open_perf_buffers(&mut maps, [0, 1, 2], |name, _, cpu| {
            calls.push((name.to_string(), cpu));
            if cpu == 1 {
                Err(anyhow!("open failed"))
            } else {
                Ok(cpu)
            }
        })
        .unwrap_err();
        assert_eq!(error.to_string(), "open failed");
        assert_eq!(
            calls,
            [("events_map".to_string(), 0), ("events_map".to_string(), 1)]
        );
        assert_eq!(maps["events_map"].1, [0]);
    }

    #[test]
    fn event_listener_starts_consumers_with_matching_buffers() {
        let mut calls = Vec::new();
        let handles = start_event_consumers("veth", "network", "tcp", |buffers, size, consumer| {
            let correct_pair = matches!(
                (&size, &consumer),
                (BufferSize::VethEvents, Consumer::VethLog)
                    | (BufferSize::ClassifierNetEvents, Consumer::PacketLog)
                    | (BufferSize::TcpEvents, Consumer::TcpPacketRegistry)
            );
            assert!(correct_pair);
            calls.push(buffers);
            buffers
        });
        assert_eq!(calls, ["veth", "network", "tcp"]);
        assert_eq!(handles, ("veth", "network", "tcp"));
    }

    #[cfg(not(feature = "experimental"))]
    #[tokio::test]
    async fn event_listener_observes_each_consumer_exit_and_shutdown() {
        for exit in ["veth", "network", "tcp", "shutdown"] {
            for panic in [false, true] {
                if exit == "shutdown" && panic {
                    continue;
                }
                let spawn = |selected: bool| {
                    tokio::spawn(async move {
                        if selected {
                            assert!(!panic, "consumer failed");
                        } else {
                            std::future::pending::<()>().await;
                        }
                    })
                };
                let veth = spawn(exit == "veth");
                let network = spawn(exit == "network");
                let tcp = spawn(exit == "tcp");
                let abort = [
                    veth.abort_handle(),
                    network.abort_handle(),
                    tcp.abort_handle(),
                ];
                let shutdown = async {
                    if exit != "shutdown" {
                        std::future::pending::<()>().await;
                    }
                    std::io::Result::Ok(())
                };
                let result = tokio::time::timeout(
                    std::time::Duration::from_secs(5),
                    wait_for_event_listener(veth, network, tcp, shutdown),
                )
                .await
                .expect("listener did not exit");
                for handle in abort {
                    handle.abort();
                }
                let task_result = match (exit, result) {
                    ("veth", ListenerExit::Veth(result))
                    | ("network", ListenerExit::Network(result))
                    | ("tcp", ListenerExit::Tcp(result)) => Some(result),
                    ("shutdown", ListenerExit::Shutdown) => None,
                    _ => panic!("unexpected listener exit for {exit}"),
                };
                if let Some(result) = task_result {
                    if panic {
                        assert!(result.unwrap_err().is_panic());
                    } else {
                        result.unwrap();
                    }
                }
            }
        }
    }

    type Calls = std::cell::RefCell<Vec<(String, String)>>;

    fn recorder(calls: &Calls, fail_on: Option<&str>) -> impl Fn(Arc<Mutex<Ebpf>>, &str, &str) -> Result<(), anyhow::Error> {
        let fail_on = fail_on.map(str::to_string);
        move |_, prog, sym| {
            calls.borrow_mut().push((prog.to_string(), sym.to_string()));
            if fail_on.as_deref() == Some(prog) {
                return Err(anyhow!("mock failure for {}", prog));
            }
            std::result::Result::Ok(())
        }
    }

    #[tokio::test]
    async fn init_veth_tracer_loads_both_programs_in_order() {
        let bpf = empty_bpf().expect("minimal eBPF ELF should load");
        let calls = Calls::default();
        init_veth_tracer_with(bpf, recorder(&calls, None)).await.unwrap();
        assert_eq!(
            *calls.borrow(),
            vec![
                ("veth_creation_trace".into(), "register_netdevice".into()),
                ("veth_deletion_trace".into(), "unregister_netdevice_queue".into()),
            ]
        );
    }

    #[tokio::test]
    async fn init_veth_tracer_stops_after_first_failure() {
        let bpf = empty_bpf().expect("minimal eBPF ELF should load");
        let calls = Calls::default();
        let err = init_veth_tracer_with(bpf, recorder(&calls, Some("veth_creation_trace")))
            .await
            .unwrap_err();
        assert!(err.to_string().contains("veth_creation_trace"));
        assert_eq!(calls.borrow().len(), 1);
    }

    #[tokio::test]
    async fn init_veth_tracer_propagates_second_failure() {
        let bpf = empty_bpf().expect("minimal eBPF ELF should load");
        let calls = Calls::default();
        let err = init_veth_tracer_with(bpf, recorder(&calls, Some("veth_deletion_trace")))
            .await
            .unwrap_err();
        assert!(err.to_string().contains("veth_deletion_trace"));
        assert_eq!(calls.borrow().len(), 2);
    }

    #[tokio::test]
    async fn init_tcp_registry_loads_both_programs_in_order() {
        let bpf = empty_bpf().expect("minimal eBPF ELF should load");
        let calls = Calls::default();
        init_tcp_registry_with(bpf, recorder(&calls, None)).await.unwrap();
        assert_eq!(
            *calls.borrow(),
            vec![
                ("tcp_message_tracer_rcv".into(), "tcp_v4_rcv".into()),
                ("tcp_message_tracer_connect".into(), "tcp_v4_connect".into()),
            ]
        );
    }

    #[tokio::test]
    async fn init_tcp_registry_skips_connect_when_rcv_fails() {
        let bpf = empty_bpf().expect("minimal eBPF ELF should load");
        let calls = Calls::default();
        let err = init_tcp_registry_with(bpf, recorder(&calls, Some("tcp_message_tracer_rcv")))
            .await
            .unwrap_err();
        assert!(err.to_string().contains("tcp_message_tracer_rcv"));
        assert_eq!(calls.borrow().len(), 1);
    }

    #[tokio::test]
    async fn init_tcp_registry_propagates_connect_failure() {
        let bpf = empty_bpf().expect("minimal eBPF ELF should load");
        let calls = Calls::default();
        let err = init_tcp_registry_with(bpf, recorder(&calls, Some("tcp_message_tracer_connect")))
            .await
            .unwrap_err();
        assert!(err.to_string().contains("tcp_message_tracer_connect"));
        assert_eq!(calls.borrow().len(), 2);
    }

    // env vars are process-global, so tests that touch them run one at a time
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn required_maps_include_those_used_by_event_listener() {
        for m in ["veth_identity_map", "events_map", "TcpPacketRegistry"] {
            assert!(REQUIRED_MAPS.contains(&m), "{} missing", m);
        }
        assert_eq!(REQUIRED_MAPS.len(), 5);
    }

    #[tokio::test]
    async fn load_bpf_object_requires_env_var() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let key = "CF_TEST_BPF_PATH_UNSET";
        unsafe { std::env::remove_var(key) };
        let err = load_bpf_object(key).await.err().unwrap();
        assert_eq!(err.to_string(), format!("{} environment variable required", key));
    }

    #[tokio::test]
    async fn load_bpf_object_fails_for_missing_file() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let key = "CF_TEST_BPF_PATH_MISSING";
        unsafe { std::env::set_var(key, "/nonexistent/identity.o") };
        let err = load_bpf_object(key).await.err().unwrap();
        assert_eq!(err.to_string(), "failed to load file from path");
    }

    #[tokio::test]
    async fn load_bpf_object_fails_for_invalid_elf() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let key = "CF_TEST_BPF_PATH_GARBAGE";
        let file = std::env::temp_dir().join("cf_identity_garbage.o");
        std::fs::write(&file, b"not an elf").unwrap();
        unsafe { std::env::set_var(key, &file) };
        assert!(load_bpf_object(key).await.is_err());
        let _ = std::fs::remove_file(file);
    }

    #[tokio::test]
    async fn load_bpf_object_loads_valid_object() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let key = "CF_TEST_BPF_PATH_VALID";
        let file = std::env::temp_dir().join("cf_identity_valid.o");
        std::fs::write(&file, minimal_elf()).unwrap();
        unsafe { std::env::set_var(key, &file) };
        assert!(load_bpf_object(key).await.is_ok());
        let _ = std::fs::remove_file(file);
    }

    #[test]
    fn bpf_map_save_path_reads_env() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let key = "CF_TEST_PIN_PATH";
        unsafe { std::env::remove_var(key) };
        let err = bpf_map_save_path(key).unwrap_err();
        assert_eq!(err.to_string(), format!("{} environment variable required", key));

        unsafe { std::env::set_var(key, "/sys/fs/bpf/maps") };
        assert_eq!(
            bpf_map_save_path(key).unwrap(),
            std::path::PathBuf::from("/sys/fs/bpf/maps")
        );
    }

    struct MockStartup {
        calls: std::cell::RefCell<Vec<&'static str>>,
        fail_at: Option<&'static str>,
    }

    impl MockStartup {
        fn step(&self, name: &'static str) -> Result<(), anyhow::Error> {
            self.calls.borrow_mut().push(name);
            if self.fail_at == Some(name) {
                Err(anyhow!("{name} failed"))
            } else {
                Ok(())
            }
        }
    }

    impl StartupOperations for MockStartup {
        fn init_maps(
            &self,
            _: Arc<Mutex<Ebpf>>,
            names: Vec<String>,
        ) -> Result<BpfMapsData, anyhow::Error> {
            assert_eq!(names, REQUIRED_MAPS);
            self.step("init_maps")?;
            Ok(BpfMapsData {
                bpf_obj_names: vec!["test_map".into()],
                bpf_obj_map: vec![],
            })
        }

        fn pin_maps(
            &self,
            maps: BpfMapsData,
            path: &std::path::PathBuf,
        ) -> Result<BpfMapsData, anyhow::Error> {
            assert_eq!(path, std::path::Path::new("/test/pin"));
            assert_eq!(maps.bpf_obj_names, ["test_map"]);
            self.step("pin_maps")?;
            Ok(maps)
        }

        async fn init_veth(&self, _: Arc<Mutex<Ebpf>>) -> Result<(), anyhow::Error> {
            self.step("veth")
        }

        fn interfaces(&self) -> Vec<String> {
            self.calls.borrow_mut().push("interfaces");
            vec!["veth0".into(), "veth1".into()]
        }

        async fn blocklist(&self) -> Result<(), anyhow::Error> {
            self.step("blocklist")
        }

        async fn init_classifier(
            &self,
            _: Arc<Mutex<Ebpf>>,
            ifaces: Vec<String>,
        ) -> Result<(), anyhow::Error> {
            assert_eq!(ifaces, ["veth0", "veth1"]);
            self.step("classifier")
        }

        async fn init_tcp(&self, _: Arc<Mutex<Ebpf>>) -> Result<(), anyhow::Error> {
            self.step("tcp")
        }

        async fn listen(&self, maps: BpfMapsData) -> Result<(), anyhow::Error> {
            assert_eq!(maps.bpf_obj_names, ["test_map"]);
            self.step("listen")
        }
    }

    #[tokio::test]
    async fn run_initializes_and_listens_in_order() {
        let ops = MockStartup {
            calls: Default::default(),
            fail_at: None,
        };
        let bpf = empty_bpf().expect("minimal eBPF ELF should load");
        assert_eq!(
            run(bpf, "/test/pin".into(), &ops).await.unwrap(),
            Outcome::Finished
        );
        assert_eq!(
            *ops.calls.borrow(),
            [
                "init_maps",
                "pin_maps",
                "veth",
                "interfaces",
                "blocklist",
                "classifier",
                "tcp",
                "listen"
            ]
        );
    }

    #[tokio::test]
    async fn run_stops_at_each_failure() {
        let stages = [
            "init_maps",
            "pin_maps",
            "veth",
            "interfaces",
            "blocklist",
            "classifier",
            "tcp",
            "listen",
        ];
        for (index, stage) in stages
            .iter()
            .enumerate()
            .filter(|(_, s)| **s != "interfaces")
        {
            let ops = MockStartup {
                calls: Default::default(),
                fail_at: Some(stage),
            };
            let bpf = empty_bpf().expect("minimal eBPF ELF should load");
            let result = run(bpf, "/test/pin".into(), &ops).await;
            match *stage {
                "init_maps" => assert_eq!(result.unwrap(), Outcome::MapsInitFailed),
                "pin_maps" => assert_eq!(result.unwrap(), Outcome::Finished),
                _ => {
                    let error = result.unwrap_err();
                    assert!(
                        format!("{error:#}").contains(&format!("{stage} failed")),
                        "{error:#}"
                    );
                    match *stage {
                        "classifier" => assert!(error.to_string().contains("attach_bpf_program")),
                        "tcp" => assert!(error.to_string().contains("init_tcp_registry")),
                        "listen" => assert!(error.to_string().contains("event_listener")),
                        _ => {}
                    }
                }
            }
            assert_eq!(*ops.calls.borrow(), stages[..=index]);
        }
    }
}
