use anyhow::Error;
use anyhow::Ok;
use aya::Bpf;
use aya::maps::Map;
use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::fs;
use tracing::{error, info};
use aya::maps::HashMap;
use k8s_openapi::api::core::v1::ConfigMap;
use kube::{Api, Client};

pub fn init_bpf_maps(bpf: Arc<Mutex<Bpf>>) -> Result<(Map, Map, Map), anyhow::Error> {
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

    //

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
pub async fn populate_blocklist(map: &mut Map) -> Result<(), Error> {
    let client = Client::try_default().await.unwrap();
    let namespace = "cortexflow";
    let configmap = "cortexbrain-client-config";

    let mut blocklist_map =HashMap::<_, [u8; 4],[u8;4]>::try_from(map)?;


    let api: Api<ConfigMap> = Api::namespaced(client, namespace);
    match api.get(configmap).await {
        std::result::Result::Ok(configs) => {
            info!("Configmap : {} loaded correctly ", configmap);
            info!("[CONFIGMAP]: {:?} ", configs);
            if let Some(data) = configs.data {
                if let Some(blocklist) = data.get("blocklist") {
                    match serde_yaml::from_str::<Vec<String>>(&blocklist) {
                        std::result::Result::Ok(addresses) => {
                            info!("Inserting addresses: {:?}", addresses);

                            for item in addresses{
                                let addr = Ipv4Addr::from_str(&item)?.octets();
                                let _ = blocklist_map.insert(addr,addr,0);
                            }

                        }
                        std::result::Result::Err(e) => {
                            error!("Error during blocklist addresses import: {}", e);
                        }
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
