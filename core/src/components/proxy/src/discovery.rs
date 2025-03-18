/*
https://iximiuz.com/en/posts/service-discovery-in-kubernetes/
https://microservices.io/patterns/client-side-discovery.html
https://medium.com/@dmosyan/service-mesh-and-service-discovery-e512253d8a17
contains the client side service discovery implementation
*/

use anyhow::Error;
use dashmap::DashMap;
use k8s_openapi::api::core::v1::Service;
use kube::{Client, api::Api};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, UdpSocket};
use tracing::{error, info};

/* service discovery structure:
   uses a dns_server-->kube-dns
   service_cache: speed up the discovery process

*/
pub struct ServiceDiscovery {
    kube_client: Client,
    service_cache: Arc<DashMap<String, String>>,
}

impl ServiceDiscovery {
    pub async fn new() -> Result<Self, Error> {
        let kube_client = Client::try_default().await?;
        Ok(ServiceDiscovery {
            kube_client,
            service_cache: Arc::new(DashMap::new()),
        })
    }

    //Destination resolver-->Main component for service discovery
    pub async fn resolve_destination(&self, service_name: &str, namespace: &str) -> Option<String> {
        if let Some(cached) = self.service_cache.get(service_name) {
            info!("Service {:?} found in cache: {:?}", service_name, cached);
            return Some(cached.clone());
        }

        let services: Api<Service> = Api::namespaced(self.kube_client.clone(), namespace);
        info!(
            "Fetching service {} from namespace {}",
            service_name, namespace
        );

        if let Ok(service) = services.get(service_name).await {
            info!("Service {} found in namespace {}", service_name, namespace);
            if let Some(spec) = service.spec {
                if let Some(cluster_ip) = spec.cluster_ip {
                    info!("Cluster IP for service {}: {}", service_name, cluster_ip);
                    if let Some(ports) = spec.ports {
                        if let Some(port) = ports.first() {
                            let endpoint = format!("{}:{}", cluster_ip, port.port);
                            info!(
                                "Resolved endpoint for service {}: {}",
                                service_name, endpoint
                            );
                            self.service_cache
                                .insert(service_name.to_string(), endpoint.clone());
                            return Some(endpoint);
                        } else {
                            error!("No ports defined for service {}", service_name);
                        }
                    } else {
                        error!("No ports defined for service {}", service_name);
                    }
                } else {
                    error!("No cluster IP defined for service {}", service_name);
                }
            } else {
                error!("No spec defined for service {}", service_name);
            }
        } else {
            error!(
                "Service {} not found in namespace {}",
                service_name, namespace
            );
        }
        None
    }

    //directly register a service in the cache
    pub fn register_service(&self, id: String, endpoint: String) {
        self.service_cache.insert(id, endpoint);
    }

    //directly get a service from the cache
    pub fn get_service(&self, id: &str) -> Option<String> {
        self.service_cache.get(id).map(|v| v.clone())
    }

    //TCP RESPONSE
    pub async fn send_response(
        &self,
        service_name: &str,
        namespace: &str,
        payload: &[u8],
    ) -> Option<Vec<u8>> {
        // Resolve the service name
        let target = match self.resolve_destination(service_name, namespace).await {
            Some(addr) => addr,
            None => {
                error!(
                    "Service {} not found in namespace {}",
                    service_name, namespace
                );
                return None;
            }
        };

        // tcp message forward to the resolved address
        match TcpStream::connect(&target).await {
            Ok(mut stream) => {
                if let Err(e) = stream.write_all(payload).await {
                    error!("Error sending TCP request: {}", e);
                    return None;
                }
                let mut response = vec![0u8; 1024];
                match stream.read(&mut response).await {
                    Ok(size) => Some(response[..size].to_vec()),
                    Err(e) => {
                        error!("Error reading TCP response: {}", e);
                        None
                    }
                }
            }
            Err(e) => {
                error!("Error connecting to {} service: {}", target, e);
                None
            }
        }
    }

    //UDP RESPONSE
    pub async fn send_udp_response(
        &self,
        service_name: &str,
        namespace: &str,
        payload: &[u8],
    ) -> Vec<u8> {
        // Resolve the service name
        let target = match self.resolve_destination(service_name, namespace).await {
            Some(addr) => addr,
            None => {
                error!(
                    "Service {} not found in namespace {}",
                    service_name, namespace
                );
                return Vec::new(); // return an empty response 
            }
        };

        // initialize the udp socket
        let socket = UdpSocket::bind("0.0.0.0:0").await.unwrap();

        // Sends the payload to the destination service
        if let Err(e) = socket.send_to(payload, &target).await {
            error!("Error sending UDP request: {}", e);
            return Vec::new(); //return an empty response if an error occured
        }

        // Get the response from the destination service
        let mut buffer = [0u8; 1024];
        match socket.recv_from(&mut buffer).await {
            Ok((len, _)) => buffer[..len].to_vec(),
            Err(e) => {
                error!("Error receiving UDP response: {}", e);
                Vec::new() //return an empty response if an error occured
            }
        }
    }
}
