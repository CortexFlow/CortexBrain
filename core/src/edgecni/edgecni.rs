use iptables;
use kube::Client as Kubeclient;
use std::sync::Arc;
use tracing::{error, info};

pub struct EdgeCni {
    config: EdgeCniConfig,
    client: Arc<Kubeclient>,
    pub mesh_adapter: MeshAdapter,
}

pub struct EdgeCniConfig {
    pub enable: bool,
    // ... other fields
}

/* Adapter implementations */
pub struct MeshAdapter {
    client: Kubeclient,
    ipt_interface: iptables::IPTables, // Manages the iptables rules
    host_cidr: String,
    edge: Vec<String>,
    // ... other fields
}

impl EdgeCni {
    // Acts as a constructor. Accepts a config file and a Kubernetes client
    pub fn new(config: EdgeCniConfig, client: Kubeclient) -> Self {
        let mesh_adapter = MeshAdapter::new_mesh_adapter(&config, &client).unwrap();
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
            error!("Cleanup failed {}", e);
        }
    }

    pub async fn cleanup_and_exit(&self) -> Result<(), String> {
        self.mesh_adapter.close_route().await?;
        Ok(())
    }
}

impl MeshAdapter {
    pub fn new_mesh_adapter(_config: &EdgeCniConfig, client: &Kubeclient) -> Result<Self, String> {
        let ipt_interface = iptables::IPTables {
            cmd: "iptables",
            has_check: true,
            has_wait: true,
            is_numeric: false,
        };

        Ok(MeshAdapter {
            client: client.clone(),
            ipt_interface,
            host_cidr: "192.168.0.0/16".to_string(),
            edge: vec!["edge-node-1".to_string(), "edge-node-2".to_string()],
            // ... other fields
        })
    }

    pub async fn run(&self) {
        info!("Running MeshAdapter...");
        // Implement the actual functionality here
    }

    pub async fn close_route(&self) -> Result<(), String> {
        info!("Closing route...");
        Ok(())
    }

    /// Function to read CIDR configuration and validate cloud and edge CIDRs
    pub fn get_cidr(&self, cfg: &MeshCIDRConfig) -> Result<(Vec<String>, Vec<String>), String> {
        let cloud = cfg.cloud_cidr.clone();
        let edge = cfg.edge_cidr.clone();

        // Validate the cloud CIDRs
        if let Err(e) = Self::validate_cidrs(&cloud) {
            return Err(format!("Cloud CIDRs are invalid, error: {:?}", e));
        }

        // Validate the edge CIDRs
        if let Err(e) = Self::validate_cidrs(&edge) {
            return Err(format!("Edge CIDRs are invalid, error: {:?}", e));
        }

        Ok((cloud, edge))
    }

    /// Helper function to validate CIDR list
    fn validate_cidrs(cidrs: &[String]) -> Result<(), String> {
        for cidr in cidrs {
            if !cidr.parse::<std::net::IpAddr>().is_ok() {
                return Err(format!("Invalid CIDR format: {}", cidr));
            }
        }
        Ok(())
    }
    //aggiungere:
    //findLocalCIDR
    //CheckTunCIDR
}

pub struct MeshCIDRConfig {
    pub cloud_cidr: Vec<String>,
    pub edge_cidr: Vec<String>,
}