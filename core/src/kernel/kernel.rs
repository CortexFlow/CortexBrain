/* Resource
https://github.com/EmilHernvall/dnsguide/blob/master/chapter1.md
*/

/* CoreDNS-->Dns resolver di Kubernetes */
/* Kubernetes in rust:
    https://www.shuttle.dev/blog/2024/10/22/using-kubernetes-with-rust
*/
use crate::client::client::Client;
use anyhow::{Error, Ok, Result};
use libloading::{Library, Symbol};
use std::ffi::{CStr, CString};
#[warn(unused_imports)]
use std::sync::Arc;
use tracing::info;

use crate::client::default_api_config::{ApiConfig,ConfigType};

pub struct EdgeDNS {
    config: Arc<ApiConfig>,
}

impl EdgeDNS {
    pub fn name(&self) -> &str {
        &self.config.edgemesh_dns_module_name
    }
    pub fn group(&self) -> &str {
        &self.config.edge_mode
    }
    pub fn enable(&self) -> bool {
        self.config.edge_mode_enable
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

    pub fn update_corefile(config: &ApiConfig, clients: &Client) -> Result<()> {
        info!("Updating the EdgeDNS corefile configuration");
        Ok(())
    }

    pub fn new(config: ApiConfig, client: &Client) -> Result<Self, Error> {
        if !config.edge_mode_enable {
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

    pub fn register(config: ApiConfig, client: Client) -> Result<(),Error> {
        // Load the KubeEdge shared library
        let library_path = "../../core/kubeedge-wrapper/libkubeedge.so";
        let library = unsafe {
            // Load the shared library using libloading::Library
            Library::new(library_path).expect("Failed to load libkubeedge.so")
        };

        unsafe {
            // Load the InitKubeEdge function from the shared library
            let register: Symbol<unsafe extern "C" fn(*const i8) -> *const i8> = library
                .get(b"Register\0")
                .expect("Failed to load InitKubeEdge");

            // Path to the configuration file
            let config_path = CString::new("/path/to/config").expect("CString::new failed");

            // Call the InitKubeEdge function
            let result_ptr = register(config_path.as_ptr());
            let result_str = CStr::from_ptr(result_ptr).to_string_lossy();

            println!("Result from InitKubeEdge: {}", result_str);
        }
        Ok(())
    }
    
}
