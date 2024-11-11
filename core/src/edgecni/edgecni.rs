use ipnet::IpNet;
use iptables;
use k8s_openapi::api::core::v1::Node;
use kube::Api;
use kube::Client as Kubeclient;
use std::env;
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
            host_cidr: "10.244.0.18/32".to_string(),
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

    pub async fn find_local_cidr(client: &Kubeclient) -> Result<String, String> {
        // Ottieni il nome del nodo dall'ambiente
        let node_name =
            env::var("NODE_NAME").map_err(|_| "The env NODE_NAME is not set".to_string())?;

        // Ottieni l'API per i nodi
        let nodes: Api<Node> = Api::all(client.clone());

        // Recupera il nodo specifico
        let node = nodes
            .get(&node_name)
            .await
            .map_err(|e| format!("Failed to get Node {}: {}", node_name, e))?;

        // Restituisci il PodCIDR del nodo, se presente
        if let Some(pod_cidr) = node.spec.as_ref().and_then(|spec| spec.pod_cidr.clone()) {
            Ok(pod_cidr)
        } else {
            Err(format!("Node {} does not have a PodCIDR", node_name))
        }
    }
    //CheckTunCIDR--->check whether the mesh CIDR and the given parameter CIDR are in the same network or not.
    pub async fn check_tunnel_cidr(cidr1: &str, cidr2: &str) -> bool {
        /* Workflow:

        1. Parse the provided outer CIDR.

        Use net.ParseCIDR(outerCidr) to get the IP address and network of the outer CIDR.
        If there is an error in parsing, return an error.

        2. Parse the CIDR of the host network associated with the mesh.
        Use net.ParseCIDR(mesh.HostCIDR) to get the network of the host CIDR associated with the MeshAdapter.
        If there is an error in parsing, return an error.

        3. Check if the outer IP address is contained in the host mesh.
        Use hostNet.Contains(outerIP) to check if the external IP address (outerIP) belongs to the host network (hostNet).

        4. Verify that the network masks are the same
        Compare the network masks using hostNet.Mask.String() and outerNet.Mask.String() to make sure they are identical.

        5. Return the result
        If both verifications are true, return true.
        If one of the verifications fails, return false.
        If an error occurs during parsing, return a descriptive error.

        */

        /*      see ipnet documentation-->https://crates.io/crates/ipnet
        https://docs.rs/ipnet/latest/ipnet/ */
        let network1: IpNet = cidr1.parse().unwrap();
        let network2: IpNet = cidr2.parse().unwrap();

        network1 == network2
    }
}

pub struct MeshCIDRConfig {
    pub cloud_cidr: Vec<String>,
    pub edge_cidr: Vec<String>,
}
