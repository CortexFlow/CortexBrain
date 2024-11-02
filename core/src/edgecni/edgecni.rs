use kube::{
    api::{Api, ListParams},
    Client as Kubeclient,
};
use iptables;
use log::{error, info};
use std::sync::Arc;

pub struct EdgeCni {
    config: EdgeCniConfig,
    client: Arc<Kubeclient>,
    mesh_adapter: MeshAdapter,
}

pub struct EdgeCniConfig {
    enable: bool,
    //.. other implementations
}
/* Adapter implementations */
pub struct MeshAdapter {
    client: Kubeclient,
    ipt_interface: iptables::IPTables, //gestisce le regole iptables
    host_cidr: String,
    edge: Vec<String>,
    // ... other implementations
}

impl EdgeCni {
    //act as a constructor. Accept a config file and a kubernetes client
    pub fn new(config: EdgeCniConfig, client: Kubeclient) -> Self {
        let mesh_adapter = MeshAdapter::new(&config, &client).unwrap();
        EdgeCni {
            config,
            client: Arc::new(client),
            mesh_adapter,
        }
    }
    pub fn name(&self) -> &str {
        "EdgeCni"
    }
    pub fn group(&self) -> &str {
        "EdgeNetworking"
    }
    pub fn enable(&self) -> bool {
        self.config.enable
    }
    pub async fn start(&self) {
        if self.enable() {
            info!("Starting the CNI...");
            self.mesh_adapter.run().await;
        }
    }
    pub async fn shutdown(&self) {
        info!("Shutting down the CNI...");
        if let Err(e) = self.cleanup_and_exit().await {
            error!("Cleaup failed {}", e);
        }
    }
    pub async fn cleanup_and_exit(&self) -> Result<(), String> {
        self.mesh_adapter.close_route().await?;
        Ok(())
    }
}

impl MeshAdapter {
    pub fn new_mesh_adapter(config: &EdgeCniConfig, client: &Client) -> Result<Self, String> {
        Ok(MeshAdapter {
            client: client.clone(),
            ipt_interface,
            host_cidr: "192.168.0.0./16".to_string(),
            edge: vec!["edge-node-1".to_string(), "edge-node-2".to_string()],
            // ... other fields
        }) //inizialization
    }
    pub async fn run(&self) {
        info!("Running MeshAdapter...");
    }
    pub async fn close_route(&self) -> Result<(), String> {
        info!("Closing route...");
        OK(())
    }
}
