use crate::constants::{
    BLOCKLIST_FILE_ENV, BLOCKLIST_MAP_NAME, DEFAULT_PIN_MAP_PATH, PIN_MAP_PATH,
};
use anyhow::Error;
use anyhow::Ok;
use aya::Ebpf;
use aya::maps::HashMap;
use aya::maps::Map;
use k8s_openapi::api::core::v1::ConfigMap;
use kube::{Api, Client};
use std::collections::HashSet;
use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use tracing::{debug, error, info, warn};

// docs
//
// this function init the bpfs maps used in the main program
//
//

#[cfg(feature = "map-handlers")]
pub struct BpfMapsData {
    pub bpf_obj_names: Vec<String>,
    pub bpf_obj_map: Vec<Map>,
    pub bpf: Arc<Mutex<Ebpf>>,
}

#[cfg(feature = "map-handlers")]
pub fn init_bpf_maps(
    bpf: Arc<Mutex<Ebpf>>,
    map_names: Vec<String>,
) -> Result<BpfMapsData, anyhow::Error> {
    let mut maps = Vec::new(); // stores bpf_maps_objects

    {
        let mut bpf_new = bpf
            .lock()
            .map_err(|e| anyhow::anyhow!("Cannot get value from lock. Reason: {}", e))?;

        for name in &map_names {
            let bpf_map_init = bpf_new
                .take_map(&name)
                .ok_or_else(|| anyhow::anyhow!("{} map not found", &name))?;
            maps.push(bpf_map_init);
        }
    }

    Ok(BpfMapsData {
        bpf_obj_names: map_names.clone(),
        bpf_obj_map: maps,
        bpf,
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
        bpf: maps.bpf,
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

/// Return the pinned path of the blocklist map.
#[cfg(feature = "map-handlers")]
pub fn blocklist_map_path() -> PathBuf {
    let pin_path = std::env::var(PIN_MAP_PATH)
        .ok()
        .filter(|path| !path.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_PIN_MAP_PATH.to_string());

    PathBuf::from(pin_path).join(BLOCKLIST_MAP_NAME)
}

fn blocklist_changes(
    current: &HashSet<[u8; 4]>,
    desired: &HashSet<[u8; 4]>,
) -> (Vec<[u8; 4]>, Vec<[u8; 4]>) {
    let added = desired.difference(current).copied().collect();
    let removed = current.difference(desired).copied().collect();
    (added, removed)
}

fn parse_blocklist_addresses(raw: &str) -> HashSet<[u8; 4]> {
    let mut addresses = HashSet::new();

    for line in raw.lines() {
        let item = line.trim();
        if item.is_empty() || item.starts_with('#') {
            continue;
        }
        match Ipv4Addr::from_str(item) {
            std::result::Result::Ok(address) => {
                addresses.insert(address.octets());
            }
            Err(e) => warn!("Invalid blocklist address {:?}: {}", item, e),
        }
    }

    addresses
}

async fn blocklist_from_kubernetes() -> Result<Option<HashSet<[u8; 4]>>, Error> {
    let client = match Client::try_default().await {
        std::result::Result::Ok(client) => client,
        Err(e) => {
            debug!("Kubernetes API not available: {}", e);
            return Ok(None);
        }
    };

    if let Err(e) = client.apiserver_version().await {
        debug!("Cannot reach the Kubernetes API server: {}", e);
        return Ok(None);
    }

    let namespace = "cortexflow";
    let configmap = "cortexbrain-client-config";
    let api: Api<ConfigMap> = Api::namespaced(client, namespace);

    match api.get(configmap).await {
        std::result::Result::Ok(configs) => {
            let raw = configs
                .data
                .and_then(|data| data.get("blocklist").cloned())
                .unwrap_or_default();
            Ok(Some(parse_blocklist_addresses(&raw)))
        }
        std::result::Result::Err(e) => {
            error!("An error occured while reading configmap: {}", e);
            Err(e.into())
        }
    }
}

/// Synchronise the pinned blocklist map with its configured source.
///
/// The Kubernetes ConfigMap is preferred. If Kubernetes is not available, a
/// local `BLOCKLIST_FILE` is used when configured. Returns `false` when no
/// source is available and the map was left untouched.
#[cfg(feature = "map-handlers")]
pub async fn populate_blocklist() -> Result<bool, Error> {
    use aya::maps::MapData;

    let desired = match blocklist_from_kubernetes().await? {
        Some(addresses) => {
            info!(
                "Blocklist loaded from ConfigMap ({} entries)",
                addresses.len()
            );
            addresses
        }
        None => {
            let std::result::Result::Ok(path) = std::env::var(BLOCKLIST_FILE_ENV) else {
                return Ok(false);
            };
            if path.trim().is_empty() {
                return Ok(false);
            }

            match std::fs::read_to_string(&path) {
                std::result::Result::Ok(raw) => {
                    let addresses = parse_blocklist_addresses(&raw);
                    info!(
                        "Blocklist loaded from {} ({} entries)",
                        path,
                        addresses.len()
                    );
                    addresses
                }
                Err(e) => {
                    warn!("Cannot read blocklist file {}: {}", path, e);
                    return Ok(false);
                }
            }
        }
    };

    let map_path = blocklist_map_path();
    let mapdata = MapData::from_pin(&map_path)
        .map_err(|e| anyhow::anyhow!("Failed to load blocklist map from {:?}: {}", map_path, e))?;
    let map = Map::HashMap(mapdata);
    let mut blocklist_map = HashMap::<_, [u8; 4], [u8; 4]>::try_from(map)?;

    let current: HashSet<[u8; 4]> = blocklist_map.keys().filter_map(|key| key.ok()).collect();
    let (added, removed) = blocklist_changes(&current, &desired);

    for address in &removed {
        blocklist_map.remove(address)?;
    }
    for address in &added {
        blocklist_map.insert(*address, *address, 0)?;
    }

    if !added.is_empty() || !removed.is_empty() {
        info!(
            "Blocklist updated: {} added, {} removed, {} total",
            added.len(),
            removed.len(),
            desired.len()
        );
    }

    Ok(true)
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

#[cfg(test)]
mod tests {
    use super::{blocklist_changes, parse_blocklist_addresses};
    use std::collections::HashSet;
    use std::net::Ipv4Addr;

    #[test]
    fn parses_blocklist_lines_and_skips_invalid_entries() {
        let raw = "# comment\n\n192.0.2.1\nnot-an-ip\n198.51.100.2\n";
        let parsed = parse_blocklist_addresses(raw);

        assert_eq!(parsed.len(), 2);
        assert!(parsed.contains(&Ipv4Addr::new(192, 0, 2, 1).octets()));
        assert!(parsed.contains(&Ipv4Addr::new(198, 51, 100, 2).octets()));
    }

    #[test]
    fn computes_added_and_removed_blocklist_entries() {
        let current = HashSet::from([
            Ipv4Addr::new(192, 0, 2, 1).octets(),
            Ipv4Addr::new(192, 0, 2, 2).octets(),
        ]);
        let desired = HashSet::from([
            Ipv4Addr::new(192, 0, 2, 2).octets(),
            Ipv4Addr::new(192, 0, 2, 3).octets(),
        ]);

        let (added, removed) = blocklist_changes(&current, &desired);

        assert_eq!(added, vec![Ipv4Addr::new(192, 0, 2, 3).octets()]);
        assert_eq!(removed, vec![Ipv4Addr::new(192, 0, 2, 1).octets()]);
    }
}
