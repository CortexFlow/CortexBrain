use anyhow::Context;
use anyhow::anyhow;
use aya::maps::perf::PerfEventArrayBuffer;
use chrono::Local;
use cortexbrain_common::buffer_type::IpProtocols;
use cortexbrain_common::buffer_type::PacketLossMetrics;
use cortexbrain_common::buffer_type::PacketLog;
use cortexbrain_common::buffer_type::TimeStampMetrics;
use cortexbrain_common::formatters::{format_ipv4, format_ipv6};
use cortexbrain_common::map_handlers::load_perf_event_array_from_mapdata;
use std::str::FromStr;
use std::sync::Mutex;
use tonic::{Request, Response, Status};
use tracing::info;

use aya::maps::MapData;
use std::result::Result::Ok;
use tonic::async_trait;

use aya::maps::HashMap as ayaHashMap;
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::task;

use crate::agent::{
    ConnectionEvent, DroppedPacketMetric, DroppedPacketsResponse, LatencyMetric,
    LatencyMetricsResponse, VethEvent,
};

use cortexbrain_common::buffer_type::VethLog;

// *  contains agent api configuration
use crate::agent::{
    ActiveConnectionResponse, AddIpToBlocklistRequest, BlocklistResponse, RequestActiveConnections,
    RmIpFromBlocklistRequest, RmIpFromBlocklistResponse, VethHashMapResponse, VethResponse,
    agent_server::Agent,
};
use crate::constants::PIN_BLOCKLIST_MAP_PATH;

use crate::helpers::comm_to_string;
use aya::maps::Map;
use std::net::Ipv4Addr;
use tracing::warn;

use cortexbrain_common::buffer_type::BufferSize;
use cortexbrain_common::buffer_type::fill_buffers;

pub struct AgentApi {
    //* event_rx is an istance of a mpsc receiver.
    //* is used to receive the data from the transmitter (tx)
    active_connection_event_rx: Mutex<mpsc::Receiver<Result<Vec<ConnectionEvent>, Status>>>,
    pub(crate) active_connection_event_tx: mpsc::Sender<Result<Vec<ConnectionEvent>, Status>>,
    latency_metrics_rx: Mutex<mpsc::Receiver<Result<Vec<LatencyMetric>, Status>>>,
    pub(crate) latency_metrics_tx: mpsc::Sender<Result<Vec<LatencyMetric>, Status>>,
    dropped_packet_metrics_rx: Mutex<mpsc::Receiver<Result<Vec<DroppedPacketMetric>, Status>>>,
    pub(crate) dropped_packet_metrics_tx: mpsc::Sender<Result<Vec<DroppedPacketMetric>, Status>>,
    tracked_veth_rx: Mutex<mpsc::Receiver<Result<Vec<VethEvent>, Status>>>,
    pub(crate) tracked_veth_tx: mpsc::Sender<Result<Vec<VethEvent>, Status>>,
}

//initialize a default trait for AgentApi. Loads a name and a bpf istance.
//this trait is essential for init the Agent.
impl Default for AgentApi {
    fn default() -> Self {
        //
        // init MapData from the kernel space
        //

        // TODO: in the future will be better to not use .unwrap()
        let active_connection_events_array =
            load_perf_event_array_from_mapdata("/sys/fs/bpf/maps/events_map").unwrap();
        let network_metrics_events_array =
            load_perf_event_array_from_mapdata("/sys/fs/bpf/trace_maps/net_metrics").unwrap();
        let time_stamp_events_array =
            load_perf_event_array_from_mapdata("/sys/fs/bpf/trace_maps/time_stamp_events").unwrap();
        let tracked_veth_events_array =
            load_perf_event_array_from_mapdata("/sys/fs/bpf/maps/veth_identity_map").unwrap();

        //
        // init a mpsc channels with TX (transmission) and RX(Receiver) components
        //

        let (conn_tx, conn_rx) = mpsc::channel(1024);
        let (lat_tx, lat_rx) = mpsc::channel(2048);
        let (drop_tx, drop_rx) = mpsc::channel(2048);
        let (veth_tx, tracked_veth_rx) = mpsc::channel(1024);

        // init the API to send the events from the agent to the CLI
        let api = AgentApi {
            active_connection_event_rx: conn_rx.into(),
            active_connection_event_tx: conn_tx.clone(),
            latency_metrics_rx: Mutex::new(lat_rx),
            latency_metrics_tx: lat_tx.clone(),
            dropped_packet_metrics_rx: Mutex::new(drop_rx),
            dropped_packet_metrics_tx: drop_tx.clone(),
            tracked_veth_rx: Mutex::new(tracked_veth_rx),
            tracked_veth_tx: veth_tx.clone(),
        };

        // init map manager
        //let map_manager = map_manager(maps)?

        // init the buffers
        let mut net_events_buffers = BufferSize::TcpEvents.set_buffer();
        let mut net_metrics_buffers = BufferSize::NetworkMetricsEvents.set_buffer();
        let mut ts_metrics_buffers = BufferSize::TimeMetricsEvents.set_buffer();
        let mut veth_metrics_buffers = BufferSize::VethEvents.set_buffer();

        // init the Vec of Buffers

        let mut net_events_vec_buffer = Vec::<PerfEventArrayBuffer<MapData>>::new();
        let mut net_metrics_vec_buffer = Vec::<PerfEventArrayBuffer<MapData>>::new();
        let mut ts_events_vec_buffer = Vec::<PerfEventArrayBuffer<MapData>>::new();
        let mut veth_events_vec_buffer = Vec::<PerfEventArrayBuffer<MapData>>::new();

        // fill the Vec of Buffers

        net_events_vec_buffer = fill_buffers(net_events_vec_buffer, active_connection_events_array);
        net_metrics_vec_buffer = fill_buffers(net_metrics_vec_buffer, network_metrics_events_array);

        ts_events_vec_buffer = fill_buffers(ts_events_vec_buffer, time_stamp_events_array);

        veth_events_vec_buffer = fill_buffers(veth_events_vec_buffer, tracked_veth_events_array);

        // For network metrics

        //spawn an event readers
        task::spawn(async move {
            info!("Starting event listener");
            //send the data through a mpsc channel
            loop {
                for buf in net_events_vec_buffer.iter_mut() {
                    match buf.read_events(&mut net_events_buffers) {
                        Ok(events) => {
                            //read the events, this function is similar to the one used in identity/helpers.rs/display_events
                            if events.read > 0 {
                                for i in 0..events.read {
                                    let data = &net_events_buffers[i];
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
                                                let _ = conn_tx.send(Ok(evt)).await;
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
                            } else if events.lost > 0 {
                                info!("[Agent/API] Lost {} events", events.lost);
                            }
                        }
                        Err(e) => {
                            eprintln!("Error while reading events: {}", e);
                            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                        }
                    }
                }
                // small delay to avoid cpu congestion
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        });

        task::spawn(async move {
            info!("Starting network metrics listener");

            //send the data through a mpsc channel
            loop {
                for buf in net_metrics_vec_buffer.iter_mut() {
                    match buf.read_events(&mut net_metrics_buffers) {
                        Ok(events) => {
                            //read the events, this function is similar to the one used in identity/helpers.rs/display_events
                            if events.read > 0 {
                                for i in 0..events.read {
                                    let data = &net_metrics_buffers[i];
                                    if data.len() >= std::mem::size_of::<PacketLossMetrics>() {
                                        let nm: PacketLossMetrics =
                                            unsafe { std::ptr::read(data.as_ptr() as *const _) };

                                        let dropped_packet_metrics = DroppedPacketMetric {
                                            tgid: nm.tgid,
                                            process_name: comm_to_string(&nm.comm),
                                            sk_drops: nm.sk_drops,
                                            sk_err: nm.sk_err,
                                            sk_err_soft: nm.sk_err_soft,
                                            sk_backlog_len: nm.sk_backlog_len as u32,
                                            sk_wmem_queued: nm.sk_write_memory_queued,
                                            sk_rcvbuf: nm.sk_receive_buffer_size,
                                            sk_ack_backlog: nm.sk_ack_backlog,
                                            timestamp_us: nm.ts_us,
                                        };

                                        if dropped_packet_metrics.sk_drops > 0 {
                                            let mut evt = Vec::new();
                                            info!(
                                                "Dropped Packet Metric - tgid: {}, process_name: {}, sk_drops: {}, sk_err: {}, sk_err_soft: {}, sk_backlog_len: {}, sk_wmem_queued: {}, sk_rcvbuf: {}, sk_ack_backlog: {}, timestamp_us: {}",
                                                dropped_packet_metrics.tgid,
                                                dropped_packet_metrics.process_name,
                                                dropped_packet_metrics.sk_drops,
                                                dropped_packet_metrics.sk_err,
                                                dropped_packet_metrics.sk_err_soft,
                                                dropped_packet_metrics.sk_backlog_len,
                                                dropped_packet_metrics.sk_wmem_queued,
                                                dropped_packet_metrics.sk_rcvbuf,
                                                dropped_packet_metrics.sk_ack_backlog,
                                                dropped_packet_metrics.timestamp_us
                                            );
                                            evt.push(dropped_packet_metrics.clone());
                                            let _ = drop_tx.send(Ok(evt)).await;
                                        }
                                    } else {
                                        warn!(
                                            "Received network metrics data too small: {} bytes",
                                            data.len()
                                        );
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Errore nella lettura network metrics eventi: {}", e);
                            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                        }
                    }
                }
                // small delay to avoid cpu congestion
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        });

        task::spawn(async move {
            info!("Starting time stamp events listener");

            //send the data through a mpsc channel
            loop {
                for buf in ts_events_vec_buffer.iter_mut() {
                    match buf.read_events(&mut ts_metrics_buffers) {
                        Ok(events) => {
                            //read the events, this function is similar to the one used in identity/helpers.rs/display_events
                            if events.read > 0 {
                                for i in 0..events.read {
                                    let data = &ts_metrics_buffers[i];
                                    if data.len() >= std::mem::size_of::<TimeStampMetrics>() {
                                        let tsm: TimeStampMetrics =
                                            unsafe { std::ptr::read(data.as_ptr() as *const _) };
                                        let saddr_v6 = tsm.saddr_v6;
                                        let daddr_v6 = tsm.daddr_v6;
                                        let latency_metric = LatencyMetric {
                                            delta_us: tsm.delta_us,
                                            timestamp_us: tsm.ts_us,
                                            tgid: tsm.tgid,
                                            process_name: comm_to_string(&tsm.comm),
                                            local_port: tsm.lport as u32,
                                            remote_port: tsm.dport_be as u32,
                                            address_family: tsm.af as u32,
                                            src_address_v4: format_ipv4(tsm.saddr_v4),
                                            dst_address_v4: format_ipv4(tsm.daddr_v4),
                                            src_address_v6: format_ipv6(&saddr_v6),
                                            dst_address_v6: format_ipv6(&daddr_v6),
                                        };
                                        info!(
                                            "Latency Metric - tgid: {}, process_name: {}, delta_us: {}, timestamp_us: {}, local_port: {}, remote_port: {}, address_family: {}, src_address_v4: {}, dst_address_v4: {}, src_address_v6: {}, dst_address_v6: {}",
                                            latency_metric.tgid,
                                            latency_metric.process_name,
                                            latency_metric.delta_us,
                                            latency_metric.timestamp_us,
                                            latency_metric.local_port,
                                            latency_metric.remote_port,
                                            latency_metric.address_family,
                                            latency_metric.src_address_v4,
                                            latency_metric.dst_address_v4,
                                            latency_metric.src_address_v6,
                                            latency_metric.dst_address_v6
                                        );
                                        let mut evt = Vec::new();
                                        evt.push(latency_metric.clone());
                                        let _ = lat_tx.send(Ok(evt)).await;
                                    } else {
                                        warn!(
                                            "Received time stamp metrics data too small: {} bytes",
                                            data.len()
                                        );
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Errore nella lettura time stamp eventi: {}", e);
                            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                        }
                    }
                }
            }
        });

        task::spawn(async move {
            info!("Starting time stamp events listener");

            //send the data through a mpsc channel
            loop {
                for buf in veth_events_vec_buffer.iter_mut() {
                    match buf.read_events(&mut veth_metrics_buffers) {
                        Ok(events) => {
                            //read the events, this function is similar to the one used in identity/helpers.rs/display_events
                            if events.read > 0 {
                                for i in 0..events.read {
                                    info!("Found veth events {}", events.read);
                                    let data = &veth_metrics_buffers[i];
                                    if data.len() >= std::mem::size_of::<VethLog>() {
                                        let veth: VethLog =
                                            unsafe { std::ptr::read(data.as_ptr() as *const _) };
                                        let veth_event = VethEvent {
                                            name: String::from_utf8_lossy(unsafe {
                                                std::slice::from_raw_parts(
                                                    veth.name.as_ptr() as *const u8,
                                                    veth.name.len() * std::mem::size_of::<u32>(),
                                                )
                                            })
                                            .trim_end_matches('\0')
                                            .to_string(),
                                            state: veth.state,
                                            dev_addr: String::from_utf8_lossy(unsafe {
                                                std::slice::from_raw_parts(
                                                    veth.dev_addr.as_ptr() as *const u8,
                                                    veth.dev_addr.len()
                                                        * std::mem::size_of::<u32>(),
                                                )
                                            })
                                            .trim_end_matches('\0')
                                            .to_string(),
                                            event_type: veth.event_type.into(),
                                            netns: veth.netns,
                                            pid: veth.pid,
                                        };
                                        info!(
                                            "Veth Event - name: {}, state: {}, dev_addr: {}, event_type: {}, netns: {}, pid: {}",
                                            veth_event.name,
                                            veth_event.state,
                                            veth_event.dev_addr,
                                            veth_event.event_type,
                                            veth_event.netns,
                                            veth_event.pid
                                        );
                                        let mut evt = Vec::new();
                                        evt.push(veth_event.clone());
                                        let _ = veth_tx.send(Ok(evt)).await;
                                    } else {
                                        warn!(
                                            "Received time stamp metrics data too small: {} bytes",
                                            data.len()
                                        );
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Errore nella lettura time stamp eventi: {}", e);
                            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                        }
                    }
                }
            }
        });

        api
    }
}

//declare the blocklist hashmap structure
//TODO: finish the creation of a blocklist hashmap
#[async_trait]
impl Agent for AgentApi {
    // * read the incoming active_connections requests and returns a response with the
    // * active connections. The data are transformed and sent to the api with a mpsc channel
    async fn active_connections(
        &self,
        request: Request<RequestActiveConnections>,
    ) -> Result<Response<ActiveConnectionResponse>, Status> {
        //read request
        let _req = request.into_inner();

        //create the hashmap to process events from the mpsc channel queue
        let mut aggregated_events: Vec<ConnectionEvent> = Vec::new();

        //aggregate events
        while let Ok(evt) = self.active_connection_event_rx.lock().unwrap().try_recv() {
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

    // * creates and add ip to the blocklist
    async fn add_ip_to_blocklist(
        &self,
        request: Request<AddIpToBlocklistRequest>,
    ) -> Result<Response<BlocklistResponse>, Status> {
        //read request
        let req = request.into_inner();

        //open blocklist map
        let mapdata = MapData::from_pin("/sys/fs/bpf/maps/blocklist_map")
            .expect("cannot open blocklist_map Mapdata");
        let blocklist_mapdata = Map::HashMap(mapdata); //load mapdata
        let mut blocklist_map: ayaHashMap<MapData, [u8; 4], [u8; 4]> =
            ayaHashMap::try_from(blocklist_mapdata).unwrap();

        if req.ip.is_none() {
            // log blocklist event
            info!("IP field in request is none");
            info!("CURRENT BLOCKLIST: {:?}", blocklist_map);
        } else {
            // add ip to the blocklist
            // log blocklist event
            let _datetime = Local::now().to_string();
            let ip = req.ip.unwrap();
            //convert ip from string to [u8;4] type and insert into the bpf map
            let u8_4_ip = Ipv4Addr::from_str(&ip).unwrap().octets();
            //TODO: convert datetime in a kernel compatible format
            blocklist_map
                .insert(u8_4_ip, u8_4_ip, 0)
                .map_err(|e| anyhow!("Cannot insert address in the blocklist. Reason: {}", e))
                .unwrap();
            info!("CURRENT BLOCKLIST: {:?}", blocklist_map);
        }
        let _path = std::env::var(PIN_BLOCKLIST_MAP_PATH)
            .context("Blocklist map path not found!")
            .unwrap();

        //convert the maps with a buffer to match the protobuffer types
        let mut converted_blocklist_map: HashMap<String, String> = HashMap::new();
        for item in blocklist_map.iter() {
            let (k, _v) = item.unwrap();
            // convert keys and values from [u8;4] to String
            let key = Ipv4Addr::from(k).to_string();
            let value = Ipv4Addr::from(k).to_string();
            converted_blocklist_map.insert(key, value);
        }

        //save ip into the blocklist
        Ok(Response::new(BlocklistResponse {
            status: "success".to_string(),
            events: converted_blocklist_map,
        }))
    }

    async fn check_blocklist(
        &self,
        _request: Request<()>,
    ) -> Result<Response<BlocklistResponse>, Status> {
        info!("Returning blocklist hashmap");
        //open blocklist map
        let mapdata = MapData::from_pin("/sys/fs/bpf/maps/blocklist_map")
            .expect("cannot open blocklist_map Mapdata");
        let blocklist_mapdata = Map::HashMap(mapdata); //load mapdata

        let blocklist_map: ayaHashMap<MapData, [u8; 4], [u8; 4]> =
            ayaHashMap::try_from(blocklist_mapdata).unwrap();

        //convert the maps with a buffer to match the protobuffer types

        let mut converted_blocklist_map: HashMap<String, String> = HashMap::new();
        for item in blocklist_map.iter() {
            let (k, _v) = item.unwrap();
            // convert keys and values from [u8;4] to String
            let key = Ipv4Addr::from(k).to_string();
            let value = Ipv4Addr::from(k).to_string();
            converted_blocklist_map.insert(key, value);
        }
        Ok(Response::new(BlocklistResponse {
            status: "success".to_string(),
            events: converted_blocklist_map,
        }))
    }
    async fn rm_ip_from_blocklist(
        &self,
        request: Request<RmIpFromBlocklistRequest>,
    ) -> Result<Response<RmIpFromBlocklistResponse>, Status> {
        //read request
        let req = request.into_inner();
        info!("Removing ip from blocklist map");
        //open blocklist map
        let mapdata = MapData::from_pin("/sys/fs/bpf/maps/blocklist_map")
            .expect("cannot open blocklist_map Mapdata");
        let blocklist_mapdata = Map::HashMap(mapdata); //load mapdata
        let mut blocklist_map: ayaHashMap<MapData, [u8; 4], [u8; 4]> =
            ayaHashMap::try_from(blocklist_mapdata).unwrap();
        //remove the address
        let ip_to_remove = req.ip;
        let u8_4_ip_to_remove = Ipv4Addr::from_str(&ip_to_remove).unwrap().octets();
        let _ = blocklist_map.remove(&u8_4_ip_to_remove);

        //convert the maps with a buffer to match the protobuffer types
        let mut converted_blocklist_map: HashMap<String, String> = HashMap::new();
        for item in blocklist_map.iter() {
            let (k, v) = item.unwrap();
            // convert keys and values from [u8;4] to String
            let key = Ipv4Addr::from(k).to_string();
            let value = Ipv4Addr::from(k).to_string();
            converted_blocklist_map.insert(key, value);
        }

        Ok(Response::new(RmIpFromBlocklistResponse {
            status: "Ip removed from blocklist".to_string(),
            events: converted_blocklist_map,
        }))
    }

    async fn get_latency_metrics(
        &self,
        request: Request<()>,
    ) -> Result<Response<LatencyMetricsResponse>, Status> {
        // Extract the request parameters
        let _req = request.into_inner();
        info!("Getting latency metrics");

        // Here you would typically query your data source for the latency metrics
        // For demonstration purposes, we'll return a dummy response

        let mut aggregated_latency_metrics_events: Vec<LatencyMetric> = Vec::new();

        while let Ok(evt) = self.latency_metrics_rx.lock().unwrap().try_recv() {
            if let Ok(vec) = evt {
                aggregated_latency_metrics_events.extend(vec);
            }
        }

        let total_count = aggregated_latency_metrics_events.len() as u32;

        let (average_latency_us, min_latency_us, max_latency_us) =
            if !aggregated_latency_metrics_events.is_empty() {
                let sum: u64 = aggregated_latency_metrics_events
                    .iter()
                    .map(|m| m.delta_us)
                    .sum();
                let avg = sum as f64 / aggregated_latency_metrics_events.len() as f64;

                let min = aggregated_latency_metrics_events
                    .iter()
                    .map(|m| m.delta_us)
                    .min()
                    .unwrap_or(0) as f64;

                let max = aggregated_latency_metrics_events
                    .iter()
                    .map(|m| m.delta_us)
                    .max()
                    .unwrap_or(0) as f64;

                (avg, min, max)
            } else {
                (0.0, 0.0, 0.0)
            };

        info!(
            "Latency metrics - total_count: {}, average: {:.2}us, min: {:.2}us, max: {:.2}us",
            total_count, average_latency_us, min_latency_us, max_latency_us
        );

        let response = LatencyMetricsResponse {
            status: "success".to_string(),
            metrics: aggregated_latency_metrics_events,
            total_count,
            average_latency_us,
            max_latency_us,
            min_latency_us,
        };

        Ok(Response::new(response))
    }

    async fn get_dropped_packets_metrics(
        &self,
        request: Request<()>,
    ) -> Result<Response<DroppedPacketsResponse>, Status> {
        // Extract the request parameters
        let _req = request.into_inner();
        info!("Getting dropped packets metrics");

        let mut aggregated_dropped_packet_metrics: Vec<DroppedPacketMetric> = Vec::new();
        let mut total_drops = 0u32;

        // Collect all metrics from channel
        while let Ok(evt) = self.dropped_packet_metrics_rx.lock().unwrap().try_recv() {
            if let Ok(vec) = evt {
                for metric in vec {
                    total_drops += metric.sk_drops as u32;
                    aggregated_dropped_packet_metrics.push(metric);
                }
            }
        }

        info!(
            "Dropped packets metrics - total_metrics: {}, total_drops: {}",
            aggregated_dropped_packet_metrics.len(),
            total_drops
        );

        let response = DroppedPacketsResponse {
            status: "success".to_string(),
            metrics: aggregated_dropped_packet_metrics,
            total_drops,
        };

        Ok(Response::new(response))
    }

    async fn get_tracked_veth(
        &self,
        request: Request<()>,
    ) -> Result<Response<VethResponse>, Status> {
        let _req = request.into_inner();
        info!("Getting tracked veth metrics");
        let mut tracked_veth = Vec::<VethEvent>::new();
        let mut tot_veth = 0 as i32;

        while let Ok(evt) = self.tracked_veth_rx.lock().unwrap().try_recv() {
            if let Ok(vec) = evt {
                tracked_veth.extend(vec);
            }
        }
        tot_veth = tracked_veth.len() as i32;

        info!("Total tracked veth events: {}", tot_veth);
        info!("Tracked veth: {:?}", &tracked_veth);

        let veth_names: Vec<String> = tracked_veth.iter().map(|v| v.name.clone()).collect();

        let response = VethResponse {
            status: "success".to_string(),
            veth_names,
            tot_monitored_veth: tot_veth,
        };

        Ok(Response::new(response))
    }

    async fn get_tracked_veth_from_hash_map(
        &self,
        _request: Request<()>,
    ) -> Result<Response<VethHashMapResponse>, Status> {
        info!("Returning veth hashmap");
        //open blocklist map
        let mapdata = MapData::from_pin("/sys/fs/bpf/maps/tracked_veth")
            .expect("cannot open tracked_veth Mapdata");
        let tracked_veth_mapdata = Map::HashMap(mapdata); //load mapdata

        let tracked_veth_map: ayaHashMap<MapData, [u8; 16], [u8; 8]> =
            ayaHashMap::try_from(tracked_veth_mapdata).unwrap();

        //convert the maps with a buffer to match the protobuffer types

        let mut converted_tracked_veth_map: HashMap<String, String> = HashMap::new();
        for item in tracked_veth_map.iter() {
            let (k, v) = item.unwrap();
            // convert keys and values from [u8;4] to String
            let key = String::from_utf8(k.to_vec()).unwrap();
            let value = String::from_utf8(v.to_vec()).unwrap();
            converted_tracked_veth_map.insert(key, value);
        }
        Ok(Response::new(VethHashMapResponse {
            status: "success".to_string(),
            veths: converted_tracked_veth_map,
        }))
    }
}
