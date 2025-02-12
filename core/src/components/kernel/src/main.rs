// module imports

/* mod apiconfig;
mod client;
mod default_api_config;
mod params; */
//use crate::apiconfig::EdgeDNSConfig;
//use crate::default_api_config::ConfigType;

mod kernel;


use anyhow::{Context, Result};
use tracing::info;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::EnvFilter;

use shared::apiconfig::EdgeDNSConfig;
use shared::default_api_config::{ApiConfig,ConfigType};

use crate::kernel::EdgeDNS;



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

    // Load the configuration from the file

    let config_path =std::env::var(CONFIG_PATH).context("CONFIG_PATH enviroment variable required")?;
    if !std::path::Path::new(&config_path).exists() {
        return Err(anyhow::anyhow!("Configuration file not found at {}", config_path));
    }

    info!("Using CONFIG_PATH = {}", &config_path);
    let edgecfg = EdgeDNSConfig::load_from_file(&config_path, ConfigType::Default).context("Failed to load the DNS configuration")?;

    let edgedns = EdgeDNS::new(configuration, edgecfg, client.clone()).await?;
    edgedns.get_kernel_info();
    edgedns.start().await;

    Ok(())
}