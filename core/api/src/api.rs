#![allow(warnings)]
use anyhow::Context;
use prost::bytes::BytesMut;
use std::{
    collections::HashMap,
    fs, string,
    sync::{atomic::AtomicBool, Arc, Mutex},
};
use tonic::{Request, Response, Status};

use aya::{
    maps::{perf::PerfEventArrayBuffer, Map, MapData, PerfEventArray},
    util::online_cpus,
    Bpf,
};
use identity::helpers::display_events;
use identity::map_handlers::init_bpf_maps;
use std::path::Path;
use std::result::Result::Ok;
use tonic::async_trait;

// *  contains agent api configuration
use crate::agent::{agent_server::Agent, ActiveConnectionResponse, RequestActiveConnections};

#[derive(Debug)]
pub struct AgentApi {
    name: String,
    bpf: Arc<Mutex<Bpf>>,
}

const BPF_PATH: &str = "BPF_PATH";

//initialize a default trait for AgentApi. Loads a name and a bpf istance.
//this trait is essential for init the Agent.
impl Default for AgentApi {
    fn default() -> Self {
        let bpf_path = std::env::var(BPF_PATH).context("BPF_PATH variable not found").unwrap();
        let data = fs::read(Path::new(&bpf_path)).context("Cannot load data from path").unwrap();

        AgentApi {
            name: "CortexFlow-Agent".to_string(),
            bpf: Arc::new(Mutex::new(Bpf::load(&data).unwrap())),
        }
    }
}
//TODO: implement a trait that inizialize init_bpf_maps function

#[async_trait]
impl Agent for AgentApi {
    async fn active_connections(
        &self,
        request: Request<RequestActiveConnections>,
    ) -> Result<Response<ActiveConnectionResponse>, Status> {
        //read request
        let req = request.into_inner();

        //initialize maps:
        let bpf_maps = init_bpf_maps(Arc::clone(&self.bpf)).unwrap();

        let mut net_events_buffer = Vec::new();

        //maps are enumerated 0: events map 1: veth events map
        let mut events_array = PerfEventArray::try_from(bpf_maps.0).unwrap();

        //scan the cpus
        for cpu_id in online_cpus()
            .map_err(|e| anyhow::anyhow!("Error {:?}", e))
            .unwrap()
        {
            let net_events_buf: PerfEventArrayBuffer<MapData> =
                events_array.open(cpu_id, None).unwrap();
            net_events_buffer.push(net_events_buf);
        }
        let mut events_buffers = vec![BytesMut::with_capacity(1024); online_cpus().iter().len()];

        let running = Arc::new(AtomicBool::new(true));

        display_events(net_events_buffer, running, events_buffers);

        Ok(Response::new(ActiveConnectionResponse {
            status: "success".to_string(),
        }))
    }
}

//TODO: add server inizialization
