use aya::{
    maps::{
         perf::{PerfEventArray, PerfEventArrayBuffer}, MapData
    }, programs::KProbe, util::online_cpus, Ebpf
};

use bytes::BytesMut;
use std::{
    convert::TryInto,
    env, fs,
    path::Path,
    sync::{
        atomic::AtomicBool, Arc, Mutex,
    },
};

use anyhow::{Context, Ok};
use tokio::{signal};
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, fmt::format::FmtSpan};

const BPF_PATH: &str = "BPF_PATH"; //BPF env path

mod helpers;
use crate::helpers::{display_metrics_map, display_time_stamp_events_map};

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
    let bpf = Arc::new(Mutex::new(Ebpf::load(&data)?));
    let tcp_bpf = bpf.clone();
    let tcp_rev_bpf = bpf.clone();

    info!("Running Ebpf logger");
    info!("loading programs");
    
    // First, handle the main bpf lock block
    let (net_metrics_map, time_stamps_events_map, _metrics_program_loaded) = {
        let mut bpf = bpf.lock().unwrap();
        
        info!("Looking for net_metrics map...");
        let net_metrics_map = bpf
            .take_map("net_metrics")
            .ok_or_else(|| anyhow::anyhow!("net_metrics map not found"))?;
        info!("Found net_metrics map");
        
        info!("Looking for time_stamp_events map...");
        let time_stamps_events_map = bpf
            .take_map("time_stamp_events")
            .ok_or_else(|| anyhow::anyhow!("time_stamp_events map not found"))?;
        info!("Found time_stamp_events map");

        info!("Looking for metrics_tracer program...");
        let program: &mut KProbe = bpf
            .program_mut("metrics_tracer")
            .ok_or_else(|| anyhow::anyhow!("program 'metrics_tracer' not found"))?
            .try_into()
            .context("Failed to init Kprobe program")?;
        info!("Found metrics_tracer program, loading...");
        program
            .load()
            .context("Failed to load metrics_tracer program")?;
        info!("metrics_tracer program loaded successfully");

        (net_metrics_map, time_stamps_events_map, true)
    }; // bpf lock is released here

    // Now handle tcp_connect program
    info!("Looking for tcp_connect program...");
    let _tcp_program_loaded = {
        let mut tcp_bpf = tcp_bpf.lock().unwrap();
        let tcp_program: &mut KProbe = tcp_bpf
            .program_mut("tcp_connect")
            .ok_or_else(|| anyhow::anyhow!("program 'tcp_connect' not found"))?
            .try_into()
            .context("Failed to init Kprobe program")?;
        info!("Found tcp_connect program, loading...");
        tcp_program.load().context("Failed to load tcp_connect program")?;
        info!("tcp_connect program loaded successfully");
        true
    }; // tcp_bpf lock is released here

    // Now handle tcp_rcv_state_process program  
    info!("Looking for tcp_rcv_state_process program...");
    let _tcp_rev_program_loaded = {
        let mut tcp_rev_bpf = tcp_rev_bpf.lock().unwrap();
        let tcp_rev_state_program: &mut KProbe = tcp_rev_bpf
            .program_mut("tcp_rcv_state_process")
            .ok_or_else(|| anyhow::anyhow!("program 'tcp_rcv_state_process' not found"))?
            .try_into()
            .context("Failed to init Kprobe program")?;
        info!("Found tcp_rcv_state_process program, loading...");
        tcp_rev_state_program
            .load()
            .context("Failed to load tcp_rcv_state_process program")?;
        info!("tcp_rcv_state_process program loaded successfully");
        true
    }; // tcp_rev_bpf lock is released here

    info!("Starting program attachments...");
    
    // Now do the attachments
    {
        let mut bpf = bpf.lock().unwrap();
        let program: &mut KProbe = bpf
            .program_mut("metrics_tracer")
            .unwrap()
            .try_into()
            .unwrap();
            
        match program.attach("tcp_identify_packet_loss", 0) {
            std::result::Result::Ok(_) => {
                info!("program attached successfully to the tcp_identify_packet_loss kprobe ")
            }
            Err(e) => {
                error!(
                    "An error occured while attaching the program to the tcp_identify_packet_loss kprobe. {:?} ",
                    e
                );
            }
        }
    }
    
    info!("Attempting tcp_v4_connect attachment...");
    {
        let mut tcp_bpf = tcp_bpf.lock().unwrap();
        let tcp_program: &mut KProbe = tcp_bpf
            .program_mut("tcp_connect")
            .unwrap()
            .try_into()
            .unwrap();
            
        match tcp_program.attach("tcp_v4_connect", 0) {
            std::result::Result::Ok(_) => {
                info!("program attached successfully to the tcp_v4_connect kprobe ")
            }
            Err(e) => {
                error!(
                    "An error occured while attaching the program to the tcp_v4_connect kprobe. {:?} ",
                    e
                );
            }
        }

        info!("Attempting tcp_v6_connect attachment...");
        match tcp_program.attach("tcp_v6_connect", 0) {
            std::result::Result::Ok(_) => {
                info!("program attached successfully to the tcp_v6_connect kprobe ")
            }
            Err(e) => {
                error!(
                    "An error occured while attaching the program to the tcp_v6_connect kprobe. {:?} ",
                    e
                );
            }
        }
    }

    info!("Attempting tcp_rcv_state_process attachment...");
    {
        let mut tcp_rev_bpf = tcp_rev_bpf.lock().unwrap();
        let tcp_rev_state_program: &mut KProbe = tcp_rev_bpf
            .program_mut("tcp_rcv_state_process")
            .unwrap()
            .try_into()
            .unwrap();
            
        match tcp_rev_state_program.attach("tcp_rcv_state_process", 0) {
            std::result::Result::Ok(_) => {
                info!("program attached successfully to the tcp_rcv_state_process kprobe ")
            }
            Err(e) => {
                error!(
                    "An error occured while attaching the program to the tcp_rcv_state_process kprobe. {:?} ",
                    e
                );
            }
        }
    }

    info!("All attachment attempts completed, setting up perf buffers...");


    info!("Getting CPU count...");
    let cpu_count = online_cpus().map_err(|e| anyhow::anyhow!("Error {:?}", e))?.len();
    info!("CPU count: {}", cpu_count);
    
    info!("Creating perf buffers...");
    let mut net_perf_buffer: Vec<PerfEventArrayBuffer<MapData>> = Vec::new();
    let mut net_perf_array: PerfEventArray<MapData> = PerfEventArray::try_from(net_metrics_map)?;
    let mut time_stamp_events_perf_buffer: Vec<PerfEventArrayBuffer<MapData>> = Vec::new();
    let mut time_stamp_events_perf_array: PerfEventArray<MapData> =
        PerfEventArray::try_from(time_stamps_events_map)?;

    info!("Opening perf buffers for {} CPUs...", cpu_count);
    for cpu_id in online_cpus().map_err(|e| anyhow::anyhow!("Error {:?}", e))? {
        let buf: PerfEventArrayBuffer<MapData> = net_perf_array.open(cpu_id, None)?;
        net_perf_buffer.push(buf);
    }
    for cpu_id in online_cpus().map_err(|e| anyhow::anyhow!("Error {:?}", e))? {
        let buf: PerfEventArrayBuffer<MapData> = time_stamp_events_perf_array.open(cpu_id, None)?;
        time_stamp_events_perf_buffer.push(buf);
    }
    info!("Perf buffers created successfully");

    // Create shared running flags
    let net_metrics_running = Arc::new(AtomicBool::new(true));
    let time_stamp_events_running = Arc::new(AtomicBool::new(true));
    
    // Create proper sized buffers
    let net_metrics_buffers = vec![BytesMut::with_capacity(1024); cpu_count];
    let time_stamp_events_buffers = vec![BytesMut::with_capacity(1024); cpu_count];
    
    // Clone for the signal handler
    let net_metrics_running_signal = net_metrics_running.clone();
    let time_stamp_events_running_signal = time_stamp_events_running.clone();
    
    info!("Starting event listener tasks...");
    let metrics_map_displayer = tokio::spawn(async move{
        display_metrics_map(net_perf_buffer, net_metrics_running, net_metrics_buffers).await;
    });

    let time_stamp_events_displayer = tokio::spawn(async move{
        display_time_stamp_events_map(time_stamp_events_perf_buffer, time_stamp_events_running, time_stamp_events_buffers).await
    });

    info!("Event listeners started, entering main loop...");

    tokio::select! {
        result = metrics_map_displayer => {
            if let Err(e) = result {
                error!("Metrics map displayer task failed: {:?}", e);
            }
        }

        result = time_stamp_events_displayer => {
            if let Err(e) = result {
                error!("Time stamp events displayer task failed: {:?}", e);
            }
        }

        _ = signal::ctrl_c() => {
            info!("Ctrl-C received, shutting down...");
            // Stop the event loops
            net_metrics_running_signal.store(false, std::sync::atomic::Ordering::SeqCst);
            time_stamp_events_running_signal.store(false, std::sync::atomic::Ordering::SeqCst);
        }
    }

    Ok(())
}