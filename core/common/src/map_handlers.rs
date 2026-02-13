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
    let mut bpf_new = bpf
        .lock()
        .map_err(|e| anyhow::anyhow!("Cannot get value from lock. Reason: {}", e))?;

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

//takes an array of bpf maps and pin them to persist session data

#[cfg(feature = "map-handlers")]
pub fn map_pinner(maps: BpfMapsData, path: &PathBuf) -> Result<BpfMapsData, Error> {
    if !path.exists() {
        info!("Pin path {:?} does not exist. Creating it...", path);
        std::fs::create_dir_all(&path)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))?;
        }
    }

    //let mut owned_maps = Vec::new(); // aya::Maps does not implement the clone trait i need to create a raw copy of the vec map
    let mut owned_bpf_maps_data = BpfMapsData {
        bpf_obj_names: Vec::new(),
        bpf_obj_map: Vec::new(),
    };
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
        //owned_maps.push(map_obj);
        owned_bpf_maps_data.bpf_obj_names.push(name);
        owned_bpf_maps_data.bpf_obj_map.push(map_obj);
    }

    Ok(owned_bpf_maps_data) // return a BpfMapsData type 
}

#[cfg(feature = "map-handlers")]
pub async fn populate_blocklist() -> Result<(), Error> {
    use aya::maps::MapData;
    // load mapdata from path

    let mapdata = MapData::from_pin("/sys/fs/bpf/maps/Blocklist")
        .map_err(|e| anyhow::anyhow!("Failed to load blocklist_map: {}", e))?;

    let map = Map::HashMap(mapdata);
    let mut blocklist_map = HashMap::<_, [u8; 4], [u8; 4]>::try_from(map)?;

    let client = Client::try_default()
        .await
        .expect("Cannot connect to Kubernetes Client");
    let namespace = "cortexflow";
    let configmap = "cortexbrain-client-config";

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
                    if addresses.is_empty() {
                        warn!("No addresses found in the blocklist. Skipping load");
                    }
                    for item in &addresses {
                        info!("Inserting addresses: {:?}", &item);
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

#[cfg(feature = "map-handlers")]
// TODO: modify this to accept also HashMap types
pub fn load_perf_event_array_from_mapdata(
    path: &'static str,
) -> Result<aya::maps::PerfEventArray<aya::maps::MapData>, Error> {
    use aya::maps::MapData;
    use aya::maps::PerfEventArray;

    let map_data = MapData::from_pin(path)
        .map_err(|e| anyhow::anyhow!("Cannot load mapdata from pin {:?} .Reason: {}", &path, e))?;

    let map = Map::PerfEventArray(map_data);

    let perf_event_array = PerfEventArray::try_from(map).map_err(|e| {
        anyhow::anyhow!("Cannot initialize perf_event_array from map. Reason: {}", e)
    })?;
    Ok(perf_event_array)
}

#[cfg(feature = "map-handlers")]
pub fn map_manager(
    maps: BpfMapsData,
) -> Result<
    std::collections::HashMap<
        String,
        (
            aya::maps::PerfEventArray<aya::maps::MapData>,
            Vec<aya::maps::perf::PerfEventArrayBuffer<aya::maps::MapData>>,
        ),
    >,
    Error,
> {
    use aya::maps::PerfEventArray;
    use aya::maps::{MapData, perf::PerfEventArrayBuffer};
    use tracing::debug;

    let mut map_manager = std::collections::HashMap::<
        String, // this will store the bpf map name
        (PerfEventArray<MapData>, Vec<PerfEventArrayBuffer<MapData>>), // this will manage the BPF_MAP_TYPE_PERF_EVENT_ARRAY and its buffer
    >::new();

    // map_manager creates an hashmap that contains:
    // MAP NAME as String (KEY)
    //
    // VALUES (tuple)
    // a PERF_EVENT_ARRAY
    // a vector of PERF_EVENT_ARRAY_BUFFER
    //
    // the map manager helps the event listener to specifically call a map by its pinned name
    // e.g. veth_identity_map and returns the associated PERF_EVENT_ARRAY and PERF_EVENT_ARRAY_BUFFERS (1 per CPU)
    // also the map manager helps to write a more complete debug context by linking map names with arrays and buffers.
    // actually i cannot return the extact information using only the Aya library

    // create the PerfEventArrays and the buffers from the BpfMapsData Objects
    for (map, name) in maps
        .bpf_obj_map
        .into_iter()
        .zip(maps.bpf_obj_names.into_iter())
    // zip two iterators at the same time for map object and map names
    {
        debug!("Debugging map type:{:?} for map name {:?}", map, &name);
        info!("Creating PerfEventArray for map name {:?}", &name);

        // save the map in a registry if is a PerfEventArray to access them by name
        if let std::result::Result::Ok(perf_event_array) = PerfEventArray::try_from(map) {
            map_manager.insert(name.clone(), (perf_event_array, Vec::new()));
        } else {
            warn!("Map {:?} is not a PerfEventArray, skipping load", &name);
        }
    }
    Ok(map_manager)
}
