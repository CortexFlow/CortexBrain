//module import s
mod client;  
mod edgecni;

use client::client::Client; 
use cni::cni::{EdgeCni,EdgeCniConfig,MeshAdapter};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    //config files
    //client config
    let client_config = "config_string".to_string(); 
    let edge_cni_config = EdgeCniConfig{
        enable:true
    };
    
    //Service configurations
    // Creiamo il nostro client
    let client = Client::new_client(&config).await?;
    let edge_cni=EdgeCni::new(config,client);
    
    //start dei servizi
    edge_cni.start().await;
    
    //Actions
    // Recuperiamo e stampiamo la lista dei pod nel namespace "default"
    let pods = client.list_pods("default").await?;
    for pod in pods {
        println!("Found pod: {:?}", pod.metadata.name);
    }

    //shutting down
    edge_cni.shutdown().await;
    Ok(())
}
