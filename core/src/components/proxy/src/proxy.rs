/* 
Cortexflow proxy is the main proxy in cortexbrain. Features:
- Caching
- Automatic prometheus metrics export
- Load balancing 
- Service discovery
*/
use anyhow::Error;
use kube::Client;
use tracing::{info, error, instrument};
use dashmap::DashMap;
use std::{env, net::UdpSocket, sync::Arc, time::Instant};
use prometheus::{Encoder, TextEncoder};
use warp::Filter;
use crate::vars::{DNS_REQUEST, DNS_RESPONSE_TIME};
use shared::apiconfig::EdgeProxyConfig;

#[derive(Debug)]
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
    pub async fn start(&self) {
        if !self.proxy_config.enable {
            error!("Proxy is not running");
        } else {
            self.run().await;
        }
    }

    #[instrument]
    pub async fn run(&self) {
        info!("Cortexflow Proxy is running");
        let dns_server = env::var("DNS_SERVER_HOST").unwrap_or_else(|_| "dns-service.default.svc.cluster.local:53".to_string());
        
        let socket = UdpSocket::bind("0.0.0.0:5053").expect("Error in socket bind");
        let cache = Arc::new(DashMap::new());

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

        let mut buffer = [0u8; 512];

        loop {
            let (len, addr) = socket.recv_from(&mut buffer).expect("Error receiving packet");
            let query = &buffer[..len];
            
            info!("Received {} bytes from {}", len, addr);

            // write in prometheus metrics
            DNS_REQUEST.with_label_values(&[&addr.to_string()]).inc();
            let start_time = Instant::now();
            
            let response = Proxy::handle_server_request(query, &dns_server, &cache, &socket).await;
            let duration = start_time.elapsed().as_secs_f64();
            DNS_RESPONSE_TIME.with_label_values(&["dns-server"]).observe(duration);

            if let Err(e) = socket.send_to(&response, addr) {
                error!("Error sending DNS request: {:?}", e);
            }
        }
    }

    pub async fn handle_server_request(
        query: &[u8],
        dns_server: &str,
        cache: &Arc<DashMap<Vec<u8>, Vec<u8>>>,
        socket: &UdpSocket,
    ) -> Vec<u8> {
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
                error!("Error in receiving DNS response: {:?}", e);
                return Vec::new();
            }
        };

        let response = buf[..len].to_vec();
        cache.insert(query.to_vec(), response.clone());
        response
    }
}
