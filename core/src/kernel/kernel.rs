/* Resource
https://github.com/EmilHernvall/dnsguide/blob/master/chapter1.md
*/

/* CoreDNS-->Dns resolver di Kubernetes */
/* Kubernetes in rust:
    https://www.shuttle.dev/blog/2024/10/22/using-kubernetes-with-rust
*/
#[allow(unused_imports)]
use crate::client::client::Client;
use anyhow::{Error, Result};
use libloading::{Library, Symbol};
use std::ffi::{CStr, CString};
use std::sync::Arc;

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

use crate::client::apiconfig::EdgeDNSConfig;
use crate::client::default_api_config::ApiConfig;
use crate::kernel::corefile::update_corefile;

#[derive(Debug)]
pub struct EdgeDNS {
    config: Arc<ApiConfig>,
    edgednsconfig: Arc<EdgeDNSConfig>,
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
        // creates the proxy server using tokio crate
        info!("EdgeDNS is running ");
        
        //cache_dns_enable
        if self.edgednsconfig.cache_dns.clone().unwrap().enable {
            info!("Running TrustDNS as a cache DNS server");
        } else {
            info!("Running TrustDNS as a local DNS server");
        }
    
        let addr: SocketAddr = "127.0.0.1:5053".parse().unwrap(); 
    
        // TODO: automatic select address
        //TODO: add support for recursion
        //TODO: add auto port recognition if the port is not available
    
        let socket = UdpSocket::bind(addr).await.unwrap();
        info!("Listening for DNS requests on {}", addr);
    
        let local_name = "example.com."; // Nome del dominio che stai parsificando
        let origin = Name::root(); // Usa il dominio radice come origin per un nome assoluto
    
        let authority = Arc::new(InMemoryAuthority::empty(
            Name::parse(local_name, Some(&origin)).expect("Failed to parse domain name"),
            ZoneType::Primary, // Tipo di zona
            false,             // Usa Some(false) invece di false
        ));
    
        // Crea un record DNS
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
                Err(anyhow::anyhow!("Shutting down the server")) // Errore con anyhow::Error
            }
        };

        // Gestisci il risultato
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
        // Crea un futuro che aspetta il segnale SIGINT (Ctrl + C)
        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("Failed to listen for Ctrl + C signal");
            info!("Ctrl + C received, shutting down...");
        };
    
        // Usa `tokio::select!` per attendere il primo futuro che si completa
        tokio::select! {
            _ = ctrl_c => {
                // Se Ctrl + C viene premuto, restituisci un errore per indicare l'arresto
                Err("Ctrl + C received, shutting down".to_string())
            }
        }
    }
        
    #[instrument]
    pub async fn shutdown(&self) {
        info!("Shutting down the EdgeDNS ");
    

        // Operazioni di pulizia
        info!("Shutting down EdgeDNS server");
        

        // Pulizia delle risorse (se necessario)
        if self.edgednsconfig.kube_api_config.clone().unwrap().delete_kube_config {
            if let Err(err) = fs::remove_file("/path/to/temp/kubeconfig") {
                error!("Failed to delete kubeconfig: {}", err);
            }
        }

        info!("EdgeDNS shutdown complete.");
    }

    pub async fn new(
        config: ApiConfig,
        edgednscfg: EdgeDNSConfig,
        client: Arc<Client>,
    ) -> Result<Self, Error> {
        if !config.edge_mode_enable {
            return Ok(EdgeDNS {
                config: Arc::new(config),
                edgednsconfig: Arc::new(edgednscfg),
            });
        }

        // Update Corefile if EdgeDNS is enabled
        update_corefile(edgednscfg.clone(), &client.as_ref().clone()).await?; // Dereferenziamento dell'Arc<Client> e passaggio as_ref

        /* Reference as_ref:
           https://doc.rust-lang.org/std/convert/trait.AsRef.html
        */
        Ok(EdgeDNS {
            config: Arc::new(config),
            edgednsconfig: Arc::new(edgednscfg),
        })
    }

    //registers a service

    pub fn register(config: ApiConfig, client: Client) -> Result<(), Error> {
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
