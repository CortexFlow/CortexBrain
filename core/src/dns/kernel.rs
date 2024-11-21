/* Resource
https://github.com/EmilHernvall/dnsguide/blob/master/chapter1.md
*/

/* CoreDNS-->Dns resolver di Kubernetes */
/* Kubernetes in rust:
    https://www.shuttle.dev/blog/2024/10/22/using-kubernetes-with-rust
*/
#[warn(unused_imports)]
use anyhow::{anyhow, Error, Ok, Result};
use kube::config;
use std::sync::Arc;
use tracing::{error, info};

use crate::client::client::Client;
pub struct EdgeDNS {
    config: Arc<EdgeDNSConfig>,
}
pub struct EdgeDNSConfig {
    pub enable: bool,
    // ... other fields
}

impl EdgeDNS {
    pub fn new(config: EdgeDNSConfig) -> Result<Self, Error> {
        //act as a constructor
        Ok(EdgeDNS {
            config: Arc::new(config),
        })
    }
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
        info!("EdgeDNS is running ")
        //TODO: Implement the EdgeDNS run function
    }
    pub async fn shutdown(&self) {
        info!("Shutting down the EdgeDNS ")
        //TODO: Implement the EdgeCNS shutdown function
    }

    pub fn update_corefile(config: &EdgeDNSConfig) -> Result<()> {
        info!("Updating the EdgeDNS corefile configuration");
        Ok(())
    }
}


 