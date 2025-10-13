use anyhow::Error;
use anyhow::Ok;
use aya::Bpf;
use aya::maps::Map;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::fs;
use tracing::info;

pub fn init_bpf_maps(bpf: Arc<Mutex<Bpf>>) -> Result<(Map, Map, Map), anyhow::Error> {
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

    let blocklist_map = bpf_new
        .take_map("Blocklist")
        .ok_or_else(|| anyhow::anyhow!("Blocklist map not found"))?;

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
    Ok((events_map, veth_map, blocklist_map))
}

//TODO: save bpf maps path in the cli metadata
//takes an array of bpf maps and pin them to persiste session data
//TODO: change maps type with a Vec<Map> instead of (Map,Map). This method is only for fast development and it's not optimized
//TODO: add bpf mounts during cli installation
pub async fn map_pinner(maps: &(Map, Map, Map), path: &PathBuf) -> Result<(), Error> {
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

    let map1_path = path.join("events_map");
    let map2_path = path.join("veth_map");
    let map3_path = path.join("blocklist_map");

    // maps pinning
    maps.0.pin(&map1_path)?;
    maps.1.pin(&map2_path)?;
    maps.2.pin(&map3_path)?;

    Ok(())
}
