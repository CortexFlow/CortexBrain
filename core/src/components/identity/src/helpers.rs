use nix::net::if_::if_nameindex;
use std::result::Result::Ok;
use tracing::{error, info};

// docs:
// This function checks if the given interface name is in the list of ignored interfaces
// Takes a interface name (iface) as &str and returns true if the interface should be ignored
// Typically we want to ignore eth0,docker0,tunl0,lo interfaces because they are not relevant for the internal monitoring
//
#[inline(always)]
pub fn ignore_iface(iface: &str) -> bool {
    let ignored_interfaces = ["eth0", "docker0", "tunl0", "lo"];
    ignored_interfaces.contains(&iface)
}

// docs:
// This function retrieves the list of veth interfaces on the system, filtering out ignored interfaces with
// the ignore_iface function.
//
#[inline(always)]
pub fn get_veth_channels() -> Vec<String> {
    match if_nameindex() {
        Ok(ifaces) => filter_veth_names(
            ifaces
                .iter()
                .map(|iface| iface.name().to_string_lossy().into_owned()),
        ),
        Err(_) => Vec::new(),
    }
}

// Keeps only the interfaces that are not ignored, preserving order.
pub fn filter_veth_names<I>(names: I) -> Vec<String>
where
    I: IntoIterator<Item = String>,
{
    let mut interfaces: Vec<String> = Vec::new();
    for iface_name in names {
        if !ignore_iface(&iface_name) {
            interfaces.push(iface_name);
        } else {
            info!("skipping interface {:?}", iface_name);
        }
    }
    interfaces
}

// Packs an interface name and the "attached" state into the fixed-size key/value
// layout of the tracked_veth map (names longer than 16 bytes are truncated).
pub fn encode_link_entry(interface: &str) -> ([u8; 16], [u8; 8]) {
    let mut if_bytes = [0u8; 16];
    let interface_bytes = interface.as_bytes();
    let len = interface_bytes.len().min(if_bytes.len());
    if_bytes[..len].copy_from_slice(&interface_bytes[..len]);

    let mut state_bytes = [0u8; 8];
    let state = b"attached";
    let state_len = state.len().min(state_bytes.len());
    state_bytes[..state_len].copy_from_slice(&state[..state_len]);

    (if_bytes, state_bytes)
}

// Attaches the program to every interface and records successful attachments.
// Failures are logged and skipped. Returns the interfaces that were attached.
pub fn attach_and_register<A, I>(ifaces: Vec<String>, mut attach: A, mut insert: I) -> Vec<String>
where
    A: FnMut(&str) -> Result<(), anyhow::Error>,
    I: FnMut([u8; 16], [u8; 8]) -> Result<(), anyhow::Error>,
{
    let mut attached = Vec::new();
    for interface in ifaces {
        match attach(&interface) {
            std::result::Result::Ok(()) => {
                info!(
                    "Program 'identity_classifier' attached to interface {}",
                    interface
                );
                let (key, value) = encode_link_entry(&interface);
                match insert(key, value) {
                    std::result::Result::Ok(_) => {
                        info!("Veth interface {} added into map", &interface);
                    }
                    Err(e) => {
                        error!(
                            "Cannot add Veth interface {} into map. Reason: {}",
                            &interface, e
                        );
                    }
                }
                attached.push(interface);
            }
            Err(e) => error!(
                "Error attaching program to interface {}: {:?}",
                interface, e
            ),
        }
    }
    attached
}

#[cfg(test)]
mod tests {
    use cortexbrain_common::buffer_type::VethLog;
    #[test]
    fn check_veth_log_struct_mem() {
        let mem_test = std::mem::size_of::<VethLog>();
        assert_eq!(mem_test, 39);
    }
    #[test]
    fn test_vethlog_buffer_len() {
        let vethlog = VethLog {
            name: [0; 16],
            dev_addr: [0; 6],
            state: 1,
            netns: 123,
            event_type: 1,
            pid: 1,
        };
        let buffer = unsafe {
            std::slice::from_raw_parts(
                (&vethlog as *const VethLog) as *const u8,
                std::mem::size_of::<VethLog>(),
            )
        };
        assert_eq!(buffer.len(), 39);
    }

    use super::*;

    #[test]
    fn ignore_iface_matches_only_known_names() {
        for n in ["eth0", "docker0", "tunl0", "lo"] {
            assert!(ignore_iface(n));
        }
        assert!(!ignore_iface("veth123"));
        assert!(!ignore_iface("eth01"));
        assert!(!ignore_iface(""));
    }

    #[test]
    fn filter_veth_names_drops_ignored_and_keeps_order() {
        let input = ["lo", "veth1", "eth0", "veth2", "docker0", "tunl0", "cali3"]
            .map(String::from);
        assert_eq!(filter_veth_names(input), vec!["veth1", "veth2", "cali3"]);
    }

    #[test]
    fn filter_veth_names_handles_empty_and_all_ignored() {
        assert!(filter_veth_names(Vec::<String>::new()).is_empty());
        assert!(filter_veth_names(["lo".to_string(), "eth0".to_string()]).is_empty());
    }

    #[test]
    fn filter_veth_names_keeps_duplicates() {
        let input = ["veth1", "veth1"].map(String::from);
        assert_eq!(filter_veth_names(input).len(), 2);
    }

    #[test]
    fn get_veth_channels_never_returns_ignored_interfaces() {
        let channels = get_veth_channels();
        assert!(channels.iter().all(|c| !ignore_iface(c)));
    }

    #[test]
    fn encode_link_entry_pads_short_names() {
        let (k, v) = encode_link_entry("veth0");
        assert_eq!(&k[..5], b"veth0");
        assert!(k[5..].iter().all(|b| *b == 0));
        assert_eq!(&v, b"attached");
    }

    #[test]
    fn encode_link_entry_truncates_long_names() {
        let (k, _) = encode_link_entry("a_very_long_interface_name");
        assert_eq!(&k, b"a_very_long_inte");
    }

    #[test]
    fn attach_and_register_registers_every_attached_interface() {
        let mut inserted = Vec::new();
        let attached = attach_and_register(
            vec!["veth0".into(), "veth1".into()],
            |_| std::result::Result::Ok(()),
            |k, v| {
                inserted.push((k, v));
                std::result::Result::Ok(())
            },
        );
        assert_eq!(attached, vec!["veth0", "veth1"]);
        assert_eq!(inserted.len(), 2);
        assert_eq!(inserted[0], encode_link_entry("veth0"));
    }

    #[test]
    fn attach_and_register_skips_failed_attachments() {
        let mut inserted = 0;
        let attached = attach_and_register(
            vec!["bad".into(), "veth1".into()],
            |i| if i == "bad" { Err(anyhow::anyhow!("boom")) } else { std::result::Result::Ok(()) },
            |_, _| {
                inserted += 1;
                std::result::Result::Ok(())
            },
        );
        assert_eq!(attached, vec!["veth1"]);
        assert_eq!(inserted, 1);
    }

    #[test]
    fn attach_and_register_survives_insert_failure() {
        let attached = attach_and_register(
            vec!["veth0".into(), "veth1".into()],
            |_| std::result::Result::Ok(()),
            |_, _| Err(anyhow::anyhow!("map full")),
        );
        assert_eq!(attached, vec!["veth0", "veth1"]);
    }

    #[test]
    fn attach_and_register_handles_empty_list() {
        let attached = attach_and_register(vec![], |_| Err(anyhow::anyhow!("unused")), |_, _| Err(anyhow::anyhow!("unused")));
        assert!(attached.is_empty());
    }
}
