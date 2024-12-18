// module imports
mod client;
mod edgecni;
mod kernel;
use client::client::Client;
use edgecni::edgecni::{EdgeCni, MeshAdapter, MeshCIDRConfig};
use std::error::Error;
use std::sync::Arc;

use client::default_api_config::{Config, ConfigType};
use kernel::kernel::EdgeDNS;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    //let edge_cni_config = EdgeCniConfig { enable: true };

    // Set the cloud and edge cidrs
    let cidr_config = MeshCIDRConfig {
        cloud_cidr: vec!["10.244.0.0/24".to_string()],
        edge_cidr: vec!["10.244.0.0/24".to_string()],
    };

    // Create your client instance using the custom Client struct
    let client = Arc::new(Client::new_client(Some((ConfigType::V1))).await?); // Use Arc for shared reference
    
    Client::print_config(&client); //return the client config

    //EdgeDNS::new(Some(ConfigType::Default), client);
    
    // Create EdgeCni instance with the new client, passing a reference to edge_cni_config
    /* let edge_cni = EdgeCni::new((edge_cni_config).clone().into(), (*client).clone().into()); // Deference and clone client
    // Create a MeshAdapter using the new_mesh_adapter method, passing a reference to edge_cni_config
    let mesh_adapter = MeshAdapter::new_mesh_adapter(&edge_cni_config, &client)?;

    // Use the mesh_adapter to call get_cidr
    match mesh_adapter.get_cidr(&cidr_config) {
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
    match MeshAdapter::find_local_cidr(&client).await {
        Ok(local_cidr) => {
            println!("Local CIDR for the node: {}", local_cidr);
        }
        Err(e) => {
            println!("Error retrieving local CIDR: {}", e);
        }
    }

    let outer_cidr = "192.168.1.0/24";
    let host_cidr = "192.168.1.0/24";
    match MeshAdapter::check_tunnel_cidr(outer_cidr, host_cidr).await {
        Ok(result) => {
            if result {
                println!("The outer ip match with the inner host");
            } else {
                println!("The outer ip does not match with the inner host");
            }
        }
        Err(e) => {
            println!("Error checking tunnel CIDR: {}", e);
        }
    }
    // Call the watch_route function to monitor network routes
    edge_cni.mesh_adapter.watch_route().await?;

    // Actions
    // Retrieve and print the list of pods in the "default" namespace
    let pods = client.list_pods("default").await?; // This now uses your custom list_pods method
    for pod in pods {
        println!("Found pod: {:?}", pod.metadata.name);
    }

    // Shutdown
    edge_cni.shutdown().await;
 */    Ok(())
}
