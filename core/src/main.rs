// module imports
mod client;
mod edgecni;
mod kernel;
use client::{apiconfig::KubeApiConfig, client::Client, default_api_config::ApiConfig};
use edgecni::edgecni::{EdgeCni, MeshAdapter, MeshCIDRConfig};
use std::sync::Arc;
use anyhow::{Result,Error};

use client::default_api_config::ConfigType;
use kernel::kernel::EdgeDNS;
use crate::client::apiconfig::{EdgeDNSConfig,EdgeCNIConfig};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    //let edge_cni_config = EdgeCniConfig { enable: true };
    let configuration = ApiConfig::load_from_file("./src/client/config.yaml", ConfigType::Default)?; /* the "?" operand return a "Result" type. Necessary */
    let edgecfg = EdgeDNSConfig::load_from_file("./src/client/config.yaml", ConfigType::Default)?; /* the "?" operand return a "Result" type. Necessary */
    let edgecnicfg = EdgeDNSConfig::load_from_file("./src/client/config.yaml", ConfigType::Default);

    // Create your client instance using the custom Client struct
    let client = Arc::new(Client::new_client(Some(ConfigType::Default)).await?); // Use Arc for shared reference
    
    Client::print_config(&client); //return the client config
    EdgeDNS::new(configuration, edgecfg, client.clone()).await?;
    

    Ok(())
}
