//! CortexBrain metrics service – eBPF-based telemetry with OpenTelemetry export.
//!
//! This binary is the node-level metrics agent for CortexBrain.  It:
//!
//! 1. Initialises an OpenTelemetry metrics pipeline (OTLP / gRPC).
//! 2. Loads a compiled eBPF object and pins its maps to the BPF filesystem.
//! 3. Attaches a set of kernel kprobe programs.
//! 4. Starts asynchronous consumers that read per-CPU perf buffers and
//!    emit OpenTelemetry instruments for every event.
//! 5. Blocks until `Ctrl-C` is received, then shuts down cleanly.

use anyhow::Context;
use aya::Ebpf;
use std::{
    env, fs,
    path::Path,
    sync::{Arc, Mutex},
};
use tracing::{error, info};
mod helpers;
mod otel_init;
use crate::helpers::event_listener;
use crate::otel_init::{init_opentelemetry, shutdown_opentelemetry};

use cortexbrain_common::{
    constants,
    logger::otlp_logger_init,
    map_handlers::{init_bpf_maps, map_pinner},
    program_handlers::load_program,
};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let _otlp_log_provider = otlp_logger_init("metrics-service".to_string());

    info!("Starting metrics service...");
    info!("fetching data");

    let meter =
        init_opentelemetry().context("Failed to initialise OpenTelemetry metrics pipeline")?;

    let bpf_path =
        env::var(constants::BPF_PATH).context("BPF_PATH environment variable required")?;
    let data = fs::read(Path::new(&bpf_path)).context("Failed to load file from path")?;
    let bpf = Arc::new(Mutex::new(Ebpf::load(&data)?));
    let tcp_bpf = bpf.clone();
    let tcp_rev_bpf = bpf.clone();
    let tcp_v6_bpf = bpf.clone();

    info!("Running Ebpf logger");
    info!("loading programs");

    let bpf_map_save_path =
        env::var(constants::PIN_MAP_PATH).context("PIN_MAP_PATH environment variable required")?;

    let map_data = vec!["time_stamp_events".to_string(), "net_metrics".to_string()];

    match init_bpf_maps(bpf.clone(), map_data) {
        Ok(bpf_maps) => {
            info!("BPF maps loaded successfully");
            let pin_path = std::path::PathBuf::from(&bpf_map_save_path);
            info!("About to call map_pinner with path: {:?}", pin_path);

            match map_pinner(bpf_maps, &pin_path) {
                Ok(maps) => {
                    info!("BPF maps pinned successfully to {}", bpf_map_save_path);

                    {
                        load_program(bpf.clone(), "metrics_tracer", "tcp_identify_packet_loss")
                            .context(
                                "An error occurred during the execution of load_program function",
                            )?;

                        load_program(tcp_bpf, "tcp_v4_connect", "tcp_v4_connect")
                            .context("An error occurred during the execution of load_and_attach_tcp_programs function")?;

                        load_program(tcp_v6_bpf, "tcp_v6_connect", "tcp_v6_connect")
                            .context("An error occurred during the execution of load_and_attach_tcp_programs function")?;

                        load_program(
                            tcp_rev_bpf,
                            "tcp_rcv_state_process",
                            "tcp_rcv_state_process",
                        )
                        .context(
                            "An error occurred during the execution of load_program function",
                        )?;
                    }

                    // Hand off to the async event consumer
                    event_listener(maps, meter).await
                }
                Err(e) => {
                    error!("Error pinning BPF maps: {:?}", e);
                    shutdown_opentelemetry();
                    Err(e)
                }
            }
        }
        Err(e) => {
            error!("Error initializing BPF maps: {:?}", e);
            shutdown_opentelemetry();
            Err(e)
        }
    }
}
