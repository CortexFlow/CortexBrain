// module imports
mod client;
mod default_api_config;
mod params;
mod apiconfig;

use anyhow::Result;
use default_api_config::ApiConfig;
use crate::client::Client;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::EnvFilter;
use std::sync::Arc;
use crate::apiconfig::EdgeDNSConfig;
use crate::default_api_config::ConfigType;

//use std::env;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    //tracing subscriber for logging purposes
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

    //println!("Current working directory: {:?}", env::current_dir());
    // Load the configuration from the file
    let configuration = ApiConfig::load_from_file("./src/client/config.yaml", ConfigType::Default)?; 
    let edgecfg = EdgeDNSConfig::load_from_file("./src/client/config.yaml", ConfigType::Default)?; 

    // Create your client instance using the custom Client struct
    let client = Arc::new(Client::new_client(Some(ConfigType::Default)).await?); 

    // Print the client configuration
    Client::print_config(&client);

    Ok(())
}
