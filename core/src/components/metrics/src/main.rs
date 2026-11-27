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
use crate::helpers::{ProbeAttachments, event_listener, resolve_libssl_path};
use crate::otel_init::{init_opentelemetry, shutdown_opentelemetry};

use cortexbrain_common::{
    constants,
    logger::otlp_logger_init,
    map_handlers::{init_bpf_maps, map_pinner},
    program_handlers::{load_program, load_tracepoint_program, load_uprobe_program},
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
    let cpu_frequency = bpf.clone();
    let cpu_idle_bpf = bpf.clone();
    let mem_alloc_bpf = bpf.clone();
    let sched_stat_wait_bpf = bpf.clone();
    let sched_stat_runtime_bpf = bpf.clone();
    let ssl_read_bpf = bpf.clone();
    let ssl_read_ret_bpf = bpf.clone();
    let ssl_write_bpf = bpf.clone();
    let ssl_write_ret_bpf = bpf.clone();

    info!("Running Ebpf logger");
    info!("loading programs");

    let bpf_map_save_path =
        env::var(constants::PIN_MAP_PATH).context("PIN_MAP_PATH environment variable required")?;

    let map_data = vec![
        "time_stamp_events".to_string(),
        "net_metrics".to_string(),
        "cpu_frequency".to_string(),
        "cpu_idle".to_string(),
        "mem_alloc".to_string(),
        "sched_stat_wait".to_string(),
        "sched_stat_runtime".to_string(),
        "ssl_events".to_string(),
    ];

    match init_bpf_maps(bpf.clone(), map_data) {
        Ok(bpf_maps) => {
            info!("BPF maps loaded successfully");
            let pin_path = std::path::PathBuf::from(&bpf_map_save_path);
            info!("About to call map_pinner with path: {:?}", pin_path);

            match map_pinner(bpf_maps, &pin_path) {
                Ok(maps) => {
                    info!("BPF maps pinned successfully to {}", bpf_map_save_path);

                    // Every probe is optional: a kernel without CONFIG_SCHEDSTATS has no
                    // sched:sched_stat_wait tracepoint, a static function can be inlined
                    // away, kprobes can be disabled. One missing hook must not take the
                    // other metrics down with it, so failures are logged and skipped.
                    let mut probes = ProbeAttachments::default();
                    {
                        probes.record(
                            "kprobe tcp_identify_packet_loss",
                            load_program(
                                bpf.clone(),
                                "packet_loss_tracer",
                                "tcp_identify_packet_loss",
                            ),
                        );
                        probes.record(
                            "kprobe tcp_v4_connect",
                            load_program(tcp_bpf, "tcp_v4_connect", "tcp_v4_connect"),
                        );
                        probes.record(
                            "kprobe tcp_v6_connect",
                            load_program(tcp_v6_bpf, "tcp_v6_connect", "tcp_v6_connect"),
                        );
                        probes.record(
                            "kprobe tcp_rcv_state_process",
                            load_program(
                                tcp_rev_bpf,
                                "tcp_latency_monitor",
                                "tcp_rcv_state_process",
                            ),
                        );
                        probes.record(
                            "tracepoint percpu:percpu_alloc_percpu",
                            load_tracepoint_program(
                                cpu_frequency,
                                "trace_cpu_frequency",
                                "percpu",
                                "percpu_alloc_percpu",
                            ),
                        );
                        probes.record(
                            "tracepoint power:cpu_idle",
                            load_tracepoint_program(
                                cpu_idle_bpf,
                                "trace_cpu_idle",
                                "power",
                                "cpu_idle",
                            ),
                        );
                        probes.record(
                            "tracepoint syscalls:sys_enter_mmap",
                            load_tracepoint_program(
                                mem_alloc_bpf,
                                "trace_enter_mmap",
                                "syscalls",
                                "sys_enter_mmap",
                            ),
                        );
                        probes.record(
                            "tracepoint sched:sched_stat_wait",
                            load_tracepoint_program(
                                sched_stat_wait_bpf,
                                "trace_sched_stat_wait",
                                "sched",
                                "sched_stat_wait",
                            ),
                        );
                        probes.record(
                            "tracepoint sched:sched_stat_runtime",
                            load_tracepoint_program(
                                sched_stat_runtime_bpf,
                                "trace_sched_stat_runtime",
                                "sched",
                                "sched_stat_runtime",
                            ),
                        );
                        match resolve_libssl_path()? {
                            Some(ssl_lib_path) => {
                                probes.record(
                                    "uprobe SSL_read",
                                    load_uprobe_program(
                                        ssl_read_bpf,
                                        "ssl_read",
                                        "SSL_read",
                                        &ssl_lib_path,
                                        None,
                                    ),
                                );
                                probes.record(
                                    "uretprobe SSL_read",
                                    load_uprobe_program(
                                        ssl_read_ret_bpf,
                                        "ssl_read_ret",
                                        "SSL_read",
                                        &ssl_lib_path,
                                        None,
                                    ),
                                );
                                probes.record(
                                    "uprobe SSL_write",
                                    load_uprobe_program(
                                        ssl_write_bpf,
                                        "ssl_write",
                                        "SSL_write",
                                        &ssl_lib_path,
                                        None,
                                    ),
                                );
                                probes.record(
                                    "uretprobe SSL_write",
                                    load_uprobe_program(
                                        ssl_write_ret_bpf,
                                        "ssl_write_ret",
                                        "SSL_write",
                                        &ssl_lib_path,
                                        None,
                                    ),
                                );
                            }
                            None => {
                                info!("OpenSSL library not found; skipping SSL uprobe tracing");
                            }
                        }
                    }

                    if let Err(e) = probes.ensure_any_attached() {
                        error!("{:?}", e);
                        shutdown_opentelemetry();
                        return Err(e);
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
