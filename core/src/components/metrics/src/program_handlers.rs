use std::sync::{Arc, Mutex};

use aya::{programs::KProbe, Ebpf};
use tracing::{info, error};
use std::convert::TryInto;

pub fn load_program(bpf: Arc<Mutex<Ebpf>>, program_name: &str, actual_program: &str) -> Result<(), anyhow::Error> {
    let mut bpf_new = bpf.lock().unwrap();

    // Load and attach the eBPF programs
    let program: &mut KProbe = bpf_new
        .program_mut(program_name)
        .ok_or_else(|| anyhow::anyhow!("Program {} not found", program_name))?
        .try_into()
        .map_err(|e| anyhow::anyhow!("Failed to convert program: {:?}", e))?;

    program.load()?;

    match program.attach(actual_program, 0) {
        Ok(_) => info!("{} program attached successfully", actual_program),
        Err(e) => {
            error!("Error attaching {} program {:?}", actual_program, e);
            return Err(anyhow::anyhow!("Failed to attach {}: {:?}", actual_program, e));
        }
    };

    info!("eBPF program {} loaded and attached successfully", program_name);
    Ok(())
}

pub fn load_and_attach_tcp_programs(bpf: Arc<Mutex<Ebpf>>) -> Result<(), anyhow::Error> {
    let mut bpf_new = bpf.lock().unwrap();

    // Load and attach the eBPF programs
    let tcp_prog: &mut KProbe = bpf_new
        .program_mut("tcp_connect")
        .ok_or_else(|| anyhow::anyhow!("Program tcp_connect not found"))?
        .try_into()
        .map_err(|e| anyhow::anyhow!("Failed to convert program tcp_connect: {:?}", e))?;
    tcp_prog.load()?;

    match tcp_prog.attach("tcp_v4_connect", 0) {
        Ok(_) => info!("tcp_v4_connect program attached successfully"),
        Err(e) => {
            error!("Error attaching tcp_v4_connect: {:?}", e);
            return Err(anyhow::anyhow!("Failed to attach tcp_v4_connect: {:?}", e));
        }
    };

    match tcp_prog.attach("tcp_v6_connect", 0) {
        Ok(_) => info!("tcp_v6_connect program attached successfully"),
        Err(e) => {
            error!("Error attaching tcp_v6_connect: {:?}", e);
            return Err(anyhow::anyhow!("Failed to attach tcp_v6_connect: {:?}", e));
        }
    };

    Ok(())
}