/* contains the code for the kernel xdp manipulation. this code lives in
the kernel space only and needs to be attached to a program in the user space
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

/*
XDP flags
Mode | Description | Compatibility | Performance
DRIVER_MODE | XDP native in the driver | Only compatible cards | Highest
SKB_MODE | XDP on top of Linux stack | Always compatible | Good
HW_MODE | XDP on hardware | Requires hardware support | Highest (very rare)
*/

//main program
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




    //loading the pre-built binaries--> reason: linux kernel does not accept non compiled code. only accepts bytecode
    info!("loading data");
    let data = fs::read("../../../target/bpfel-unknown-none/release/xdp-filter").await.context("failed to load file from path")?;
    let mut bpf = aya::Ebpf::load(&data).context("failed to load data from file")?;
    let mut maps_owned = bpf.take_map("services").context("failed to take services map")?;
    let backend_map = bpf.take_map("Backend_ports").context("failed to take backends map")?;
    

    if Path::new("/sys/fs/bpf/services").exists(){
        warn!("map already pinned,skipping process");
    }
    else{
        maps_owned.pin("/sys/fs/bpf/services").context("failed to pin map")?;
    }

    let mut service_map = UserSpaceMap::<&mut MapData, SVCKey, SVCValue>::try_from(&mut maps_owned)?;
    let mut backends = UserSpaceMap::<MapData, u16, BackendPorts>::try_from(backend_map)?;

    let mut ports = [0;4];
    ports[0] = 9876;
    ports[1] = 9877;

    let backend_ports = BackendPorts{
        ports:ports,
        index:0,
    }; 
    backends.insert(9875,backend_ports,0);


    //declare service discovery
    let service_discovery=ServiceDiscovery::new(&mut service_map).await?;

    let kubeconfig_path = PathBuf::from("/home/cortexflow/.kube/config");

    /* annotations for permissions:
    sudo chmod 644 /home/<name>/.kube/config
    sudo chown <name>:<name> /home/<name>/.kube/config

    sudo mkdir -p /root/.kube
    sudo cp /home/<name>/.kube/config /root/.kube/config
 */


    let kubeconfig = Kubeconfig::read_from(kubeconfig_path)?;
    let config = Config::from_custom_kubeconfig(kubeconfig, &Default::default()).await?;
    let client = Client::try_from(config)?;

    let configmap: Api<ConfigMap> = Api::namespaced(client.clone(), "cortexflow");

    let proxycfg = EdgeProxyConfig::load_from_configmap(configmap, ConfigType::Default).await?;
    let loadbalancer = Loadbalancer::new(proxycfg,service_discovery).await?;
    loadbalancer.run().await?;

    Ok(())
}
