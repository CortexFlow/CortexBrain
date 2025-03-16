/*
Cortexflow proxy is the main proxy in cortexbrain. Features:
- Caching
- UDP/TCP traffic
- Automatic prometheus metrics export
- Load balancing
- Service discovery
*/
use crate::vars::{DNS_REQUEST, DNS_RESPONSE_TIME};
use anyhow::{Error, Result};
use dashmap::DashMap;
use prometheus::{Encoder, TextEncoder};
use shared::apiconfig::EdgeProxyConfig;
use std::{env, net::UdpSocket, sync::Arc, time::Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{error, info, instrument};
use warp::Filter;

#[derive(Debug, Clone)]
pub struct Proxy {
    proxy_config: Arc<EdgeProxyConfig>,
}

impl Proxy {
    pub async fn new(proxycfg: EdgeProxyConfig) -> Result<Self, Error> {
        Ok(Proxy {
            proxy_config: Arc::new(proxycfg),
        })
    }

    #[instrument]
    pub async fn get_info(&self) {
        info!("Enable: {:?}", self.proxy_config.enable);
        info!("Listen interface: {:?}", self.proxy_config.listen_interface);
    }

    #[instrument]
    pub async fn start(&self) -> Result<(), Error> {
        if !self.proxy_config.enable {
            error!("Proxy is not running");
            return Ok(());
        }
        self.run().await
    }

    #[instrument]
    pub async fn run(&self) -> Result<(), Error> {
        info!("Cortexflow Proxy is running");
        let dns_server = env::var("DNS_SERVER_HOST")
            .unwrap_or_else(|_| "kube-dns.kube-system.svc.cluster.local".to_string());

        // Start udpsocket
        let socket = UdpSocket::bind("0.0.0.0:5053")?;
        info!("Socket bound to {}", socket.local_addr()?);

        // Start tcp_lister
        let tcp_listener = TcpListener::bind("0.0.0.0:5054").await?;
        info!("Tcp listener bound to {}", tcp_listener.local_addr()?);

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
        let dns_server_tcp = dns_server.clone();

        tokio::spawn(async move {
            while let Ok((stream, _)) = tcp_listener.accept().await {
                let cache = cache_clone.clone();
                let dns_server = dns_server_tcp.clone();
                let proxy = proxy_clone.clone();

                tokio::spawn(async move {
                    Self::handle_tcp_connection_static(proxy, stream, dns_server, cache).await;
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

                    DNS_REQUEST.with_label_values(&[&addr.to_string()]).inc();
                    let start_time = Instant::now();

                    let response = self
                        .handle_server_request(query, &dns_server, &cache, &socket_clone, addr)
                        .await;
                    let duration = start_time.elapsed().as_secs_f64();
                    DNS_RESPONSE_TIME
                        .with_label_values(&["dns_server"])
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

    pub async fn handle_server_request(
        &self,
        query: &[u8],
        dns_server: &str,
        cache: &Arc<DashMap<Vec<u8>, Vec<u8>>>,
        socket: &UdpSocket,
        addr: std::net::SocketAddr,
    ) -> Vec<u8> {
        // Reply only to test messages
        if query == b"Hi CortexFlow" {
            let response = b"Hi user!".to_vec();
            info!("sending response...");
            if let Err(e) = socket.send_to(&response, addr) {
                error!("Error sending response: {:?}", e);
            }
            info!("if you can see this code the response sender is fine!");
            return response;
        }

        if let Some(response) = cache.get(query) {
            return response.clone();
        }

        if let Err(e) = socket.send_to(query, dns_server) {
            error!("Error sending DNS request: {:?}", e);
            return Vec::new();
        }

        let mut buf = [0u8; 512];
        let (len, _) = match socket.recv_from(&mut buf) {
            Ok(res) => res,
            Err(e) => {
                error!("Error receiving DNS response: {:?}", e);
                return Vec::new();
            }
        };

        let response = buf[..len].to_vec();
        cache.insert(query.to_vec(), response.clone());
        response
    }

    // new static method for handle_tcp_connection function
    pub async fn handle_tcp_connection_static(
        _proxy: Self, // the owned Proxy instance.
        mut stream: TcpStream,
        dns_server: String,
        cache: Arc<DashMap<Vec<u8>, Vec<u8>>>,
    ) {
        let mut buffer = [0u8; 1024];

        match stream.read(&mut buffer).await {
            Ok(size) if size > 0 => {
                let query = &buffer[..size];
                info!("Received TCP message: {:?}", query);

                // Se il messaggio è "Hi CortexFlow TCP", risponde senza inoltrarlo
                if query == b"Hi TCP" {
                    let response = b"Hi TCP user!".to_vec();
                    info!("Sending response...");

                    if let Err(e) = stream.write_all(&response).await {
                        error!("Error sending TCP response: {}", e);
                    }
                    return;
                }

                // Controlla se la richiesta è in cache
                if let Some(response) = cache.get(query) {
                    let _ = stream.write_all(&response).await;
                    return;
                }

                // Se non è in cache, inoltra la richiesta al server DNS
                let response = Self::send_tcp_request(query, &dns_server).await;
                let _ = stream.write_all(&response).await;
            }
            Ok(_) => {
                info!("Received empty message");
            }
            Err(e) => {
                error!("TCP Error: {}", e);
            }
        }
    }

    pub async fn send_tcp_request(query: &[u8], dns_server: &str) -> Vec<u8> {
        match TcpStream::connect(dns_server).await {
            Ok(mut dns_stream) => {
                if let Err(e) = dns_stream.write_all(query).await {
                    error!("Error sending tcp request: {}", e);
                    return Vec::new();
                }

                let mut response = vec![0u8; 1024];
                match dns_stream.read(&mut response).await {
                    Ok(size) => response[..size].to_vec(),
                    Err(e) => {
                        error!("Error reading tcp request: {}", e);
                        return Vec::new();
                    }
                }
            }
            Err(e) => {
                error!("Error sending TCP request to the DNS Server: {}", e);
                return Vec::new();
            }
        }
    }
}
