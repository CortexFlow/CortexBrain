/* Resource
https://github.com/EmilHernvall/dnsguide/blob/master/chapter1.md
*/

/* CoreDNS-->Dns resolver di Kubernetes */
/* Kubernetes in rust:
    https://www.shuttle.dev/blog/2024/10/22/using-kubernetes-with-rust
*/
#[warn(unused_imports)]
use std::sync::Arc;
use anyhow::{Error, Ok, Result};
use crate::client::client::Client;
use crate::utilities::utilities;
use tracing::info;
pub struct EdgeDNS {
    config: Arc<EdgeDNSConfig>,
}

#[derive(Clone)]
pub struct EdgeDNSConfig {
    pub enable: bool,
    pub namespace: String,
    // ... other fields
}

impl EdgeDNS {
    pub fn name(&self) -> &str {
        "EdgeDNS"
    }
    pub fn group(&self) -> &str {
        "EdgeDNSNetwork"
    }
    pub fn enable(&self) -> bool {
        self.config.enable
    }
    pub async fn start(&self) {
        if self.enable() {
            self.run().await;
        }
    }

    pub async fn run(&self) {
        info!("EdgeDNS is running ");
        //TODO: Implement the EdgeDNS run function
    }

    pub async fn shutdown(&self) {
        info!("Shutting down the EdgeDNS ");
        //TODO: Implement the EdgeDNS shutdown function
    }

    pub fn update_corefile(config: &EdgeDNSConfig, clients: &Client) -> Result<()> {
        info!("Updating the EdgeDNS corefile configuration");
        Ok(())
    }

    pub fn new(config: EdgeDNSConfig, client: &Client) -> Result<Self, Error> {
        if !config.enable {
            return Ok(EdgeDNS {
                config: Arc::new(config),
            });
        }

        // Update Corefile if EdgeDNS is enabled
        EdgeDNS::update_corefile(&config, client)?;

        Ok(EdgeDNS {
            config: Arc::new(config),
        })
    }

    pub fn register(config: EdgeDNSConfig, client: Client) -> Result<(), Error> {
        //TODO: This function interacts with the KubeEdge Core function. 
        let dns = EdgeDNS::new(config, &client)?;
        info!("EdgeDNS module registered successfully");
        Ok(())
    }
}
