use anyhow::{Context, Ok};
use aya::Ebpf;
use cortexbrain_common::{constants, logger};
use std::{
    env, fs,
    path::Path,
    sync::{Arc, Mutex},
};
use tracing::{error, info};

mod helpers;
use crate::helpers::event_listener;

use cortexbrain_common::map_handlers::{init_bpf_maps, map_pinner};
use cortexbrain_common::program_handlers::load_program;

mod structs;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    //init tracing subscriber
    logger::init_default_logger();

    info!("Starting metrics service...");
    info!("fetching data");

    let bpf_path =
        env::var(constants::BPF_PATH).context("BPF_PATH environment variable required")?;
    let data = fs::read(Path::new(&bpf_path)).context("Failed to load file from path")?;
    let bpf = Arc::new(Mutex::new(Ebpf::load(&data)?));
    let tcp_bpf = bpf.clone();
    let tcp_rev_bpf = bpf.clone();
    let tcp_v6_bpf = bpf.clone();

    info!("Running Ebpf logger");
    info!("loading programs");
    let bpf_map_save_path = std::env::var(constants::PIN_MAP_PATH)
        .context("PIN_MAP_PATH environment variable required")?;

    let map_data = vec!["time_stamp_events".to_string(), "net_metrics".to_string()];

    match init_bpf_maps(bpf.clone(), map_data) {
        std::result::Result::Ok(bpf_maps) => {
            info!("BPF maps loaded successfully");
            let pin_path = std::path::PathBuf::from(&bpf_map_save_path);
            info!("About to call map_pinner with path: {:?}", pin_path);
            match map_pinner(bpf_maps, &pin_path) {
                std::result::Result::Ok(maps) => {
                    info!("BPF maps pinned successfully to {}", bpf_map_save_path);

                    {
                        load_program(bpf.clone(), "metrics_tracer", "tcp_identify_packet_loss")
                            .context(
                                "An error occured during the execution of load_program function",
                            )?;

                        load_program(tcp_bpf,"tcp_v4_connect","tcp_v4_connect")
                        .context("An error occured during the execution of load_and_attach_tcp_programs function")?;
                        load_program(tcp_v6_bpf,"tcp_v6_connect","tcp_v6_connect")
                        .context("An error occured during the execution of load_and_attach_tcp_programs function")?;

                        load_program(
                            tcp_rev_bpf,
                            "tcp_rcv_state_process",
                            "tcp_rcv_state_process",
                        )
                        .context(
                            "An error occured during the execution of load_program function",
                        )?;
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
