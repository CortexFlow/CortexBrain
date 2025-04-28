/* contains the code for the kernel xdp manipulation. this code lives in
the kernel space only and needs to be attached to a program in the user space
*/

use anyhow::Context;
use aya::programs::{Xdp, XdpFlags};
use log::info;
use tokio::fs;
use tokio::signal;

/*
XDP flags
Mode | Description | Compatibility | Performance
DRIVER_MODE | XDP native in the driver | Only compatible cards | Highest
SKB_MODE | XDP on top of Linux stack | Always compatible | Good
HW_MODE | XDP on hardware | Requires hardware support | Highest (very rare)
*/

//main program
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    //loading the pre-built binaries--> reason: linux kernel does not accept non compiled code. only accepts bytecode
    info!("loading data");
    let data = fs::read("../../../target/bpfel-unknown-none/release/xdp").await?;
    let mut bpf = aya::Ebpf::load(&data)?;

    //extract the bpf program "xdp-hello" from builded binaries
    let program: &mut Xdp = bpf.program_mut("xdp_hello").unwrap().try_into()?;
    program.load().context("Failed to laod XDP program")?; //load the program

    info!("Starting program");
    program
        .attach("enp0s3", XdpFlags::default())
        .context("Failed to attach XDP program with default flags to interface eth0")?;

    //waiting for signint (ctrl-c) to shutdown the program
    info!("Waiting for Ctrl-C...");
    signal::ctrl_c().await?;
    info!("Exiting");
    Ok(())
}
