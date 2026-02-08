use aya::maps::perf::PerfEventArrayBuffer;
use cortexbrain_common::buffer_type::BufferType;
use nix::net::if_::if_nameindex;
use std::result::Result::Ok;
use tracing::{error, info};

// docs:
// This function checks if the given interface name is in the list of ignored interfaces
// Takes a interface name (iface) as &str and returns true if the interface should be ignored
// Typically we want to ignore eth0,docker0,tunl0,lo interfaces because they are not relevant for the internal monitoring
//
pub fn ignore_iface(iface: &str) -> bool {
    let ignored_interfaces = ["eth0", "docker0", "tunl0", "lo"];
    ignored_interfaces.contains(&iface)
}

// docs:
// This function retrieves the list of veth interfaces on the system, filtering out ignored interfaces with
// the ignore_iface function.
//
pub fn get_veth_channels() -> Vec<String> {
    //filter interfaces and save the output in the
    let mut interfaces: Vec<String> = Vec::new();

    if let Ok(ifaces) = if_nameindex() {
        for iface in &ifaces {
            let iface_name = iface.name().to_str().unwrap().to_owned();
            if !ignore_iface(&iface_name) {
                interfaces.push(iface_name);
            } else {
                info!("skipping interface {:?}", iface_name);
            }
        }
    }

    interfaces
}

// docs: read buffer function:
// template function that take a mut perf_event_array_buffer of type T and a mutable buffer of Vec<BytesMut>

pub async fn read_perf_buffer<T: std::borrow::BorrowMut<aya::maps::MapData>>(
    mut array_buffers: Vec<PerfEventArrayBuffer<T>>,
    mut buffers: Vec<bytes::BytesMut>,
    buffer_type: BufferType,
) {
    // loop over the buffers
    loop {
        for buf in array_buffers.iter_mut() {
            match buf.read_events(&mut buffers) {
                Ok(events) => {
                    // triggered if some events are lost
                    if events.lost > 0 {
                        tracing::debug!("Lost events: {} ", events.lost);
                    }
                    // triggered if some events are readed
                    if events.read > 0 {
                        tracing::debug!("Readed events: {}", events.read);
                        let offset = 0;
                        let tot_events = events.read as i32;

                        //read the events in the buffer
                        match buffer_type {
                            BufferType::PacketLog => {
                                BufferType::read_packet_log(&mut buffers, tot_events, offset).await
                            }
                            BufferType::TcpPacketRegistry => {
                                BufferType::read_tcp_registry_log(&mut buffers, tot_events, offset)
                                    .await
                            }
                            BufferType::VethLog => {
                                BufferType::read_and_handle_veth_log(
                                    &mut buffers,
                                    tot_events,
                                    offset,
                                )
                                .await
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Cannot read events from buffer. Reason: {} ", e);
                }
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await; // small sleep 
    }
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
}
