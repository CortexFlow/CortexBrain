use anyhow::Error;
use anyhow::Ok;
use aya::Ebpf;
use aya::maps::HashMap;
use aya::maps::Map;
use k8s_openapi::api::core::v1::ConfigMap;
use kube::{Api, Client};
use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;
use tracing::warn;
use tracing::{error, info};

pub fn init_bpf_maps(bpf: Arc<Mutex<Ebpf>>) -> Result<(Map, Map, Map, Map), anyhow::Error> {
    // this function init the bpfs maps used in the main program
    /*
       index 0: events_map
       index 1: veth_map
       index 2: blocklist map
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

    let tcp_registry_map = bpf_new
        .take_map("TcpPacketRegistry")
        .ok_or_else(|| anyhow::anyhow!("TcpPacketRegistry map not found"))?;

    Ok((events_map, veth_map, blocklist_map, tcp_registry_map))
}

//TODO: save bpf maps path in the cli metadata
//takes an array of bpf maps and pin them to persiste session data
//TODO: change maps type with a Vec<Map> instead of (Map,Map). This method is only for fast development and it's not optimized
//TODO: add bpf mounts during cli installation
pub fn map_pinner(maps: &(Map, Map, Map, Map), path: &PathBuf) -> Result<(), Error> {
    if !path.exists() {
        info!("Pin path {:?} does not exist. Creating it...", path);
        std::fs::create_dir_all(&path)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))?;
        }
    }

    let configs = [
        (&maps.0, "events_map"),
        (&maps.1, "veth_map"),
        (&maps.2, "blocklist_map"),
        (&maps.3, "tcp_packet_registry"),
    ];

    for (name, paths) in configs {
        let map_path = path.join(paths);
        if map_path.exists() {
            warn!("Path {} already exists", paths);
            warn!("Removing path {}", paths);
            let _ = std::fs::remove_file(&map_path);
        }
        info!("Trying to pin map {:?} in map path: {:?}", name, &map_path);
        name.pin(&map_path)?;
    }

    Ok(())
}
pub async fn populate_blocklist(map: &mut Map) -> Result<(), Error> {
    let client = Client::try_default().await.unwrap();
    let namespace = "cortexflow";
    let configmap = "cortexbrain-client-config";

    let mut blocklist_map = HashMap::<_, [u8; 4], [u8; 4]>::try_from(map)?;

    let api: Api<ConfigMap> = Api::namespaced(client, namespace);
    match api.get(configmap).await {
        std::result::Result::Ok(configs) => {
            info!("Configmap : {} loaded correctly ", configmap);
            info!("[CONFIGMAP]: {:?} ", configs);
            if let Some(data) = configs.data {
                if let Some(blocklist) = data.get("blocklist") {
                    let addresses: Vec<String> = blocklist
                        .lines()
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    //String parsing from "x y" to ["x","y"]
                    info!("Inserting addresses: {:?}", addresses);
                    for item in addresses {
                        let addr = Ipv4Addr::from_str(&item)?.octets();
                        let _ = blocklist_map.insert(addr, addr, 0);
                    }
                }
            }
            Ok(())
        }
        std::result::Result::Err(e) => {
            error!("An error occured while reading configmap: {}", e);
            return Err(e.into());
        }
    }
}
