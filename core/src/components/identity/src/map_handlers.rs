use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::Arc;
use anyhow::Error;
use aya::maps::Map;
use aya::Bpf;
use tracing::{error};
use tokio::fs;


pub fn init_bpf_maps(bpf: Arc<Mutex<Bpf>>) -> Result<(Map, Map), anyhow::Error> {
    // this function init the bpfs maps used in the main program
    /*
       index 0: events_map
       index 1: veth_map
    */
    let mut bpf_new = bpf.lock().unwrap();

    let events_map = bpf_new
        .take_map("EventsMap")
        .ok_or_else(|| anyhow::anyhow!("EventsMap map not found"))?;

    let veth_map = bpf_new
        .take_map("veth_identity_map")
        .ok_or_else(|| anyhow::anyhow!("veth_identity_map map not found"))?;

    /* EDIT: this part is paused right now
       info!("loading bpf connections map");

       //init connection map
       let connections_map_raw = bpf
           .take_map("ConnectionMap")
           .context("failed to take connections map")?;

       let connection_tracker_map = bpf
           .take_map("ConnectionTrackerMap")
           .context("failed to take ConnectionTrackerMap map")?;
    */
    Ok((events_map, veth_map))
}

//TODO: save bpf maps path in the cli metadata
//takes an array of bpf maps and pin them to persiste session data
//TODO: change maps type with a Vec<Map> instead of (Map,Map). This method is only for fast development and it's not optimized
//TODO: chmod 700 <path> to setup the permissions to pin maps TODO:add this permission in the CLI
//TODO: add bpf mounts during cli installation
pub async fn map_pinner(maps: &(Map, Map), path: &PathBuf) -> Result<(), Error> {
    
    //FIXME: add exception for already pinned maps 
    if !path.exists() {
        error!("Pin path {:?} does not exist. Creating it...", path);
        let _ = fs::create_dir_all(path)
            .await
            .map_err(|e| error!("Failed to create directory: {}", e));
    }

    let map1_path = path.join("events_map");
    let map2_path = path.join("veth_map");

    maps.0.pin(&map1_path)?;
    maps.1.pin(&map2_path)?;

    Ok(())
}
