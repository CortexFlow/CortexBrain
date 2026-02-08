use aya::{Ebpf, programs::KProbe};
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

    // Load and attach the eBPF programs
    let program: &mut KProbe = bpf_new
        .program_mut(program_name)
        .ok_or_else(|| anyhow::anyhow!("Program {} not found", program_name))?
        .try_into()
        .map_err(|e| anyhow::anyhow!("Failed to convert program: {:?}", e))?;

    program
        .load()
        .map_err(|e| anyhow::anyhow!("Cannot load program: {}. Error: {}", &program_name, e))?;

    match program.attach(kernel_symbol, 0) {
        Ok(_) => info!("{} program attached successfully", kernel_symbol),
        Err(e) => {
            error!("Error attaching {} program {:?}", kernel_symbol, e);
            return Err(anyhow::anyhow!(
                "Failed to attach {}: {:?}",
                kernel_symbol,
                e
            ));
        }
    };

    info!(
        "eBPF program {} loaded and attached successfully",
        program_name
    );
    Ok(())
}
