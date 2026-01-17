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

// docs
//
// this function init the bpfs maps used in the main program
//
//  index 0: events_map
//  index 1: veth_map
//  index 2: blocklist map
//  index 3: tcp_registry map
//

#[cfg(feature = "map-handlers")]
pub struct BpfMapsData {
    pub bpf_obj_names: Vec<String>,
    pub bpf_obj_map: Vec<Map>,
}

#[cfg(feature = "map-handlers")]
pub fn init_bpf_maps(
    bpf: Arc<Mutex<Ebpf>>,
    map_names: Vec<String>,
) -> Result<BpfMapsData, anyhow::Error> {
    let mut bpf_new = bpf.lock().expect("Cannot get value from lock");
    let mut maps = Vec::new(); // stores bpf_maps_objects

    for name in &map_names {
        let bpf_map_init = bpf_new
            .take_map(&name)
            .ok_or_else(|| anyhow::anyhow!("{} map not found", &name))?;
        maps.push(bpf_map_init);
    }
    Ok(BpfMapsData {
        bpf_obj_names: map_names.clone(),
        bpf_obj_map: maps,
    })
}

//TODO: save bpf maps path in the cli metadata

//takes an array of bpf maps and pin them to persiste session data
// FIXME: is this ok that we are returning a BpfMapsData?

#[cfg(feature = "map-handlers")]
pub fn map_pinner(maps: BpfMapsData, path: &PathBuf) -> Result<Vec<Map>, Error> {
    if !path.exists() {
        info!("Pin path {:?} does not exist. Creating it...", path);
        std::fs::create_dir_all(&path)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))?;
        }
    }

    let mut owned_maps = Vec::new(); // aya::Maps does not implement the clone trait i need to create a raw copy of the vec map
    // an iterator that iterates two iterators simultaneously
    for (map_obj, name) in maps
        .bpf_obj_map
        .into_iter()
        .zip(maps.bpf_obj_names.into_iter())
    {
        let map_path = path.join(&name);
        if map_path.exists() {
            warn!("Path {} already exists", name);
            warn!("Removing path {}", name);
            std::fs::remove_file(&map_path)?;
        }
        info!("Trying to pin map {:?} in map path: {:?}", name, &map_path);
        map_obj.pin(&map_path)?;
        owned_maps.push(map_obj);
    }

    Ok(owned_maps)
}

#[cfg(feature = "map-handlers")]
pub async fn populate_blocklist(map: &mut Map) -> Result<(), Error> {
    let client = Client::try_default()
        .await
        .expect("Cannot connect to Kubernetes Client");
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
