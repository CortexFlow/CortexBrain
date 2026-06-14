use aya::{
    Ebpf,
    programs::{KProbe, TracePoint},
};
use std::convert::TryInto;
use std::sync::{Arc, Mutex};
use tracing::{error, info};

#[cfg(feature = "program-handlers")]
pub fn load_program(
    bpf: Arc<Mutex<Ebpf>>,
    program_name: &str,
    kernel_symbol: &str,
) -> Result<(), anyhow::Error> {
    let mut bpf_new = bpf
        .lock()
        .map_err(|e| anyhow::anyhow!("Cannot get value from lock. Reason: {}", e))?;

    // Load and attach the eBPF program
    let program: &mut KProbe = bpf_new
        .program_mut(program_name)
        .ok_or_else(|| anyhow::anyhow!("Program {} not found", program_name))?
        .try_into()
        .map_err(|e| anyhow::anyhow!("Failed to convert program: {:?}", e))?;

    // STEP 1: load program

    program
        .load()
        .map_err(|e| anyhow::anyhow!("Cannot load program: {}. Error: {}", &program_name, e))?;

    // STEP 2: Attach the loaded program to kernel symbol
    match program.attach(kernel_symbol, 0) {
        Ok(_) => info!(
            "{} program attached successfully to kernel symbol {}",
            &program_name, &kernel_symbol
        ),
        Err(e) => {
            error!(
                "Error attaching {} program to kernel symbol {}. Reason: {:?}",
                &program_name, &kernel_symbol, e
            );
            return Err(anyhow::anyhow!(
                "Failed to attach program {} to kernel symbol {}. Reason {:?}",
                &program_name,
                &kernel_symbol,
                e
            ));
        }
    };

    Ok(())
}

#[cfg(feature = "program-handlers")]
pub fn load_tracepoint_program(
    bpf: Arc<Mutex<Ebpf>>,
    program_name: &str,
    tracepoint_type: &str,
    tracepoint_symbol: &str,
) -> Result<(), anyhow::Error> {
    let mut bpf_new = bpf
        .lock()
        .map_err(|e| anyhow::anyhow!("Cannot get value from lock. Reason: {}", e))?;

    // Load and attach the eBPF program
    let program: &mut TracePoint = bpf_new
        .program_mut(program_name)
        .ok_or_else(|| anyhow::anyhow!("Program {} not found", program_name))?
        .try_into()
        .map_err(|e| anyhow::anyhow!("Failed to convert program: {:?}", e))?;

    // STEP 1: load program

    program
        .load()
        .map_err(|e| anyhow::anyhow!("Cannot load program: {}. Error: {}", &program_name, e))?;

    // STEP 2: Attach the loaded program to kernel symbol
    match program.attach(tracepoint_type, tracepoint_symbol) {
        Ok(_) => info!(
            "{} program attached successfully to tracepoint  {}",
            &program_name, &tracepoint_symbol
        ),
        Err(e) => {
            error!(
                "Error attaching {} program to tracepoint  {}. Reason: {:?}",
                &program_name, &tracepoint_symbol, e
            );
            return Err(anyhow::anyhow!(
                "Failed to attach program {} to tracepoint  {}. Reason {:?}",
                &program_name,
                &tracepoint_symbol,
                e
            ));
        }
    };

    Ok(())
}
