use aya::{Ebpf, programs::KProbe};
use std::convert::TryInto;
use std::sync::{Arc, Mutex};
use tracing::{error, info};

#[cfg(feature = "program-handlers")]
pub fn load_program(
    bpf: Arc<Mutex<Ebpf>>,
    program_name: &str,
    actual_program: &str,
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

    match program.attach(actual_program, 0) {
        Ok(_) => info!("{} program attached successfully", actual_program),
        Err(e) => {
            error!("Error attaching {} program {:?}", actual_program, e);
            return Err(anyhow::anyhow!(
                "Failed to attach {}: {:?}",
                actual_program,
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
