/*
Cortexflow proxy is the main proxy in cortexbrain. Features:
- Caching ✅
- UDP/TCP traffic ✅
- Automatic prometheus metrics export ✅
- Load balancing ❌
- Service discovery ✅
*/
use crate::discovery::ServiceDiscovery;
use crate::messaging;
use crate::metrics::{DNS_REQUEST, DNS_RESPONSE_TIME};
use anyhow::{Error, Result};
use dashmap::DashMap;
use prometheus::{Encoder, TextEncoder};
use shared::apiconfig::EdgeProxyConfig;
use std::{net::UdpSocket, sync::Arc, time::Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{error, info};
use warp::Filter;
#[derive(Clone)]
pub struct Proxy {
    proxy_config: Arc<EdgeProxyConfig>,
    service_discovery: Arc<ServiceDiscovery>,
}

impl Proxy {
    pub async fn new(proxycfg: EdgeProxyConfig) -> Result<Self, Error> {
        let service_discovery = ServiceDiscovery::new().await?;
        Ok(Proxy {
            proxy_config: Arc::new(proxycfg),
            service_discovery: Arc::new(service_discovery),
        })
    }

    pub async fn get_info(&self) {
        info!("Enable: {:?}", self.proxy_config.enable);
        info!("Listen interface: {:?}", self.proxy_config.listen_interface);
    }

    pub async fn start(&self) -> Result<(), Error> {
        if !self.proxy_config.enable {
            error!("Proxy is not running");
            return Ok(());
        }
        self.run().await
    }

    pub async fn run(&self) -> Result<(), Error> {
        info!("Cortexflow Proxy is running");

        // Start udpsocket
        let socket = UdpSocket::bind("0.0.0.0:5053")?;
        info!("Socket bound to {}", socket.local_addr()?);

        // Start tcp_listener
        let tcp_listener = TcpListener::bind("0.0.0.0:5054").await?;
        info!("Tcp listener bound to {}", tcp_listener.local_addr()?);

        //start the cache
        let cache = Arc::new(DashMap::new());
        let cache_clone = cache.clone();

        let metrics_route = warp::path!("metrics").map(|| {
            let mut buffer = Vec::new();
            let encoder = TextEncoder::new();
            let metrics_families = prometheus::gather();
            encoder.encode(&metrics_families, &mut buffer).unwrap();
            warp::reply::with_header(
                String::from_utf8(buffer).unwrap(),
                "Content-Type",
                "text/plain; charset=utf-8",
            )
        });

        tokio::spawn(async move {
            warp::serve(metrics_route).run(([0, 0, 0, 0], 9090)).await;
        });

        // Clone all the necessary for the tcp task
        let proxy_clone = self.clone();

        tokio::spawn(async move {
            while let Ok((stream, _)) = tcp_listener.accept().await {
                let cache = cache_clone.clone();
                let proxy = proxy_clone.clone();

                tokio::spawn(async move {
                    Self::handle_tcp_connection_static(proxy, stream, cache).await;
                });
            }
        });

        let socket = Arc::new(socket);
        let socket_clone = socket.clone();

        let mut buffer = [0u8; 512];
        loop {
            match socket_clone.recv_from(&mut buffer) {
                Ok((len, addr)) => {
                    let query = &buffer[..len];
                    info!("Received {} bytes from {}", len, addr);

                    //dns request metrics export
                    DNS_REQUEST.with_label_values(&[&addr.to_string()]).inc();
                    let start_time = Instant::now();

                    let response = self.handle_udp_connection(query, &socket_clone, addr).await;
                    let duration = start_time.elapsed().as_secs_f64();
                    //dns response time metrics export
                    DNS_RESPONSE_TIME
                        .with_label_values(&["service_discovery"])
                        .observe(duration);

                    if let Err(e) = socket_clone.send_to(&response, addr) {
                        error!("Error sending response: {:?}", e);
                    }
                }
                Err(e) => {
                    error!("Error receiving packet: {}", e);
                }
            }
        }
    }

    pub async fn handle_udp_connection(
        &self,
        query: &[u8],
        socket: &UdpSocket,
        addr: std::net::SocketAddr,
    ) -> Vec<u8> {
        // Reply only to test message
        if query == b"Hi CortexFlow" {
            let response = b"Hi user!".to_vec();
            info!("sending response...");
            if let Err(e) = socket.send_to(&response, addr) {
                error!("Error sending response: {:?}", e);
            }
            info!("if you can see this code the response sender is fine!"); //logging message
            return response;
        }

        // Extract the service and the payload from the incoming request
        let (service_name, payload) = match messaging::extract_service_name_and_payload(query) {
            Some((name, payload)) => (name, payload),
            None => {
                error!("Invalid request format");
                return Vec::new(); // return a empty message 
            }
        };

        // use service discovery to resolve the request
        let namespace = "cortexflow";
        let response = self
            .service_discovery
            .send_udp_response(service_name, namespace, payload)
            .await;

        // send a response to the client
        if let Err(e) = socket.send_to(&response, addr) {
            error!("Error sending UDP response: {:?}", e);
        }
        //TODO: add checks for debugging purpouses
        response
    }

    // handles the tcp connection
    pub async fn handle_tcp_connection_static(
        proxy: Self,
        mut stream: TcpStream,
        cache: Arc<DashMap<Vec<u8>, Vec<u8>>>,
    ) {
        let mut buffer = [0u8; 1024];

        match stream.read(&mut buffer).await {
            Ok(size) if size > 0 => {
                let query = &buffer[..size];
                info!("Received TCP message: {:?}", query);

                // Extract the service and the payload from the incoming request
                match messaging::extract_service_name_and_payload(query) {
                    Some((service_name, payload)) if !service_name.is_empty() => {
                        //use the service discovery to resolve the requests
                        let namespace = "cortexflow";
                        match proxy
                            .service_discovery
                            .send_tcp_response(service_name, namespace, payload)
                            .await
                        {
                            Some(response) => {
                                if let Err(e) = stream.write_all(&response).await {
                                    error!("Error sending TCP response: {}", e);
                                }
                            }
                            None => {
                                error!(
                                    "Service {} not found in namespace {}",
                                    service_name, namespace
                                );
                            }
                        }
                    }
                    Some((_, payload)) => {
                        info!("Ignoring response message: {:?}", payload);
                    }
                    None => {
                        error!("Invalid request format");
                    }
                }

                // ACK confirmation message
                let ack_message = b"Message Received";
                if let Err(e) = stream.write_all(ack_message).await {
                    error!("Error sending TCP acknowledgment: {}", e);
                }
            }
            Ok(_) => {
                //empty message fallback
                info!("Received empty message");
            }
            Err(e) => {
                //error message fallback
                error!("TCP Error: {}", e);
            }
        }
    }
}
