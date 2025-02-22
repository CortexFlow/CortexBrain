/* Resource
https://github.com/EmilHernvall/dnsguide/blob/master/chapter1.md
*/

/* CoreDNS-->Dns resolver di Kubernetes */
/* Kubernetes in rust:
    https://www.shuttle.dev/blog/2024/10/22/using-kubernetes-with-rust
*/
#[allow(unused_imports)]

use anyhow::{Error, Result};
use std::sync::Arc;
use kube::Client;
use std::net::SocketAddr;
use tokio::net::UdpSocket;
use tracing::{error,info,warn,instrument};
use trust_dns_server::authority::{AuthorityObject, Catalog};
use trust_dns_server::proto::rr::{Name,Record,RecordType,RData};
use trust_dns_server::authority::ZoneType;
use trust_dns_server::server::ServerFuture;
use trust_dns_server::store::in_memory::InMemoryAuthority;
use trust_dns_server::proto::rr::rdata::A;
use std::net::Ipv4Addr;

use std::fs;
use tokio::signal;
use shared::apiconfig::EdgeDNSConfig;

use crate::corefile::update_corefile;

#[derive(Debug)]
pub struct EdgeDNS {
    edgednsconfig: Arc<EdgeDNSConfig>,
}

impl EdgeDNS {
    pub fn name(&self) -> &str {
        &self.edgednsconfig.module_name
    }
    pub fn group(&self) -> &str {
        &self.edgednsconfig.edge_mode
    }
    pub fn enable(&self) -> bool {
        self.edgednsconfig.enable
    }
    pub fn get_kernel_info(&self) {
        info!("Kernel info:\n");
        info!("name: {}", self.name());
        info!("group: {}", self.group());
        info!("enabled: {}\n", self.enable());
    }
    #[instrument]
    pub async fn start(&self) {
        if self.enable() {
            self.run().await;
        } else {
            warn!("kernel is disabled");
        }
    }
    #[instrument]
    pub async fn run(&self) {

        if !self.enable(){
            error!("EdgeDNS is not enabled");
        }
        info!("EdgeDNS is running ");
        
        //cache_dns_enable
        if self.edgednsconfig.cache_dns.clone().unwrap().enable {
            info!("Running TrustDNS as a cache DNS server");
        } else {
            info!("Running TrustDNS as a local DNS server");
        }
    
        let addr: SocketAddr = "0.0.0.0:5000".parse().unwrap(); //changed the port from 53-->5000 5353 is the alternative port for the dns 
    
        // TODO: automatic select address
        //TODO: add support for recursion
        //TODO: add auto port recognition if the port is not available
    
        let socket = UdpSocket::bind(addr).await.unwrap();
        info!("Listening for DNS requests on {}", addr);
    
        let local_name = "example.com."; 
        let origin = Name::root(); 
    
        let authority = Arc::new(InMemoryAuthority::empty(
            Name::parse(local_name, Some(&origin)).expect("Failed to parse domain name"),
            ZoneType::Primary, // Zone type
            false,            
        ));
    
        // Create a DNS record
        let mut record = Record::with(
            Name::parse("www.example.com.", None).unwrap(),
            RecordType::A,
            self.edgednsconfig.cache_dns.clone().unwrap().cache_ttl,
        );
    
        record.set_data(Some(RData::A(A(Ipv4Addr::new(192, 168, 0, 1))))); 
    
        // Aggiungi il record all'autorità
        authority.upsert(record, 0).await;
    
        let mut catalog = Catalog::new();
        catalog.upsert(
            Name::parse(local_name, Some(&origin))
                .expect("Failed to parse domain name").into(),
            Box::new(authority) as Box<dyn AuthorityObject + Send + Sync>,  // Correzione qui
        );
    
        let mut server = ServerFuture::new(catalog);
        server.register_socket(socket);
    
        // Inizializzazione di un meccanismo di "shutdown" basato su un errore o su un input
        // Esegui la selezione
        let server_result:Result<(), anyhow::Error> = tokio::select! {
            _ = server.block_until_done() => {
                info!("Server stopped gracefully");
                Ok(())  
            },
            _ = self.wait_for_shutdown() => {
                info!("Shutdown command received");
                Err(anyhow::anyhow!("Shutting down the server")) 
            }
        };

        // handle the server_result
        match server_result {
            Ok(_) => {
                info!("Server stopped gracefully");
            }
            Err(err) => {
                error!("Server encountered an error: {}", err);
                self.shutdown().await;  // Chiamata alla funzione di shutdown
            }
        }
    }

    async fn wait_for_shutdown(&self) -> Result<(), String> {
        // wait for sigint for shutting down
        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("Failed to listen for Ctrl + C signal");
            info!("Ctrl + C received, shutting down...");
        };
    
        tokio::select! {
            _ = ctrl_c => {
                // if sigint is triggered shut down the server and reutrn an error msg
                Err("Ctrl + C received, shutting down".to_string())
            }
        }
    }
        
    #[instrument]
    pub async fn shutdown(&self) {
        info!("Shutting down the EdgeDNS ");
    

        info!("Shutting down EdgeDNS server");
        

        // clear the resources
        if self.edgednsconfig.kube_api_config.clone().unwrap().delete_kube_config {
            if let Err(err) = fs::remove_file("/path/to/temp/kubeconfig") {
                error!("Failed to delete kubeconfig: {}", err);
            }
            //TODO: remove the temp files
        }

        info!("EdgeDNS shutdown complete.");
    }

    pub async fn new(
        edgednscfg: EdgeDNSConfig,
        client: Client,
    ) -> Result<Self, Error> {
        // Update Corefile if EdgeDNS is enabled
        update_corefile(edgednscfg.clone(), &client.clone()).await?; 

        /* Reference as_ref:
           https://doc.rust-lang.org/std/convert/trait.AsRef.html
        */
        Ok(EdgeDNS {
            edgednsconfig: Arc::new(edgednscfg),
        })
    }

    //registers a service

    //TODO: delete this part
    /* pub fn register(config: ApiConfig, client: Client) -> Result<(), Error> {
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
    } */
}
