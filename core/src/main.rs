// module imports
mod client;
mod developers_msg;
mod edgecni;
mod kernel;

use anyhow::Result;
use client::{client::Client, default_api_config::ApiConfig};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::EnvFilter;
use std::sync::Arc;
use crate::client::apiconfig::EdgeDNSConfig;
use crate::developers_msg::developers_msg::info;
use client::default_api_config::ConfigType;
use kernel::kernel::EdgeDNS;
//use kernel::kafka::test_kafka;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    
    //tracing subscriber for logging purpouses
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_level(true)
        .with_span_events(FmtSpan::NONE)
        .without_time()
        .with_target(false)
        .with_file(false)
        .pretty()
        .with_env_filter(EnvFilter::new("info"))
        .with_line_number(false)
        .init();
    //Development message for all the developers
    info();

    //TODO: general: clean unused or useless code in EdgeDNSConfig
    //let edge_cni_config = EdgeCniConfig { enable: true };
    let configuration = ApiConfig::load_from_file("./src/client/config.yaml", ConfigType::Default)?; /* the "?" operand return a "Result" type. Necessary */
    let edgecfg = EdgeDNSConfig::load_from_file("./src/client/config.yaml", ConfigType::Default)?; /* the "?" operand return a "Result" type. Necessary */
    let edgecnicfg = EdgeDNSConfig::load_from_file("./src/client/config.yaml", ConfigType::Default);

    // Create your client instance using the custom Client struct
    let client = Arc::new(Client::new_client(Some(ConfigType::Default)).await?); // Use Arc for shared reference

    Client::print_config(&client); //return the client config
    let edgedns = EdgeDNS::new(configuration, edgecfg, client.clone()).await?;
    edgedns.get_kernel_info();
    edgedns.start().await;

    //test_kafka();

    Ok(())
}
