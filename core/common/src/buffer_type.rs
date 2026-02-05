use bytemuck_derive::Zeroable;
use bytes::BytesMut;
use std::net::Ipv4Addr;
use tracing::{error, info, warn};

//
// IpProtocols enum to reconstruct the packet protocol based on the
// IPV4 Header Protocol code
//

#[derive(Debug)]
#[repr(u8)]
pub enum IpProtocols {
    ICMP = 1,
    TCP = 6,
    UDP = 17,
}

//
// TryFrom Trait implementation for IpProtocols enum
// This is used to reconstruct the packet protocol based on the
// IPV4 Header Protocol code
//

impl TryFrom<u8> for IpProtocols {
    type Error = ();
    fn try_from(proto: u8) -> Result<Self, Self::Error> {
        match proto {
            1 => Ok(IpProtocols::ICMP),
            6 => Ok(IpProtocols::TCP),
            17 => Ok(IpProtocols::UDP),
            _ => Err(()),
        }
    }
}

//
// Structure PacketLog
//This structure is used to store the packet information
//
#[cfg(feature = "network-structs")]
#[repr(C)]
#[derive(Clone, Copy, Zeroable)]
pub struct PacketLog {
    pub proto: u8,
    pub src_ip: u32,
    pub src_port: u16,
    pub dst_ip: u32,
    pub dst_port: u16,
    pub pid: u32,
}
#[cfg(feature = "network-structs")]
unsafe impl aya::Pod for PacketLog {}

#[cfg(feature = "network-structs")]
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct VethLog {
    pub name: [u8; 16],    // 16 bytes: veth interface name
    pub state: u64,        // 8 bytes: state variable (unsigned long in kernel)
    pub dev_addr: [u8; 6], // 32 bytes: device address
    pub event_type: u8,    // 1 byte: 1 for veth creation, 2 for veth destruction
    pub netns: u32,        // 4 bytes: network namespace inode number
    pub pid: u32,          // 4 bytes: PID that triggered the event
}

#[cfg(feature = "network-structs")]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct TcpPacketRegistry {
    pub proto: u8,
    pub src_ip: u32,
    pub dst_ip: u32,
    pub src_port: u16,
    pub dst_port: u16,
    pub pid: u32,
    pub command: [u8; 16],
    pub cgroup_id: u64,
}

// docs:
// This function perform a byte swap from little-endian to big-endian
// It's used to reconstruct the correct IPv4 address from the u32 representation
//
// Takes a u32 address in big-endian format and returns a Ipv4Addr with reversed octets
//
#[inline(always)]
pub fn reverse_be_addr(addr: u32) -> Ipv4Addr {
    let octects = addr.to_be_bytes();
    let [a, b, c, d] = [octects[3], octects[2], octects[1], octects[0]];
    let reversed_ip = Ipv4Addr::new(a, b, c, d);
    reversed_ip
}

// enum BuffersType
#[cfg(feature = "buffer-reader")]
pub enum BufferType {
    PacketLog,
    TcpPacketRegistry,
    VethLog,
}

// IDEA: this is an experimental implementation to centralize buffer reading logic
// TODO: add variant for cortexflow API exporter
#[cfg(feature = "buffer-reader")]
impl BufferType {
    pub async fn read_packet_log(buffers: &mut [BytesMut], tot_events: i32, offset: i32) {
        for i in offset..tot_events {
            let vec_bytes = &buffers[i as usize];
            if vec_bytes.len() < std::mem::size_of::<PacketLog>() {
                error!(
                    "Corrupted Packet log data. Raw data: {}. Readed {} bytes expected {} bytes",
                    vec_bytes
                        .iter()
                        .map(|b| format!("{:02x}", b))
                        .collect::<Vec<_>>()
                        .join(" "),
                    vec_bytes.len(),
                    std::mem::size_of::<PacketLog>()
                );
                continue;
            }
            if vec_bytes.len() >= std::mem::size_of::<PacketLog>() {
                let pl: PacketLog =
                    unsafe { std::ptr::read_unaligned(vec_bytes.as_ptr() as *const _) }; // reading raw bytes

                // extracting struct info from bytes
                let src_ip = reverse_be_addr(pl.src_ip);
                let dst_ip = reverse_be_addr(pl.dst_ip);
                let src_port = u16::from_be(pl.src_port);
                let dst_port = u16::from_be(pl.dst_port);
                let event_id = pl.pid;
                let protocol = pl.proto;

                // protocol extraction
                match IpProtocols::try_from(protocol) {
                    Ok(proto) => {
                        info!(
                            "Event Id: {} Protocol: {:?} SRC: {}:{} -> DST: {}:{}",
                            event_id, proto, src_ip, src_port, dst_ip, dst_port
                        );
                    }
                    Err(e) => {
                        error!("Unknown protocol. Data maybe corrupted. Reason:{:?}", e);
                    }
                }
            }
        }
    }
    pub async fn read_tcp_registry_log(buffers: &mut [BytesMut], tot_events: i32, offset: i32) {
        for i in offset..tot_events {
            let vec_bytes = &buffers[i as usize];
            if vec_bytes.len() < std::mem::size_of::<TcpPacketRegistry>() {
                error!(
                    "Corrupted data Tcp Registry data. Raw data: {}. Readed {} bytes expected {} bytes",
                    vec_bytes
                        .iter()
                        .map(|b| format!("{:02x}", b))
                        .collect::<Vec<_>>()
                        .join(" "),
                    vec_bytes.len(),
                    std::mem::size_of::<TcpPacketRegistry>()
                );
                continue;
            }
            if vec_bytes.len() >= std::mem::size_of::<TcpPacketRegistry>() {
                let pl: TcpPacketRegistry =
                    unsafe { std::ptr::read_unaligned(vec_bytes.as_ptr() as *const _) }; // reading raw bytes

                // extracting struct info from bytes
                let src = reverse_be_addr(pl.src_ip);
                let dst = reverse_be_addr(pl.dst_ip);
                let src_port = u16::from_be(pl.src_port);
                let dst_port = u16::from_be(pl.dst_port);
                let event_id = pl.pid;
                let command = pl.command.to_vec();
                let end = command
                    .iter()
                    .position(|&x| x == 0)
                    .unwrap_or(command.len());
                let command_str = String::from_utf8_lossy(&command[..end]).to_string();
                let cgroup_id = pl.cgroup_id;
                let protocol = pl.proto;

                // protocol extraction
                match IpProtocols::try_from(protocol) {
                    Ok(proto) => {
                        info!(
                            "Event Id: {} Protocol: {:?} SRC: {}:{} -> DST: {}:{} Command: {} Cgroup_id: {}",
                            event_id,
                            proto,
                            src,
                            src_port,
                            dst,
                            dst_port,
                            command_str,
                            cgroup_id //proc_content
                        );
                    }
                    Err(e) => {
                        error!("Unknown protocol. Data maybe corrupted. Reason:{:?}", e);
                    }
                }
            }
        }
    }
    pub async fn read_and_handle_veth_log(
        buffers: &mut [BytesMut],
        tot_events: i32,
        offset: i32,
    ) {
        for i in offset..tot_events {
            let vec_bytes = &buffers[i as usize];
            if vec_bytes.len() < std::mem::size_of::<VethLog>() {
                error!(
                    "Corrupted data VethLog data. Raw data: {}. Readed {} bytes expected {} bytes",
                    vec_bytes
                        .iter()
                        .map(|b| format!("{:02x}", b))
                        .collect::<Vec<_>>()
                        .join(" "),
                    vec_bytes.len(),
                    std::mem::size_of::<VethLog>()
                );
                continue;
            }
            if vec_bytes.len() >= std::mem::size_of::<VethLog>() {
                let vthl: VethLog =
                    unsafe { std::ptr::read_unaligned(vec_bytes.as_ptr() as *const _) }; // reading raw bytes

                // extracting struct info from bytes
                let name_bytes = vthl.name;
                let dev_addr_bytes = vthl.dev_addr;
                let name = std::str::from_utf8(&name_bytes);
                let state = vthl.state;

                let dev_addr = dev_addr_bytes;
                let netns = vthl.netns;
                let mut event_type = String::new();

                // event_type extraction
                match vthl.event_type {
                    1 => {
                        event_type = "creation".to_string();
                        match name {
                            Ok(veth_name) => {
                                info!(
                                    "[{}] Veth Event: Type: {} Name: {} Dev_addr: {:x?} State: {}",
                                    netns,
                                    event_type,
                                    veth_name.trim_end_matches("\0"),
                                    dev_addr,
                                    state
                                );
                            }
                            Err(e) => {
                                error!(
                                    "Failed to extract veth name during event_type = creation (1).Reason:{}",
                                    e
                                );
                            }
                        }
                    }
                    2 => {
                        event_type = "deletion".to_string();
                        match name {
                            Ok(veth_name) => {
                                info!(
                                    "[{}] Veth Event: Type: {} Name: {} Dev_addr: {:x?} State: {}",
                                    netns,
                                    event_type,
                                    veth_name.trim_end_matches("\0"),
                                    dev_addr,
                                    state
                                );
                            }
                            Err(e) => {
                                error!(
                                    "Failed to extract veth name during event_type = deletion (2).Reason:{}",
                                    e
                                );
                            }
                        }
                    }
                    _ => {
                        warn!("Unknown event type")
                    }
                }
            }
        }
    }
}
