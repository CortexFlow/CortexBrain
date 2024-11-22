#[warn(unused_imports)]
/* Imports */
use anyhow::{anyhow, Error, Ok, Result};
use ipnet::IpNet;
use k8s_openapi::api::core::v1::Node;
use kube::Api;
use std::sync::Arc;
use std::env;
use tracing::{error, info}; //used for logging

use crate::client::client::Client; //custom Client

pub struct EdgeCni<'a> {
    config: Arc<EdgeCniConfig>,
    client: Arc<Client>,
    pub mesh_adapter: MeshAdapter<'a>,
}

#[derive(Clone)]
pub struct EdgeCniConfig {
    pub enable: bool,
    // ... other fields
}

pub trait IpTableInterface {
    fn ensure_rule(&self, args: &[&str]) -> Result<String, Error>;
}
pub struct IPTables;
impl IPTables {
    fn insert(
        &self,
        table: &str,
        chain: &str,
        args: &[&str],
        _append: bool,
    ) -> Result<String, Error> {
        Ok(format!(
            "Rule inserted into table {} chain {} with args: {:?}",
            table, chain, args
        ))
    }
}
impl IpTableInterface for IPTables {
    fn ensure_rule(&self, args: &[&str]) -> Result<String, Error> {
        let res = self
            .insert("nat", "POSTROUTING", args, true)
            .map_err(|e| Error::msg(format!("An error occured {}", e)))?;

        Ok(res)
    }
}

/* Adapter implementations */
pub struct MeshAdapter<'a> {
    /*<'a>-->referes to "Rust Lifetimes"
    https://doc.rust-lang.org/rust-by-example/scope/lifetime.html

    Reference to Borrowing:
    https://doc.rust-lang.org/rust-by-example/scope/borrow.html
    DOC:
    Most of the time, we'd like to access data without taking ownership over it.
    To accomplish this, Rust uses a borrowing mechanism.
    Instead of passing objects by value (T), objects can be passed by reference (&T).
    The compiler statically guarantees (via its borrow checker) that references always
    point to valid objects. That is, while references to an object exist, the object cannot
    be destroyed.

    Reference to Rust Lifetimes: https://doc.rust-lang.org/rust-by-example/scope/lifetime.html
    DOC:
    Take, for example, the case where we borrow a variable via &. The borrow has a lifetime
    that is determined by where it is declared. As a result, the borrow is valid as long as
    it ends before the lender is destroyed. However, the scope of the borrow is determined by
    where the reference is used.

    */
    client: Arc<Client>, //my custom client defined in ./src/client/client.rs
    ipt_interface: Box<dyn IpTableInterface + 'a>, // Manages the iptables rules
    host_cidr: String,
    edge: Vec<String>,  //consider the upgrade to Vec<IpNet>
    cloud: Vec<String>, //consider the upgrade to Vec<IpNet>
    tun_dev_name: &'a str,
}
pub struct MeshCIDRConfig {
    pub cloud_cidr: Vec<String>,
    pub edge_cidr: Vec<String>,
}

impl<'a> EdgeCni<'a> {
    // Acts as a constructor. Accepts a config file and a Kubernetes client
    pub fn new(config: Arc<EdgeCniConfig>, client: Arc<Client>) -> Self {
        let mesh_adapter = MeshAdapter::new_mesh_adapter(&config, &client).unwrap();
        EdgeCni {
            config,
            client: client,
            mesh_adapter,
        }
    }

    //add route to tunnel
    pub fn add_route_to_tun(cidr: &str, tun_dev_name: &str) -> Result<()> {
        //Execute the command to add a route

        let output = std::process::Command::new("ip")
            .args(&["route", "add", cidr, "dev", tun_dev_name])
            .output()
            .map_err(|e| Error::msg(format!("Error executing ip route add {}", e)))?;

        //check if the command has been executed correclty
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::msg(format!(
                "Error adding the route {} to {}",
                tun_dev_name, stderr
            )));
        }
        Ok(())
    }

    pub fn name(&self) -> &str {
        "EdgeCni"
    }

    pub fn group(&self) -> &str {
        "EdgeNetworking"
    }

    pub fn enable(&self) -> bool {
        //enables the config
        self.config.enable
    }
    pub fn print_info(&self) {
        //user output
        print!("------- E D G E M E S H  N E T W O R K -------\n");
        println!("Info:");
        let name = self.name();
        let group = self.group();
        println!("Name: {}", name);
        print!("Group {}\n",group);
        print!("--------------------------------\n");
    }
    pub async fn start(&self) {
        if self.enable() {
            info!("Starting the CNI...");
            /*
            Here i need to inizialize the CNI workflow:

            Workflow:
            Is config.enable true?
                -Yes: start the service
                    - Initialize the VPN connection (Route Protection).
                        Ok:
                            - Inizialize the MeshAdapter::Watchroute function
                                Ok:
                                    - Everything is connected
                                Error:
                                    - Return an error status
                        Error:
                            - Return an error status


                -No: shutdown
                    -Return an error status



            */

            self.mesh_adapter.run().await;
        }
    }

    pub async fn shutdown(&self) {
        info!("Shutting down the CNI...");
        println!("Shutting down the Container Network Interface");
        if let Err(e) = self.cleanup_and_exit().await {
            error!("Cleanup failed {}", e);
        }
    }

    pub async fn cleanup_and_exit(&self) -> Result<(), Error> {
        self.mesh_adapter.close_route().await?;
        Ok(())
    }
}

impl<'a> MeshAdapter<'a> {
    pub fn new_mesh_adapter(_config: &EdgeCniConfig, client: &Client) -> Result<Self, Error> {
        let ipt_interface: Box<dyn IpTableInterface> = Box::new(IPTables);

        Ok(MeshAdapter {
            client: Arc::new(client.clone()),
            ipt_interface,
            host_cidr: "10.244.0.18/16".to_string(),
            edge: vec!["edge-node-1".to_string(), "edge-node-2".to_string()],
            cloud: vec!["cloud-node-1".to_string()],
            tun_dev_name: "tunnel0",
        })
    }

    pub async fn run(&self) {
        println!("Running the Mesh Adapter");
        info!("Running MeshAdapter...");
        // Implement the run functionality here
    }

    pub async fn close_route(&self) -> Result<(), Error> {
        println!("Closing route...");
        info!("Closing route...");
        //implement the stop function here
        Ok(())
    }

    // Function to read CIDR configuration and validate cloud and edge CIDRs
    pub fn get_cidr(&self, cfg: &MeshCIDRConfig) -> Result<(Vec<String>, Vec<String>), Error> {
        /* Workflow:
            1.Get the CIDR from the Mesh CIDR Configuration
                - OK :
                    -2. Validate the cidr
                        -Ok:
                            valid
                        -Error:
                            invalid CIDR
                -Error:
                    -Return error status


        */
        let cloud = cfg.cloud_cidr.clone();
        let edge = cfg.edge_cidr.clone();

        // Validate the cloud CIDRs
        if let Err(e) = Self::validate_cidrs(&cloud) {
            error!("Cloud CIDRs are invalid, error: {:?}", e);
        }

        // Validate the edge CIDRs
        if let Err(e) = Self::validate_cidrs(&edge) {
            error!("Edge CIDRs are invalid, error: {:?}", e);
        }

        Ok((cloud, edge))
    }

    // Helper function to validate CIDR list
    fn validate_cidrs(cidrs: &[String]) -> Result<(), Error> {
        /* Workflow:
            1. Perform Validate cidrs contained in the cidrs vector
                Ok:
                    -valid cidr
                Error:
                    -Invalid cidr
                    -Return error status

        */
        for cidr in cidrs {
            if !cidr.parse::<std::net::IpAddr>().is_ok() {
                error!("Invalid CIDR format: {}", cidr);
            }
        }
        Ok(())
    }

    pub async fn find_local_cidr(client: &Client) -> Result<String, Error> {
        // Ottieni il nome del nodo dall'ambiente
        let node_name = env::var("NODE_NAME")
            .map_err(|_| anyhow!("The env NODE_NAME is not set".to_string()))?;

        // Return the nodes from the Kubernetes APi

        //this function needs to be replaced with an aux function in the Client crate
        //client crate --> ./src/client/client.rs
        let nodes: Api<Node> = Api::all(client.kube_client.clone());

        // Recupera il nodo specifico
        let node = nodes
            .get(&node_name)
            .await
            .map_err(|e| anyhow!("Failed to get Node {}: {}", node_name, e))?;

        // Restituisci il PodCIDR del nodo, se presente
        if let Some(pod_cidr) = node.spec.as_ref().and_then(|spec| spec.pod_cidr.clone()) {
            Ok(pod_cidr)
        } else {
            error!("Node {} does not have a PodCIDR", node_name);
            Err(anyhow!("Node {} does not have a PodCIDR", node_name))
        }
    }

    //CheckTunCIDR--->check whether the mesh CIDR and the given parameter CIDR are in the same network or not.
    pub async fn check_tunnel_cidr(outer_cidr: &str, host_cidr: &str) -> Result<bool, Error> {
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
        let outer_network: IpNet = outer_cidr
            .parse()
            .map_err(|e| Error::msg(format!("Error parsing outer CIDR {}", e)))?;
        let mesh_network: IpNet = host_cidr
            .parse()
            .map_err(|e| Error::msg(format!("Error parsing host CIDR {}", e)))?;

        let outer_ip = outer_network.network();

        if !mesh_network.contains(&outer_ip) {
            return Err(Error::msg(
                "The outer IP does not belong to the host network",
            ));
        }

        if mesh_network.prefix_len() == outer_network.prefix_len() {
            return Ok(true);
        } else {
            return Err(Error::msg("Network masks do not match"));
        }
    }

    pub async fn watch_route(&self) -> Result<(), Error> {
        /* Workflow
            -1: Combine all the edge and cloud cidrs
            -2: check if the cidrs are in the same network
            -3: if the cidrs are not in the same network perform add_route_to_tun

        */

        //1. combine the edge and cloud cidrs
        let all_cidr = self.edge.iter().chain(self.cloud.iter());

        for cidr in all_cidr {
            //2.verify if the cidrs are in the same network
            let same_network = MeshAdapter::<'_>::check_tunnel_cidr(cidr, &self.host_cidr).await?;
            if !same_network {
                if let Err(e) = EdgeCni::add_route_to_tun(cidr, self.tun_dev_name) {
                    /* Err(Error::msg(format!("Failed to add route to TunnnelDevice, error {}",e))); */
                    error!("Failed to add route to TunnelDevice, error {}", e);
                    continue;
                }
            }
        }

        //insert an IPTable rule to avoid that other CNI execeute SNAT
        let rule = self
            .ipt_interface
            .ensure_rule(&[
                "-I",
                "nat",
                "POSTROUTING",
                "1",
                "-s",
                &self.host_cidr,
                "!",
                "-o",
                "docker0",
                "-j",
                "ACCEPT",
            ])
            .map_err(|e| Error::msg(format!("Failed to insert iptable rule, error: {}", e)))?;

        info!("Inserted iptable rule: {}", rule);
        Ok(())
    }

    /* TODO:
        - TunToTunnel implementation
        - HandleReceiveFromTun implementation
        - GetNodeNameByPodIP implementation

    */
}
