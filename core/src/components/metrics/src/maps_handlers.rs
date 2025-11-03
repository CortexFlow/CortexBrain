use std::{path::PathBuf, sync::{Arc, Mutex}};
use tokio::fs;
use anyhow::Error;
use aya::{maps::Map, Ebpf};
use tracing::info;



pub fn init_ebpf_maps(bpf: Arc<Mutex<Ebpf>>) -> Result<(Map, Map), anyhow::Error> {
    // this function init the bpfs maps used in the main program
    /*
       index 0: net_metrics
       index 1: time_stamp_events
    */
    let mut bpf_new = bpf.lock().unwrap();

    let net_metrics_map = bpf_new
        .take_map("net_metrics")
        .ok_or_else(|| anyhow::anyhow!("net_metrics map not found"))?;

    let time_stamps_events_map = bpf_new
        .take_map("time_stamp_events")
        .ok_or_else(|| anyhow::anyhow!("time_stamp_events map not found"))?;

    Ok((net_metrics_map, time_stamps_events_map))
}

pub async fn map_pinner(maps: &(Map, Map), path: &PathBuf) -> Result<(), Error> {
    // check if the map exists
    if !path.exists() {
        info!("Pin path {:?} does not exist. Creating it...", path);
        fs::create_dir_all(&path).await?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).await?;
        }
    }

    let map1_path = path.join("net_metrics");
    let map2_path = path.join("time_stamp_events");

    // maps pinning
    maps.0.pin(&map1_path)?;
    maps.1.pin(&map2_path)?;

    Ok(())
}
