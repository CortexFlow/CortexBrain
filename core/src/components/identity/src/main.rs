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
    convert::TryInto,
    path::Path,
    sync::{Arc, Mutex},
};

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

    if run(bpf, pin_path, init_bpf_maps).await? == Outcome::MapsInitFailed {
        let _ = signal::ctrl_c().await;
        let _ = otlp_provider.shutdown();
    }

    Ok(())
}

// init_maps is injected so tests can replace the real bpf map creation
async fn run<I>(
    bpf: Arc<Mutex<Ebpf>>,
    pin_path: std::path::PathBuf,
    init_maps: I,
) -> Result<Outcome, anyhow::Error>
where
    I: FnOnce(Arc<Mutex<Ebpf>>, Vec<String>) -> Result<BpfMapsData, anyhow::Error>,
{
    let map_data = REQUIRED_MAPS.iter().map(|m| m.to_string()).collect();
    match init_maps(bpf.clone(), map_data) {
        std::result::Result::Ok(bpf_maps) => {
            info!("Successfully loaded bpf maps");
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

                    {
                        populate_blocklist().await?;
                    }

                    {
                        init_tc_classifier(bpf.clone(), interfaces).await.context(
                            "An error occured during the execution of attach_bpf_program function",
                        )?;
                    }
                    {
                        init_tcp_registry(bpf.clone()).await.context(
                            "An error occured during the execution of init_tcp_registry function",
                        )?;
                    }

                    event_listener(maps).await.map_err(|e| {
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

    // load classifier program

    program
        .load()
        .context("Failed to load identity_classifier program")?;

    // attach program only to desired interfaces. We can skip the dock0,tunl0,lo and eth0 interface
    // we also save the interfaces to a BPF_HASH_MAP to easily monitor the interfaces using the agent

    // decleare link_ids HashMap which is a shared hashmap between kernel and userspace
    // Link_ids hashmap has type of HashMap<[u8; 16], [u8; 8]>. The key is the program name and the value is the state

    // at this point the pinning is already successfull so we can invoque the maps from the pin

    let link_ids_mapdata = MapData::from_pin("/sys/fs/bpf/maps/tracked_veth")
        .map_err(|e| anyhow!("Cannot return link_ids_mapdata. Reason: {}", e))?;

    let link_ids_map = Map::HashMap(link_ids_mapdata);

    let mut link_ids: aya::maps::HashMap<MapData, [u8; 16], [u8; 8]> =
        aya::maps::HashMap::try_from(link_ids_map).map_err(|e| {
            anyhow!(
                "Cannot create link_ids HashMap from link_ids_map. Reason:{}",
                e
            )
        })?;

    attach_and_register(
        ifaces,
        |interface| {
            program
                .attach(interface, TcAttachType::Ingress)
                .map(|_| ())
                .map_err(|e| anyhow!("{:?}", e))
        },
        |key, value| {
            link_ids
                .insert(key, value, 0)
                .map_err(|e| anyhow!("{}", e))
        },
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
    for cpu_id in online_cpus().map_err(|e| anyhow::anyhow!("Error {:?}", e))? {
        for (name, (perf_evt_array, perf_evt_array_buffer)) in maps.iter_mut() {
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
            perf_evt_array_buffer.push(buf);
        }
    }

    info!("Listening for events...");

    // i need to use remove to move the values from the Map Manager to the the async tasks
    let (perf_veth_array, perf_veth_buffers) = maps
        .remove("veth_identity_map")
        .ok_or_else(|| anyhow!("Cannot create perf_veth buffer: map veth_identity_map not found"))?;
    let (perf_net_events_array, perf_net_events_buffers) = maps
        .remove("events_map")
        .ok_or_else(|| anyhow!("Cannot create perf_net_events buffer: map events_map not found"))?;
    let (tcp_registry_array, tcp_registry_buffers) = maps
        .remove("TcpPacketRegistry")
        .ok_or_else(|| anyhow!("Cannot create tcp_registry buffer: map TcpPacketRegistry not found"))?;

    // init output buffers
    let veth_buffers = BufferSize::VethEvents.set_buffer();
    let events_buffers = BufferSize::ClassifierNetEvents.set_buffer();
    let tcp_buffers = BufferSize::TcpEvents.set_buffer();

    // spawn async tasks
    let veth_events_displayer = tokio::spawn(async move {
        read_perf_buffer(
            perf_veth_buffers,
            veth_buffers,
            Consumer::VethLog,
            None,
            None,
        )
        .await;
    });

    let net_events_displayer = tokio::spawn(async move {
        read_perf_buffer(
            perf_net_events_buffers,
            events_buffers,
            Consumer::PacketLog,
            None,
            None
        )
        .await;
    });

    let tcp_registry_events_displayer: tokio::task::JoinHandle<()> = tokio::spawn(async move {
        read_perf_buffer(
            tcp_registry_buffers,
            tcp_buffers,
            Consumer::TcpPacketRegistry,
            None,
            None
        )
        .await;
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

    #[tokio::test]
    async fn run_reports_maps_init_failure_without_further_work() {
        let bpf = empty_bpf().expect("minimal eBPF ELF should load");
        let seen = std::cell::RefCell::new(Vec::new());
        let outcome = run(bpf, "/nonexistent".into(), |_, names| {
            *seen.borrow_mut() = names;
            Err(anyhow!("cannot create maps"))
        })
        .await
        .unwrap();
        assert_eq!(outcome, Outcome::MapsInitFailed);
        assert_eq!(*seen.borrow(), REQUIRED_MAPS.to_vec());
    }
}
