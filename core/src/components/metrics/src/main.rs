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
        atomic::{AtomicBool},
    },
};

use anyhow::{Context, Ok};
use tokio::{signal};
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, fmt::format::FmtSpan};

const BPF_PATH: &str = "BPF_PATH"; //BPF env path

mod helpers;
use crate::helpers::display_metrics_map;

mod structs;

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
