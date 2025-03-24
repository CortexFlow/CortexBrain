/*
https://iximiuz.com/en/posts/service-discovery-in-kubernetes/
https://microservices.io/patterns/client-side-discovery.html
https://medium.com/@dmosyan/service-mesh-and-service-discovery-e512253d8a17
contains the client side service discovery implementation
*/

use crate::messaging;
use crate::messaging::MexType;
use crate::metrics::{DNS_REQUEST, DNS_RESPONSE_TIME};
use anyhow::Error ;
use dashmap::DashMap;
use k8s_openapi::api::core::v1::{Pod, Service};
use kube::api::ListParams;
use kube::{Client, api::Api};
use serde_json::{json, to_vec};
use std::collections::BTreeMap;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, UdpSocket};
use tokio::time::timeout;
use tracing::{debug, error, info, warn};
use std::result::Result::Ok;


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

    /*
        Destination resolver:
        Args: service_name, namespace
        Return: service endpoint


    */
    pub async fn resolve_service_destination(
        &self,
        service_name: &str,
        namespace: &str,
        port: i32,
    ) -> Option<String> {
        // Check the cache first
        if let Some(cached) = self.service_cache.get(service_name) {
            info!("Service {:?} found in cache: {:?}", service_name, cached);
            return Some(cached.clone());
        }

        let services: Api<Service> = Api::namespaced(self.kube_client.clone(), namespace);
        let pods: Api<Pod> = Api::namespaced(self.kube_client.clone(), namespace);

        debug!(
            "Fetching service {} from namespace {}",
            service_name, namespace
        );

        // Fetch the service endpoint directly
        self.fetch_service_endpoint_from_kubeapi(service_name, services, pods, namespace, port)
            .await
    }

    /*
       Resolver function:
       Args: service_name, namespace
       Return: service address or None
    */
    async fn resolve_service_address(
        &self,
        service_name: &str,
        namespace: &str,
        port: i32,
    ) -> Option<String> {
        match self
            .resolve_service_destination(service_name, namespace, port)
            .await
        {
            Some(service_address) => {
                info!(
                    "Resolved service address for {}: {}",
                    service_name, service_address
                );
                Some(service_address)
            }
            None => {
                error!(
                    "Service {} not found in namespace {}",
                    service_name, namespace
                );
                None
            }
        }
    }

    /*
        fetch service endpoint from the KUBERNETES-API
        Args: service name, service_api, namespace
        Return: service endpoint
    */
    async fn fetch_service_endpoint_from_kubeapi(
        &self,
        service_name: &str,
        service_api: Api<Service>,
        pod_api: Api<Pod>,
        namespace: &str,
        communication_port: i32, //can be udp or tcp port
    ) -> Option<String> {
        // retrieve the service
        let service = match service_api.get(service_name).await {
            Ok(service) => service,
            Err(e) => {
                error!(
                    "Service {} not found in namespace {}: {:?}",
                    service_name, namespace, e
                );
                return None;
            }
        };

        // return the service selector
        let selector_map = match service
            .spec
            .as_ref()
            .and_then(|spec| spec.selector.as_ref())
        {
            Some(selector) => selector,
            None => {
                error!("No selector found for service {}", service_name);
                return None;
            }
        };

        // Convert the selector to a string format
        let selector = self.labels_to_selector(selector_map);

        // find the pods that match the selector
        let pods = match pod_api.list(&ListParams::default().labels(&selector)).await {
            Ok(pods) => pods,
            Err(e) => {
                error!(
                    "Failed to fetch pods for service {} in namespace {}: {:?}",
                    service_name, namespace, e
                );
                return None;
            }
        };

        // Select the first pod available
        // TODO: more advanced load balancing techniques needed here
        if let Some(pod) = pods.items.first() {
            if let Some(pod_ip) = pod
                .status
                .as_ref()
                .and_then(|status| status.pod_ip.as_ref())
            {
                info!("Pod IP for service {}: {}", service_name, pod_ip);
                let endpoint = format!("{}:{}", pod_ip, communication_port);
                info!(
                    "Resolved endpoint for service {}: {}",
                    service_name, endpoint
                );
                // add to service cache
                self.service_cache
                    .insert(service_name.to_string(), endpoint.clone());
                return Some(endpoint);
            } else {
                error!(
                    "No Pod IP defined for pod {}",
                    pod.metadata.name.as_deref().unwrap_or("unknown")
                );
            }
        } else {
            error!("No pods found for service {}", service_name);
        }

        None
    }

    fn labels_to_selector(&self, labels: &BTreeMap<String, String>) -> String {
        labels
            .iter()
            .map(|(key, value)| format!("{}={}", key, value))
            .collect::<Vec<String>>()
            .join(",")
    }
    //directly register a service in the cache
    pub fn register_service(&self, service_id: String, endpoint: String) {
        self.service_cache.insert(service_id, endpoint);
    }

    //directly get a service from the cache
    pub fn get_service(&self, service_id: &str) -> Option<String> {
        self.service_cache.get(service_id).map(|v| v.clone())
    }

    //TCP RESPONSE
    pub async fn send_tcp_request(
        &self,
        service_name: &str,
        namespace: &str,
        payload: &[u8],
        port: i32,
    ) -> Option<Vec<u8>> {
        // Resolves the address of the service
        let target_service = match self.resolve_service_destination(service_name, namespace, port).await {
            Some(addr) => addr,
            None => {
                error!("Service {} not found in namespace {}", service_name, namespace);
                return None;
            }
        };
    
        // Convert the address in a socket address
        let target_addr: SocketAddr = match target_service.parse() {
            Ok(addr) => addr,
            Err(e) => {
                error!("Invalid target address: {}", e);
                return None;
            }
        };
    
        // TCP connection to the service
        match TcpStream::connect(target_addr).await {
            Ok(mut stream) => {
                info!("Connected to service at {}", target_addr);
    
                // Create the json message
                info!("Message waiting to be forwarded:{:?}",&payload);
                
                let response = json!({
                    "message": String::from_utf8_lossy(payload)
                });

                let msg_forwarded_serialized = match to_vec(&response) {
                    Ok(data) => data,
                    Err(e) => {
                        error!("Failed to serialize request: {}", e);
                        return None;
                    }
                };
                let response_message = messaging::create_message(
                    &service_name,
                    MexType::Outcoming,
                    &msg_forwarded_serialized,
                );
    
                // send the message
                if let Err(e) = stream.write_all(&response_message).await {
                    error!("Failed to send request to {}: {}", target_addr, e);
                    return None;
                }
                info!("Request sent to {}", target_addr);
    
                let client_addr = match stream.peer_addr() {
                    Ok(addr) => addr,
                    Err(e) => {
                        error!("Cannot return client address: {}", e);
                        return None;
                    }
                };
    
                let mut buffer = vec![0u8; 1024];
    
                // wait for the response with a timeout timer
                match timeout(Duration::from_secs(10), stream.read(&mut buffer)).await {
                    Ok(Ok(len)) if len > 0 => {
                        let response_data = buffer[..len].to_vec();
                        info!("Received response from {} ({} bytes)", client_addr, len);
                        Some(response_data)
                    }
                    Ok(_) => {
                        warn!("Empty response received from {}", client_addr);
                        None
                    }
                    Err(e) => {
                        error!("TCP response timed out: {}", e);
                        None
                    }
                }
            }
            Err(e) => {
                error!("Failed to connect to {}: {}", target_addr, e);
                None
            }
        }
    }
    //UDP RESPONSE
    pub async fn wait_for_udp_response(
        &self,
        service_name: &str,
        namespace: &str,
        payload: &[u8],
        port: i32,
        client_addr: SocketAddr,
    ) -> Option<Vec<u8>> {
        // Resolve the service name
        let target_service = match self
            .resolve_service_destination(service_name, namespace, port)
            .await
        {
            Some(addr) => addr,
            None => {
                error!(
                    "Service {} not found in namespace {}",
                    service_name, namespace
                );
                return None; // return None if service not found
            }
        };
    
        // Parse target_service into SocketAddr if it's a string
        let target_addr = match target_service.to_socket_addrs() {
            Ok(mut addrs) => match addrs.next() {
                Some(addr) => addr,
                None => {
                    error!("Could not resolve address for {}", target_service);
                    return None;
                }
            },
            Err(e) => {
                error!("Failed to parse socket address: {}", e);
                return None;
            }
        };
    
        // initialize the udp socket
        // bind to a random port
        let socket = match UdpSocket::bind("0.0.0.0:0").await {
            Ok(socket) => socket,
            Err(e) => {
                error!("Failed to bind UDP socket: {}", e);
                return None;
            }
        };
    
        // Allow the socket to receive from any address, not just the target
        // This is important for UDP where responses might come from different ports
        if let Err(e) = socket.set_broadcast(true) {
            error!("Failed to set socket to broadcast mode: {}", e);
            return None;
        }
    
        DNS_REQUEST
            .with_label_values(&[&target_service.to_string()])
            .inc();
    
        // Sends the payload to the destination service
        let response = json!({
            "message": String::from_utf8_lossy(payload)
        });
    
        let serialized_response = match to_vec(&response) {
            Ok(bytes) => bytes,
            Err(e) => {
                error!("cannot serialize udp response: {}", e);
                return None;
            }
        };
    
        let response_message = messaging::create_message(
            &service_name,
            MexType::Outcoming,
            &serialized_response,
        );
        
        info!(
            "([{}]->[{}])sending response (outcoming) message to : {}",
            target_addr, client_addr, client_addr
        );
        
        if let Err(e) = socket.send_to(&response_message, &client_addr).await {
            error!(
                "Error sending UDP response to target service {}: {}",
                target_addr, e
            );
            return None;
        }else {
            info!(
                "UDP response successfully sent to {} from {} with message {:?} ",
                client_addr,target_addr ,response_message
            );
        }
        
        
    
        let start_time = Instant::now();
        let client_ip = client_addr.ip();
        
        // Set up timeout for receiving the response
        match timeout(
            Duration::from_secs(10), // 10 second timeout
            async {
                // Get the response from the destination service
                // We're willing to accept a response from any address associated with the target service
                let mut buffer = [0u8; 1024];
                loop {
                    match socket.recv_from(&mut buffer).await {
                        Ok((len, addr)) => {
                            // Check if this is a response from our target (any port)
                            //TODO: is this part safe?
                            if addr.ip() == client_ip {
                                if len == 0 {
                                    warn!(
                                        "Received null UDP response from {} at address {}",
                                        client_addr, addr
                                    );
                                    return None;
                                } else {
                                    info!(
                                        "Received UDP response from {}({} bytes) at address:{}",
                                        client_addr, len, addr
                                    );
                                    
                                    let response_data = buffer[..len].to_vec();
                                    return Some(response_data);
                                }
                            } else {
                                // Message from another address, ignore and keep waiting
                                info!("Received message from unexpected address: {}, continuing to wait", addr);
                                continue;
                            }
                        }
                        Err(e) => {
                            error!("Error receiving UDP response: {}", e);
                            return None;
                        }
                    }
                }
            }
        ).await {
            Ok(Some(response_data)) => {
                let duration = start_time.elapsed().as_secs_f64();
                DNS_RESPONSE_TIME
                    .with_label_values(&["service_discovery_udp"])
                    .observe(duration);
                
                // Forward the response to the original client
                info!("Forwarding the response to the client: {client_addr}");
                if let Err(e) = socket.send_to(&response_data, client_addr).await {
                    error!("Error sending UDP response to client: {}", e);
                } else {
                    info!("Response forwarded to client {}", client_addr);
                }
                
                Some(response_data)
            }
            Ok(None) => {
                warn!("None UDP response from {}", target_addr);
                None
            }
            Err(e) => {
                error!("UDP response timed out with error: {}", e);
                None
            }
        }
    }
}
