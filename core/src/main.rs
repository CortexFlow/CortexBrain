// module imports
mod client;  
mod edgecni;

use client::client::Client; 
use edgecni::edgecni::{EdgeCni, EdgeCniConfig, MeshAdapter, MeshCIDRConfig}; // Removed MeshAdapter since it's unused
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Configuration files
    let client_config = "config_string".to_string(); 
    let edge_cni_config = EdgeCniConfig {
        enable: true,
    };

    // Set the cloud and edge cidrs
    let cidr_config = MeshCIDRConfig {
        cloud_cidr: vec!["10.244.0.0/24".to_string()],
        edge_cidr: vec!["10.244.0.0/24".to_string()],
    };

    // Create your client instance using the custom Client struct
    let client = Client::new_client(&client_config).await?; // Fixed to use your custom client

    // Create EdgeCni instance with the new client
    let edge_cni = EdgeCni::new(edge_cni_config, client.kube_client.clone()); // Pass the kube_client from your custom Client
    
    // Use the mesh_adapter to call get_cidr
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

    // Retrieve and print the local CIDR for the node
    match MeshAdapter::find_local_cidr(&client.kube_client).await {
        Ok(local_cidr) => {
            println!("Local CIDR for the node: {}", local_cidr);
        }
        Err(e) => {
            println!("Error retrieving local CIDR: {}", e);
        }
    }

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
