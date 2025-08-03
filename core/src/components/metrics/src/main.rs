use aya::{
    Ebpf,
    maps::{
         MapData,
        perf::{PerfEventArray, PerfEventArrayBuffer},
    },
    programs::{KProbe},
    util::online_cpus,
};

use bytes::BytesMut;
use std::{
    convert::TryInto,
    env, fs,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
    },
};

use anyhow::{Context, Ok};
use tokio::{signal};
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, fmt::format::FmtSpan};

const BPF_PATH: &str = "BPF_PATH"; //BPF env path

mod structs;
use crate::structs::NetworkMetrics;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    //init tracing subscriber
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_level(true)
        .with_span_events(FmtSpan::NONE)
        .with_file(false)
        .pretty()
        .with_env_filter(EnvFilter::new("info"))
        .with_line_number(false)
        .init();

    info!("Starting metrics service...");
    info!("fetching data");

    let bpf_path = env::var(BPF_PATH).context("BPF_PATH environment variable required")?;
    let data = fs::read(Path::new(&bpf_path)).context("Failed to load file from path")?;
    let mut bpf = Ebpf::load(&data)?;
    //init bpf logger
    info!("Running Ebpf logger");
    info!("loading programs");
    let net_metrics_map = bpf
        .take_map("net_metrics")
        .ok_or_else(|| anyhow::anyhow!("net_metrics map not found"))?;

    let program: &mut KProbe = bpf
        .program_mut("metrics_tracer")
        .ok_or_else(|| anyhow::anyhow!("program 'metrics_tracer' not found"))?
        .try_into()
        .context("Failed to init Kprobe program")?;

    program
        .load()
        .context("Failed to load metrics_tracer program")?;

    match program.attach("tcp_identify_packet_loss", 0) {
        std::result::Result::Ok(_) => {
            info!("program attached successfully to the tcp_identify_packet_loss kprobe ")
        }
        Err(e) => error!(
            "An error occured while attaching the program to the tcp_identify_packet_loss kprobe. {:?} ",
            e
        ),
    }
    let mut net_perf_buffer: Vec<PerfEventArrayBuffer<MapData>> = Vec::new();
    let mut net_perf_array: PerfEventArray<MapData> = PerfEventArray::try_from(net_metrics_map)?;

    for cpu_id in online_cpus().map_err(|e| anyhow::anyhow!("Error {:?}", e))? {
        let buf: PerfEventArrayBuffer<MapData> = net_perf_array.open(cpu_id, None)?;
        net_perf_buffer.push(buf);
    }
    let running = AtomicBool::new(true);

    let buffers = vec![BytesMut::with_capacity(1024); 10];

    
    tokio::spawn(async move{
        display_metrics_map(net_perf_buffer, running, buffers).await;
    });


    signal::ctrl_c().await?;
    Ok(())
}

pub async fn display_metrics_map(
    mut perf_buffers: Vec<PerfEventArrayBuffer<MapData>>,
    running: AtomicBool,
    mut buffers: Vec<BytesMut>,
) {
    while running.load(Ordering::SeqCst) {
        for buf in perf_buffers.iter_mut() {
            match buf.read_events(&mut buffers) {
                std::result::Result::Ok(events) => {
                    for i in 0..events.read {
                        let data = &buffers[i];
                        if data.len() >= std::mem::size_of::<NetworkMetrics>() {
                            let net_metrics: NetworkMetrics =
                                unsafe { std::ptr::read_unaligned(data.as_ptr() as *const _) };
                            let sk_drop_count = net_metrics.sk_drops;
                            let sk_err = net_metrics.sk_err;
                            let sk_err_soft = net_metrics.sk_err_soft;
                            let sk_backlog_len = net_metrics.sk_backlog_len;
                            let sk_wmem_queued = net_metrics.sk_wmem_queued;
                            let sk_ack_backlog = net_metrics.sk_ack_backlog;
                            let sk_rcvbuf = net_metrics.sk_rcvbuf;
                            info!(
                                "sk_drops: {}, sk_err: {}, sk_err_soft: {}, sk_backlog_len: {}, sk_wmem_queued: {}, sk_ack_backlog: {}, sk_rcvbuf: {}",
                                sk_drop_count, sk_err, sk_err_soft, sk_backlog_len, sk_wmem_queued, sk_ack_backlog, sk_rcvbuf
                            );
                        }
                    }
                }
                Err(e) => {
                    error!("Error reading events: {:?}", e);
                }
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}
