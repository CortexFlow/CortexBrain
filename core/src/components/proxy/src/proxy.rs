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
use crate::messaging::MexType;
use crate::messaging::{
    ignore_message_with_no_service, produce_incoming_message, produce_unknown_message,
    produce_unknown_message_udp, send_fail_ack_message, send_outcoming_message,
    send_outcoming_message_udp, send_success_ack_message,
};
use crate::metrics::{DNS_REQUEST, DNS_RESPONSE_TIME};
use anyhow::{Error, Result};
use dashmap::DashMap;
use prometheus::{Encoder, TextEncoder};
use shared::apiconfig::EdgeProxyConfig;
use std::{sync::Arc, time::Instant};
use tokio::io::AsyncReadExt;
use tokio::net::UdpSocket;
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, error, info};
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
        debug!("Cortexflow Proxy is running");

        // Start udpsocket
        let socket = UdpSocket::bind("0.0.0.0:5053").await?;
        debug!("Socket bound to {}", socket.local_addr()?);

        // Start tcp_listener
        let tcp_listener = TcpListener::bind("0.0.0.0:5054").await?;
        debug!("Tcp listener bound to {}", tcp_listener.local_addr()?);

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
                    Self::handle_tcp_connection_static(proxy, stream, 5054, cache).await;
                });
            }
        });

        let socket = Arc::new(socket);
        let socket_clone = socket.clone();

        let mut buffer = [0u8; 512];
        loop {
            match socket_clone.recv_from(&mut buffer).await {
                Ok((len, addr)) => {
                    let query = &buffer[..len];
                    info!("Received {} bytes from sender: {}", len, addr);

                    //dns request metrics export
                    DNS_REQUEST.with_label_values(&[&addr.to_string()]).inc();
                    let start_time = Instant::now();

                    let response = self
                        .handle_udp_connection(query, &socket_clone, addr, 5053)
                        .await;
                    let duration = start_time.elapsed().as_secs_f64();
                    //dns response time metrics export
                    DNS_RESPONSE_TIME
                        .with_label_values(&["service_discovery"])
                        .observe(duration);

                    if let Err(e) = socket_clone.send_to(&response, addr).await {
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
        sender_addr: std::net::SocketAddr,
        port: i32,
    ) -> Vec<u8> {
        // Extract service name, direction, and payload
        let (direction, service_name, payload) =
            match messaging::extract_service_name_and_payload(query) {
                Some((direction, name, payload)) if !name.is_empty() => (direction, name, payload),
                _ => {
                    error!("Invalid UDP request format");
                    return Vec::new(); // Return an empty response
                }
            };

        let namespace = "cortexflow";

        match direction {
            MexType::Incoming => {
                info!(
                    "([{}]->[{}]): Processing incoming UDP message from service: {}",
                    sender_addr, service_name, sender_addr
                );

                // Use service discovery to resolve the request and forward response to client
                if let Some(response) = self
                    .service_discovery
                    .wait_for_udp_response(&service_name, namespace, &payload, port, sender_addr)
                    .await
                {
                    if let Err(e) = socket.send_to(&response, sender_addr).await {
                        error!(
                            "([{}]->[{}]):Error sending UDP response : {}",
                            service_name, sender_addr, e
                        );
                    }
                    response
                } else {
                    Vec::new() // Return empty if no response received
                }
            }
            MexType::Outcoming => {
                send_outcoming_message_udp(socket, service_name, sender_addr).await
            }
            _ => produce_unknown_message_udp(socket, service_name, sender_addr).await,
        }
    }

    //TODO: this part needs new debug
    // handles the tcp connection
    pub async fn handle_tcp_connection_static(
        proxy: Self,
        mut stream: TcpStream,
        port: i32,
        cache: Arc<DashMap<Vec<u8>, Vec<u8>>>,
    ) {
        //TODO: add cache check
        let mut buffer = [0u8; 1024];

        match stream.read(&mut buffer).await {
            Ok(size) if size > 0 => {
                let query = &buffer[..size];
                match messaging::extract_service_name_and_payload(query) {
                    Some((direction, service_name, payload)) if !service_name.is_empty() => {
                        if direction == MexType::Incoming {
                            //TODO: check this part MexType::incoming case. the logic seems to be redundant

                            info!("Receiving incoming message from sender: {}", service_name);
                            let namespace = "cortexflow";

                            match proxy
                                .service_discovery
                                .wait_for_tcp_response(&service_name, namespace, &payload, port)
                                .await
                            {
                                Some(response) => {
                                    produce_incoming_message(&mut stream, service_name).await;
                                    send_success_ack_message(&mut stream).await;
                                }
                                None => {
                                    error!(
                                        "Service {} not found in namespace {}",
                                        service_name, namespace
                                    );
                                    send_fail_ack_message(&mut stream).await;
                                }
                            }
                        } else if direction == MexType::Outcoming {
                            send_outcoming_message(&mut stream, service_name).await;
                            send_success_ack_message(&mut stream).await;
                        } else {
                            produce_unknown_message(&mut stream, service_name).await;
                            send_success_ack_message(&mut stream).await;
                        }
                    }
                    Some((direction, _, payload)) => {
                        ignore_message_with_no_service(direction, &payload);
                        send_fail_ack_message(&mut stream).await;
                    }
                    None => {
                        error!("Invalid request format");
                        send_fail_ack_message(&mut stream).await;
                    }
                }
            }
            Ok(_) => {
                info!("Received empty message"); //log empty message if the message has no bytes
                send_success_ack_message(&mut stream).await;
            }
            Err(e) => {
                error!("Error: {}", e); //return error if the first match fails
                send_fail_ack_message(&mut stream).await;
            }
        }
    }
}
