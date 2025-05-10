/* contains the code for the kernel xdp manipulation. this code lives in
the kernel space only and needs to be attached to a program in the user space
*/
mod shared_struct;

use std::error;

use anyhow::Context;
use aya::programs::{Xdp, XdpFlags};
use tokio::fs;
use tokio::signal;
use aya_log::EbpfLogger;
use tracing::{error,info,warn};
use std::path::Path;


use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::EnvFilter;

use aya::maps::{HashMap as UserSpaceMap, MapData};
use crate::shared_struct::{SVCKey,SVCValue};


unsafe impl aya::Pod for shared_struct::SVCKey {}
unsafe impl aya::Pod for shared_struct::SVCValue {}

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

    tracing_subscriber::fmt()
    .with_max_level(tracing::Level::INFO)
    .with_target(false)
    .with_level(true)
    .with_span_events(FmtSpan::NONE)
    .without_time()
    .with_file(false)
    .pretty()
    .with_env_filter(EnvFilter::new("info"))
    .with_line_number(false)
    .init();




    //loading the pre-built binaries--> reason: linux kernel does not accept non compiled code. only accepts bytecode
    info!("loading data");
    let data = fs::read("../../../target/bpfel-unknown-none/release/xdp-filter").await.context("failed to load file from path")?;
    let mut bpf = aya::Ebpf::load(&data).context("failed to load data from file")?;
    let maps_owned = bpf.take_map("services").context("failed to take services map")?;

    if Path::new("/sys/fs/bpf/services").exists(){
        warn!("map already pinned,skipping process");
    }
    else{
        maps_owned.pin("/sys/fs/bpf/services").context("failed to pin map")?;
    }

    let service_map = UserSpaceMap::<MapData, SVCKey, SVCValue>::try_from(maps_owned)?;



    EbpfLogger::init(&mut bpf).context("Cannot initialize ebpf logger");

    //extract the bpf program "xdp-hello" from builded binaries
    let program: &mut Xdp = bpf.program_mut("xdp_hello").unwrap().try_into()?;
    program.load().context("Failed to laod XDP program")?; //load the program

    info!("Starting program");
    program
        .attach("enp0s25", XdpFlags::default())
        .context("Failed to attach XDP program with default flags to interface eth0")?;
    
    //waiting for signint (ctrl-c) to shutdown the program
    info!("Waiting for Ctrl-C...");
    signal::ctrl_c().await?;
    warn!("Exiting");
    info!("Printing existing (key,value) values from the services map ");
    for entry in service_map.iter(){
        match entry {
            Ok((key,value))=>{
                info!("Key: {:?}, value: {:?}",key,value);
            },
            Err(e)=>{
                error!("An error occured: {}",e);
            }
        }
    } 
    Ok(())
}
