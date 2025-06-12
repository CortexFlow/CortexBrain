/* Implementation */
//TODO: implement loadbalancer function 

use crate::discovery::ServiceDiscovery;
use crate::messaging;
use crate::messaging::MexType;
use crate::messaging::{
    ignore_message_with_no_service, produce_incoming_message, produce_unknown_message,
    produce_unknown_message_udp, send_fail_ack_message, send_outcoming_message,
    send_outcoming_message_udp, send_success_ack_message,
};
use anyhow::{Error, Result};
use aya::Ebpf;
use anyhow::Context;
use tokio::signal;
use tokio::fs;
use aya_log::EbpfLogger;
use aya::programs::{Xdp, XdpFlags};
use prometheus::{Encoder, TextEncoder};
use std::sync::{Arc};
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::UdpSocket;
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, warn,error, info};
use warp::Filter;
use shared::apiconfig::EdgeProxyConfig;
use std::path::Path;
use aya::maps::{HashMap as UserSpaceMap, MapData};
use crate::shared_struct::BackendPorts;
use tokio::sync::RwLock;


const BPF_PATH : &str = "BPF_PATH";

pub struct Loadbalancer<'a> {
    proxy_config: Arc<EdgeProxyConfig>,
    service_discovery: Arc<tokio::sync::RwLock<ServiceDiscovery<'a>>>,
    backends: Arc<RwLock<UserSpaceMap<MapData, u16, BackendPorts>>>,
}

impl<'a> Loadbalancer<'a>{
    
    pub async fn new(proxycfg: EdgeProxyConfig,cache_map: ServiceDiscovery<'a>,backends_list: UserSpaceMap<MapData, u16, BackendPorts>
) -> Result<Self, Error> {
        Ok(Loadbalancer {
            proxy_config: Arc::new(proxycfg),
            service_discovery: Arc::new(cache_map.into()),
            backends: Arc::new(backends_list.into()),
        })
    }

    pub async fn run(&self) -> Result<(), Error> {
        let bpf_path= std::env::var(BPF_PATH).context("BPF_PATH environment variable required")?;
        let data = fs::read(Path::new(&bpf_path)).await.context("failed to load file from path")?;
        let mut bpf = aya::Ebpf::load(&data).context("failed to load data from file")?;
        EbpfLogger::init(&mut bpf).context("Cannot initialize ebpf logger");

        //extract the bpf program "xdp-hello" from builded binaries
        info!("loading xdp program");
        let program: &mut Xdp = bpf.program_mut("xdp_hello").unwrap().try_into()?;
        program.load().context("Failed to laod XDP program")?; //load the program
    
        info!("Starting program");
        program
            .attach("eth0", XdpFlags::default())
            .context("Failed to attach XDP program with default flags to interface eth0")?;
        info!("Cortexflow Intelligent Loadbalancer is running");

        //waiting for signint (ctrl-c) to shutdown the program
        info!("Waiting for Ctrl-C...");

        // Start udpsocket
        let socket = UdpSocket::bind("0.0.0.0:5053").await?;
        info!("Udp socket bound to {}", socket.local_addr()?);

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

        let socket = Arc::new(socket);
        let socket_clone = socket.clone();

        let mut buffer = [0u8; 512];
        loop {
            match socket_clone.recv_from(&mut buffer).await {
                Ok((len, addr)) => {
                    let query = &buffer[..len];
                    info!("Received {} bytes from sender: {}", len, addr);

                    let response = self
                        .handle_udp_connection(query, &socket_clone, addr, 5053)
                        .await;

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
                    .write()
                    .await
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
}
