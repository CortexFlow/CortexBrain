#[cfg(feature = "buffer-reader")]
use aya::maps::{MapData, PerfEventArray};
use aya::{maps::perf::PerfEventArrayBuffer, util::online_cpus};
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
#[derive(Clone, Copy, Zeroable)]
pub struct VethLog {
    pub name: [u8; 16],    // 16 bytes: veth interface name
    pub state: u64,        // 8 bytes: state variable (unsigned long in kernel)
    pub dev_addr: [u8; 6], // 6 bytes: device address
    pub event_type: u8,    // 1 byte: 1 for veth creation, 2 for veth destruction
    pub netns: u32,        // 4 bytes: network namespace inode number
    pub pid: u32,          // 4 bytes: PID that triggered the event
}
#[cfg(feature = "network-structs")]
unsafe impl aya::Pod for VethLog {}

#[cfg(feature = "network-structs")]
#[repr(C)]
#[derive(Clone, Copy, Zeroable)]
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
#[cfg(feature = "network-structs")]
unsafe impl aya::Pod for TcpPacketRegistry {}

#[cfg(feature = "monitoring-structs")]
pub const TASK_COMM_LEN: usize = 16; // linux/sched.h
#[cfg(feature = "monitoring-structs")]
#[repr(C, packed)]
#[derive(Clone, Copy, Zeroable)]
pub struct NetworkMetrics {
    pub tgid: u32,
    pub comm: [u8; TASK_COMM_LEN],
    pub ts_us: u64,
    pub sk_err: i32,                 // Offset 284
    pub sk_err_soft: i32,            // Offset 600
    pub sk_backlog_len: i32,         // Offset 196
    pub sk_write_memory_queued: i32, // Offset 376
    pub sk_receive_buffer_size: i32, // Offset 244
    pub sk_ack_backlog: u32,         // Offset 604
    pub sk_drops: i32,               // Offset 136
}
#[cfg(feature = "monitoring-structs")]
unsafe impl aya::Pod for NetworkMetrics {}

#[cfg(feature = "monitoring-structs")]
#[repr(C, packed)]
#[derive(Clone, Copy, Zeroable)]
pub struct TimeStampMetrics {
    pub delta_us: u64,
    pub ts_us: u64,
    pub tgid: u32,
    pub comm: [u8; TASK_COMM_LEN],
    pub lport: u16,
    pub dport_be: u16,
    pub af: u16,
    pub saddr_v4: u32,
    pub daddr_v4: u32,
    pub saddr_v6: [u32; 4],
    pub daddr_v6: [u32; 4],
}
#[cfg(feature = "monitoring-structs")]
unsafe impl aya::Pod for TimeStampMetrics {}

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
    #[cfg(feature = "network-structs")]
    PacketLog,
    #[cfg(feature = "network-structs")]
    TcpPacketRegistry,
    #[cfg(feature = "network-structs")]
    VethLog,
    #[cfg(feature = "monitoring-structs")]
    NetworkMetrics,
    #[cfg(feature = "monitoring-structs")]
    TimeStampMetrics,
}

// IDEA: this is an experimental implementation to centralize buffer reading logic
// TODO: add variant for cortexflow API exporter
#[cfg(feature = "buffer-reader")]
impl BufferType {
    #[cfg(feature = "network-structs")]
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
    #[cfg(feature = "network-structs")]
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
    #[cfg(feature = "network-structs")]
    pub async fn read_and_handle_veth_log(buffers: &mut [BytesMut], tot_events: i32, offset: i32) {
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
    #[cfg(feature = "monitoring-structs")]
    pub async fn read_network_metrics(buffers: &mut [BytesMut], tot_events: i32, offset: i32) {
        for i in offset..tot_events {
            let vec_bytes = &buffers[i as usize];
            if vec_bytes.len() < std::mem::size_of::<NetworkMetrics>() {
                error!(
                    "Corrupted Network Metrics data. Raw data: {}. Readed {} bytes expected {} bytes",
                    vec_bytes
                        .iter()
                        .map(|b| format!("{:02x}", b))
                        .collect::<Vec<_>>()
                        .join(" "),
                    vec_bytes.len(),
                    std::mem::size_of::<NetworkMetrics>()
                );
                continue;
            }
            if vec_bytes.len() >= std::mem::size_of::<NetworkMetrics>() {
                let net_metrics: NetworkMetrics =
                    unsafe { std::ptr::read_unaligned(vec_bytes.as_ptr() as *const _) };
                let tgid = net_metrics.tgid;
                let comm = String::from_utf8_lossy(&net_metrics.comm);
                let ts_us = net_metrics.ts_us;
                let sk_drop_count = net_metrics.sk_drops;
                let sk_err = net_metrics.sk_err;
                let sk_err_soft = net_metrics.sk_err_soft;
                let sk_backlog_len = net_metrics.sk_backlog_len;
                let sk_write_memory_queued = net_metrics.sk_write_memory_queued;
                let sk_ack_backlog = net_metrics.sk_ack_backlog;
                let sk_receive_buffer_size = net_metrics.sk_receive_buffer_size;

                info!(
                    "tgid: {}, comm: {}, ts_us: {}, sk_drops: {}, sk_err: {}, sk_err_soft: {}, sk_backlog_len: {}, sk_write_memory_queued: {}, sk_ack_backlog: {}, sk_receive_buffer_size: {}",
                    tgid,
                    comm,
                    ts_us,
                    sk_drop_count,
                    sk_err,
                    sk_err_soft,
                    sk_backlog_len,
                    sk_write_memory_queued,
                    sk_ack_backlog,
                    sk_receive_buffer_size
                );
            }
        }
    }
    #[cfg(feature = "monitoring-structs")]
    pub async fn read_timestamp_metrics(buffers: &mut [BytesMut], tot_events: i32, offset: i32) {
        for i in offset..tot_events {
            let vec_bytes = &buffers[i as usize];
            if vec_bytes.len() < std::mem::size_of::<TimeStampMetrics>() {
                error!(
                    "Corrupted Network Metrics data. Raw data: {}. Readed {} bytes expected {} bytes",
                    vec_bytes
                        .iter()
                        .map(|b| format!("{:02x}", b))
                        .collect::<Vec<_>>()
                        .join(" "),
                    vec_bytes.len(),
                    std::mem::size_of::<TimeStampMetrics>()
                );
                continue;
            }
            if vec_bytes.len() >= std::mem::size_of::<TimeStampMetrics>() {
                let time_stamp_event: TimeStampMetrics =
                    unsafe { std::ptr::read_unaligned(vec_bytes.as_ptr() as *const _) };
                let delta_us = time_stamp_event.delta_us;
                let ts_us = time_stamp_event.ts_us;
                let tgid = time_stamp_event.tgid;
                let comm = String::from_utf8_lossy(&time_stamp_event.comm);
                let lport = time_stamp_event.lport;
                let dport_be = time_stamp_event.dport_be;
                let af = time_stamp_event.af;
                info!(
                    "TimeStampEvent - delta_us: {}, ts_us: {}, tgid: {}, comm: {}, lport: {}, dport_be: {}, af: {}",
                    delta_us, ts_us, tgid, comm, lport, dport_be, af
                );
            }
        }
    }
}

// docs: read buffer function:
// template function that take a mut perf_event_array_buffer of type T and a mutable buffer of Vec<BytesMut>
#[cfg(feature = "buffer-reader")]
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
                            #[cfg(feature = "network-structs")]
                            BufferType::PacketLog => {
                                BufferType::read_packet_log(&mut buffers, tot_events, offset).await
                            }
                            #[cfg(feature = "network-structs")]
                            BufferType::TcpPacketRegistry => {
                                BufferType::read_tcp_registry_log(&mut buffers, tot_events, offset)
                                    .await
                            }
                            #[cfg(feature = "network-structs")]
                            BufferType::VethLog => {
                                BufferType::read_and_handle_veth_log(
                                    &mut buffers,
                                    tot_events,
                                    offset,
                                )
                                .await
                            }
                            #[cfg(feature = "monitoring-structs")]
                            BufferType::NetworkMetrics => {
                                BufferType::read_network_metrics(&mut buffers, tot_events, offset)
                                    .await
                            }
                            #[cfg(feature = "monitoring-structs")]
                            BufferType::TimeStampMetrics => {
                                BufferType::read_timestamp_metrics(&mut buffers, tot_events, offset)
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

#[cfg(feature = "buffer-reader")]
pub enum BufferSize {
    #[cfg(feature = "network-structs")]
    ClassifierNetEvents,
    #[cfg(feature = "network-structs")]
    VethEvents,
    #[cfg(feature = "network-structs")]
    TcpEvents,
    #[cfg(feature = "monitoring-structs")]
    NetworkMetricsEvents,
    #[cfg(feature = "monitoring-structs")]
    TimeMetricsEvents,
}
#[cfg(feature = "buffer-reader")]
impl BufferSize {
    pub fn get_size(&self) -> usize {
        match self {
            #[cfg(feature = "network-structs")]
            BufferSize::ClassifierNetEvents => std::mem::size_of::<PacketLog>(),
            #[cfg(feature = "network-structs")]
            BufferSize::VethEvents => std::mem::size_of::<VethLog>(),
            #[cfg(feature = "network-structs")]
            BufferSize::TcpEvents => std::mem::size_of::<TcpPacketRegistry>(),
            #[cfg(feature = "monitoring-structs")]
            BufferSize::NetworkMetricsEvents => std::mem::size_of::<NetworkMetrics>(),
            #[cfg(feature = "monitoring-structs")]
            BufferSize::TimeMetricsEvents => std::mem::size_of::<TimeStampMetrics>(),
        }
    }
    pub fn set_buffer(&self) -> Vec<BytesMut> {
        // iter returns and iterator of cpu ids,
        // we need only the total number of cpus to set the buffer size so we use .len() to get
        // the count of total cpus and then we allocate a buffer for each cpu with a capacity
        // based on the structure size * a factor to have a bigger buffer to avoid overflows and lost events

        // Old buffers where 1024 bytes long. Now we set different buffer size based on
        // the frequence of the events.
        // ClassifierNetEvents are triggered by the TC classifier program, events has high frequency
        // VethEvents are triggered by the creation and deletion of veth interfaces, events has small frequency compared to classifier events
        // TcpEvents are triggered by TCP events and connections. Events has similar frequency to ClassifierNetEvents.

        let tot_cpu = online_cpus().iter().len(); // total number of cpus

        // TODO: finish to do all the calculations for the buffer sizes
        match self {
            #[cfg(feature = "network-structs")]
            BufferSize::ClassifierNetEvents => {
                let capacity = self.get_size() * 200;
                return vec![BytesMut::with_capacity(capacity); tot_cpu];
            }
            #[cfg(feature = "network-structs")]
            BufferSize::VethEvents => {
                let capacity = self.get_size() * 100; // Allocates 4Kb of memory for the buffers
                return vec![BytesMut::with_capacity(capacity); tot_cpu];
            }
            #[cfg(feature = "network-structs")]
            BufferSize::TcpEvents => {
                let capacity = self.get_size() * 200;
                return vec![BytesMut::with_capacity(capacity); tot_cpu];
            }
            #[cfg(feature = "monitoring-structs")]
            BufferSize::NetworkMetricsEvents => {
                let capacity = self.get_size() * 1024;
                return vec![BytesMut::with_capacity(capacity); tot_cpu];
            }
            #[cfg(feature = "monitoring-structs")]
            BufferSize::TimeMetricsEvents => {
                let capacity = self.get_size() * 1024;
                return vec![BytesMut::with_capacity(capacity); tot_cpu];
            }
        }
    }
}

#[cfg(feature = "buffer-reader")]
pub fn fill_buffers(
    //buf: PerfEventArrayBuffer<MapData>,
    mut vec_of_buffers: Vec<PerfEventArrayBuffer<MapData>>,
    //buffers: Vec<BytesMut>,
    mut events_array: PerfEventArray<MapData>,
) -> Vec<PerfEventArrayBuffer<MapData>> {
    for cpu_id in online_cpus()
        .map_err(|e| anyhow::anyhow!("Error {:?}", e))
        .unwrap()
    {
        let buf = events_array
            .open(cpu_id, None)
            .expect("Error during the creation of net_events_buf structure");

        vec_of_buffers.push(buf);
    }
    vec_of_buffers
}
