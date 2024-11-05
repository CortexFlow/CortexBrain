// module imports
mod client;  
mod edgecni;

use client::client::Client; 
use edgecni::edgecni::{EdgeCni, EdgeCniConfig,MeshCIDRConfig}; // Removed MeshAdapter since it's unused
use std::error::Error;


// Dummy struct for MeshCIDRConfig to match the Rust code

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Configuration files
    let client_config = "config_string".to_string(); 
    let edge_cni_config = EdgeCniConfig {
        enable: true,
    };
    //set the cloud and edge cidrs
    let cidr_config = MeshCIDRConfig {
        cloud_cidr: vec!["10.1.0.0/16".to_string()],
        edge_cidr: vec!["192.168.1.0/24".to_string()],
    };

    // Create your client instance using the custom Client struct
    let client = Client::new_client(&client_config).await?; // Fixed to use your custom client

    // Create EdgeCni instance with the new client
    let edge_cni = EdgeCni::new(edge_cni_config, client.kube_client.clone()); // Pass the kube_client from your custom Client
    
    // Usa il mesh_adapter per chiamare get_cidr
    match edge_cni.mesh_adapter.get_cidr(&cidr_config) {
        Ok((cloud, edge)) => {
            println!("Cloud CIDRs: {:?}", cloud);
            println!("Edge CIDRs: {:?}", edge);
        }
        Err(e) => {
            println!("\n");
            println!("Error in the CIDR configuration:\n {}\n", e);
        }
    }



    // Start the services
    edge_cni.start().await;

    // Actions
    // Retrieve and print the list of pods in the "default" namespace
    let pods = client.list_pods("default").await?; // This now uses your custom list_pods method
    for pod in pods {
        println!("Found pod: {:?}", pod.metadata.name);
    }

    // Shutdown
    edge_cni.shutdown().await;
    Ok(())
}
