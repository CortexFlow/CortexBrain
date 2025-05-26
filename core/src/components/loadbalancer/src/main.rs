/* 
    * Contains the Load balancer (CortexFlow Agent) user space code implementation.
    * The implementation leverages the power of bpf programs to interact with the internet interface
    * to distribute load accross multiple backends.
    * The program leverages bpf maps to enable the communication between Kernel Space and User Space

    * //TODO: Update the code to use the discovered services from the cortexflow identity service 
    */

/*     
    * Annotations

    let kubeconfig_path = PathBuf::from("/home/cortexflow/.kube/config");
*/
    /* annotations for permissions:
    sudo chmod 644 /home/<name>/.kube/config
    sudo chown <name>:<name> /home/<name>/.kube/config

    sudo mkdir -p /root/.kube
    sudo cp /home/<name>/.kube/config /root/.kube/config
*/

mod shared_struct;
mod discovery;

mod messaging;
mod metrics;
mod loadbalancer;

use std::error;

use anyhow::Context;
use aya::programs::{Xdp, XdpFlags};
use tokio::fs;
use tokio::signal;
use aya_log::EbpfLogger;
use tracing::{error,info,warn};
use std::path::Path;
use std::path::PathBuf;

use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::EnvFilter;

use aya::maps::{HashMap as UserSpaceMap, MapData};
use crate::shared_struct::{SVCKey,SVCValue,BackendPorts};
use crate::loadbalancer::Loadbalancer;
use crate::discovery::ServiceDiscovery; 
use shared::{apiconfig::EdgeProxyConfig, default_api_config::ConfigType};
use k8s_openapi::api::core::v1::ConfigMap;
use kube::{Client,Config, api::Api};
use kube::config::Kubeconfig;


unsafe impl aya::Pod for shared_struct::SVCKey {}
unsafe impl aya::Pod for shared_struct::SVCValue {}
unsafe impl aya::Pod for shared_struct::BackendPorts {}

const BPF_PATH : &str = "BPF_PATH";


//main program
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {

    // * init tracing subscriber
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




    // * loading the pre-built binaries--> reason: linux kernel does not accept non compiled code. only accepts bytecode
    
    info!("loading data");
    let bpf_path= std::env::var(BPF_PATH).context("BPF_PATH environment variable required")?;
    let data = fs::read(Path::new(&bpf_path)).await.context("failed to load file from path")?;
    info!("loading bpf data");
    let mut bpf = aya::Ebpf::load(&data).context("failed to load data from file")?;
    info!("loading maps data");
    let mut maps_owned = bpf.take_map("services").context("failed to take services map")?;
    info!("loading bpf backends map");
    let backend_map = bpf.take_map("Backend_ports").context("failed to take backends map")?;


    if Path::new("/sys/fs/bpf/services").exists(){
        warn!("map already pinned,skipping process");
    }
    else{
        maps_owned.pin("/sys/fs/bpf/services").context("failed to pin map")?;
    }

    info!("loading service map in user space");
    let mut service_map = UserSpaceMap::<&mut MapData, SVCKey, SVCValue>::try_from(&mut maps_owned)?;
    info!("loading backends in user space");
    let mut backends = UserSpaceMap::<MapData, u16, BackendPorts>::try_from(backend_map)?;

    let mut ports = [0;4];
    ports[0] = 9876;
    ports[1] = 9877;

    let backend_ports = BackendPorts{
        ports:ports,
        index:0,
    }; 
    backends.insert(5053,backend_ports,0);


    //declare service discovery
    info!("Initializing service discovery");
    let service_discovery=ServiceDiscovery::new(&mut service_map).await?;

    info!("connecting to client");
    let client = Client::try_default().await?;

    info!("reading kubernetes configmap");
    let configmap: Api<ConfigMap> = Api::namespaced(client.clone(), "cortexflow");

    info!("Loading Loadbalancer configuration from configmap");
    let lbcfg = EdgeProxyConfig::load_from_configmap(configmap, ConfigType::Default).await?;
    info!("Initializing Loadbalancer");
    let loadbalancer = Loadbalancer::new(lbcfg,service_discovery,backends).await?;
    loadbalancer.run().await?;

    Ok(())
}
