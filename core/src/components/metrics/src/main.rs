use aya::{
    Ebpf
};

use std::{
    env, fs,
    path::Path,
    sync::{
        Arc, Mutex,
    },
};

use anyhow::{Context, Ok};
use tracing::{error, info};
use cortexbrain_common::{constants, logger};

mod helpers;
use crate::{helpers::event_listener, maps_handlers::map_pinner, program_handlers::load_and_attach_tcp_programs};

mod maps_handlers;
use crate::maps_handlers::init_ebpf_maps;

mod program_handlers;
use crate::program_handlers::load_program;

mod structs;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    //init tracing subscriber
    logger::init_default_logger();

    info!("Starting metrics service...");
    info!("fetching data");

    let bpf_path = env::var(constants::BPF_PATH).context("BPF_PATH environment variable required")?;
    let data = fs::read(Path::new(&bpf_path)).context("Failed to load file from path")?;
    let bpf = Arc::new(Mutex::new(Ebpf::load(&data)?));
    let tcp_bpf = bpf.clone();
    let tcp_rev_bpf = bpf.clone();

    info!("Running Ebpf logger");
    info!("loading programs");
    let bpf_map_save_path =
        std::env::var(constants::PIN_MAP_PATH).context("PIN_MAP_PATH environment variable required")?;

    match init_ebpf_maps(bpf.clone()) {
        std::result::Result::Ok(maps) => {
            info!("BPF maps loaded successfully");
            match map_pinner(&maps, &bpf_map_save_path.clone().into()).await {
                std::result::Result::Ok(_) => {
                    info!("BPF maps pinned successfully to {}", bpf_map_save_path);

                    {
                        load_program(bpf.clone(), "metrics_tracer", "tcp_identify_packet_loss")
                            .context("An error occured during the execution of load_program function")?;
                    }

                    {
                        load_and_attach_tcp_programs(tcp_bpf.clone())
                            .context("An error occured during the execution of load_and_attach_tcp_programs function")?;
                    }

                    {
                        load_program(tcp_rev_bpf.clone(), "tcp_rcv_state_process", "tcp_rcv_state_process")
                            .context("An error occured during the execution of load_program function")?;
                    }

                    event_listener(maps).await?;
                }
                Err(e) => {
                    error!("Error pinning BPF maps: {:?}", e);
                    return Err(e);
                }
            }
        }
        Err(e) => {
            error!("Error initializing BPF maps: {:?}", e);
            return Err(e);
        }
    }

    Ok(())
}