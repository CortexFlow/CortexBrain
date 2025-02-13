// module imports

mod kernel;
mod corefile;
mod utilities;

use anyhow::Result;

use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::EnvFilter;

use shared::apiconfig::EdgeDNSConfig;
use shared::default_api_config::ConfigType;


use crate::kernel::EdgeDNS;


use kube::{api::Api, Client};
use k8s_openapi::api::core::v1::ConfigMap;


const CONFIG_PATH: &str = "CONFIG_PATH";

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    //tracing subscriber for logging purposes
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

    // Load the configuration from the Kubernetes API:
    /* Workflow:
        - load the configmap from the Kubernetes API
        - read the dns config from the configmap    
        - apply the configmap 
        - start the server
    */

    let client = Client::try_default().await?;
    let configmap: Api<ConfigMap> = Api::namespaced(client.clone(), "default");

    let edgecfg = EdgeDNSConfig::load_from_configmap(configmap, ConfigType::Default).await?;

    let edgedns = EdgeDNS::new(edgecfg, client.clone()).await?;
    edgedns.get_kernel_info();
    edgedns.start().await;

    Ok(())
}