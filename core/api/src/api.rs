#![allow(warnings)]
use prost::bytes::BytesMut;
use std::sync::Mutex;
use tonic::{Request, Response, Status};
use tracing::info;

use aya::{
    maps::{MapData, PerfEventArray},
    util::online_cpus,
};
use std::result::Result::Ok;
use tonic::async_trait;

use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::task;

// *  contains agent api configuration
use crate::agent::{agent_server::Agent, ActiveConnectionResponse, RequestActiveConnections, ConnectionEvent};
use aya::maps::Map;
use bytemuck_derive::Zeroable;
use cortexflow_identity::enums::IpProtocols;
use std::net::Ipv4Addr;
use tracing::warn;

#[repr(C)]
#[derive(Clone, Copy, Zeroable)]
pub struct PacketLog {
    pub proto: u8,
    pub src_ip: u32,
    pub src_port: u16,
    pub dst_ip: u32,
    pub dst_port: u16,
    pub pid: u32,
}
unsafe impl aya::Pod for PacketLog {}

pub struct AgentApi {
    //* event_rx is an istance of a mpsc receiver.
    //* is used to receive the data from the transmitter (tx)
    event_rx: Mutex<mpsc::Receiver<Result<Vec<ConnectionEvent>, Status>>>,
    event_tx: mpsc::Sender<Result<Vec<ConnectionEvent>, Status>>,
}

//* Event sender trait. Takes an event from a map and send that to the mpsc channel
//* using the send_map function
#[async_trait]
pub trait EventSender: Send + Sync + 'static {
    async fn send_event(&self, event: Vec<ConnectionEvent>);
    async fn send_map(
        &self,
        map: Vec<ConnectionEvent>,
        tx: mpsc::Sender<Result<Vec<ConnectionEvent>, Status>>,
    ) {
        let status = Status::new(tonic::Code::Ok, "success");
        let event = Ok(map);

        let _ = tx.send(event).await;
    }
}

// send event function. takes an HashMap and send that using mpsc event_tx
#[async_trait]
impl EventSender for AgentApi {
    async fn send_event(&self, event: Vec<ConnectionEvent>) {
        self.send_map(event, self.event_tx.clone()).await;
    }
}

const BPF_PATH: &str = "BPF_PATH";

//initialize a default trait for AgentApi. Loads a name and a bpf istance.
//this trait is essential for init the Agent.
impl Default for AgentApi {
    //TODO:this part needs a better error handling
    fn default() -> Self {
        // load maps mapdata
        let mapdata = MapData::from_pin("/sys/fs/bpf/maps/events_map")
            .expect("cannot open events_map Mapdata");
        let map = Map::PerfEventArray(mapdata); //creates a PerfEventArray from the mapdata

        //init a mpsc channel
        let (tx, rx) = mpsc::channel(1024);
        let api = AgentApi {
            event_rx: rx.into(),
            event_tx: tx.clone(),
        };

        let mut events_array =
            PerfEventArray::try_from(map).expect("Error while initializing events array");

        //spawn an event reader
        task::spawn(async move {
            let mut net_events_buffer = Vec::new();
            //scan the cpus to read the data

            for cpu_id in online_cpus()
                .map_err(|e| anyhow::anyhow!("Error {:?}", e))
                .unwrap()
            {
                let buf = events_array
                    .open(cpu_id, None)
                    .expect("Error during the creation of net_events_buf structure");

                let buffers = vec![BytesMut::with_capacity(4096); 8];
                net_events_buffer.push((buf, buffers));
            }

            info!("Starting event listener");
            //send the data through a mpsc channel
            loop {
                for (buf, buffers) in net_events_buffer.iter_mut() {
                    match buf.read_events(buffers) {
                        Ok(events) => {
                            //read the events, this function is similar to the one used in identity/helpers.rs/display_events
                            if events.read > 0 {
                                for i in 0..events.read {
                                    let data = &buffers[i];
                                    if data.len() >= std::mem::size_of::<PacketLog>() {
                                        let pl: PacketLog =
                                            unsafe { std::ptr::read(data.as_ptr() as *const _) };
                                        let src = Ipv4Addr::from(u32::from_be(pl.src_ip));
                                        let dst = Ipv4Addr::from(u32::from_be(pl.dst_ip));
                                        let src_port = u16::from_be(pl.src_port as u16);
                                        let dst_port = u16::from_be(pl.dst_port as u16);
                                        let event_id = pl.pid;

                                        match IpProtocols::try_from(pl.proto) {
                                            Ok(proto) => {
                                                info!(
                                            "Event Id: {} Protocol: {:?} SRC: {}:{} -> DST: {}:{}",
                                            event_id, proto, src, src_port, dst, dst_port
                                        );
                                                info!("creating vector for the aggregated data");
                                                let mut evt = Vec::new();
                                                // insert event in the vector
                                                info!("Inserting events into the vector");
                                                //TODO: use a Arc<str> or Box<str> type instead of String type.
                                                //The data doesn't need to implement any .copy() or .clone() trait
                                                // using an Arc<str> type will also waste less resources
                                                evt.push(ConnectionEvent {
                                                    event_id: event_id.to_string(),
                                                    src_ip_port: format!("{}:{}", src, src_port),
                                                    dst_ip_port: format!("{}:{}", dst, dst_port),
                                                });
                                                info!("sending events to the MPSC channel");
                                                let _ = tx.send(Ok(evt)).await;
                                            }
                                            Err(_) => {
                                                info!(
                                                    "Event Id: {} Protocol: Unknown ({})",
                                                    event_id, pl.proto
                                                );
                                            }
                                        };
                                    } else {
                                        warn!(
                                            "Received packet data too small: {} bytes",
                                            data.len()
                                        );
                                    }
                                }
                            } else if events.read == 0 {
                                info!("[Agent/API] 0 Events found");
                                let mut evt = Vec::new();
                                evt.push(ConnectionEvent {
                                    event_id: "0".to_string(),
                                    src_ip_port: "0:0".to_string(),
                                    dst_ip_port: "0:0".to_string(),
                                });
                                let _ = tx.send(Ok(evt)).await;
                            }
                        }
                        Err(e) => {
                            eprintln!("Errore nella lettura eventi: {}", e);
                            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                        }
                    }
                }
                // small delay to avoid cpu congestion
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        });

        api
    }
}

#[async_trait]
impl Agent for AgentApi {
    // * read the incoming active_connections requests and returns a response with the
    // * active connections. The data are transformed and sent to the api with a mpsc channel
    async fn active_connections(
        &self,
        request: Request<RequestActiveConnections>,
    ) -> Result<Response<ActiveConnectionResponse>, Status> {
        //read request
        let req = request.into_inner();

        //create the hashmap to process events from the mpsc channel queue
        let mut aggregated_events: Vec<ConnectionEvent> = Vec::new();

        //aggregate events
        while let Ok(evt) = self.event_rx.lock().unwrap().try_recv() {
            if let Ok(vec) = evt {
                aggregated_events.extend(vec);
            }
        }

        //log response for debugging
        info!(
            "DEBUGGING RESPONSE FROM ACTIVE CONNECTION REQUEST: {:?}",
            aggregated_events
        );

        //return response
        Ok(Response::new(ActiveConnectionResponse {
            status: "success".to_string(),
            events: aggregated_events,
        }))
    }
}
