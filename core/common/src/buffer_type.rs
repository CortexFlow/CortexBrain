//! eBPF data structures and buffer size definitions.
//!
//! This module contains:
//! - C-compatible structs emitted by the eBPF programs (`PacketLossMetrics`, `SchedStatWait`, etc.).
//! - [`BufferSize`] for pre-allocating per-CPU byte buffers.
//! - [`IpProtocols`] for L4 protocol reconstruction.
//!
//! The consumer logic has been moved to [`crate::consumer`].

use aya::maps::perf::PerfEventArrayBuffer;
#[cfg(feature = "buffer-reader")]
use aya::maps::{MapData, PerfEventArray};
use aya::util::online_cpus;
use bytemuck_derive::Zeroable;
use bytes::BytesMut;
use std::net::Ipv4Addr;

///
/// IpProtocols enum to reconstruct the packet protocol based on the
/// IPV4 Header Protocol code
///

#[derive(Debug)]
#[repr(u8)]
pub enum IpProtocols {
    ICMP = 1,
    TCP = 6,
    UDP = 17,
}

///
/// TryFrom Trait implementation for IpProtocols enum
/// This is used to reconstruct the packet protocol based on the
/// IPV4 Header Protocol code
///

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

///
/// Structure PacketLog
/// This structure is used to store the packet information
///
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
pub const TASK_COMM_LEN: usize = 16;
#[cfg(feature = "monitoring-structs")]
#[repr(C, packed)]
#[derive(Clone, Copy, Zeroable)]
pub struct PacketLossMetrics {
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
    pub cgroup_id: u64,
}
#[cfg(feature = "monitoring-structs")]
unsafe impl aya::Pod for PacketLossMetrics {}

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
    pub cgroup_id: u64,
}
#[cfg(feature = "monitoring-structs")]
unsafe impl aya::Pod for TimeStampMetrics {}
#[cfg(feature = "monitoring-structs")]
#[repr(C, packed)]
#[derive(Clone, Copy, Zeroable)]
pub struct CpuFrequency {
    pub bytes_alloc: u32,
    pub pid: u32,
    pub command: [u8; 16],
}
#[cfg(feature = "monitoring-structs")]
unsafe impl aya::Pod for CpuFrequency {}

#[cfg(feature = "monitoring-structs")]
#[repr(C, packed)]
#[derive(Clone, Copy, Zeroable)]
pub struct MemAlloc {
    pub tgid: u32,
    pub length: u64,
    pub addr: u64,
    pub command: [u8; TASK_COMM_LEN],
    pub cgroup_id: u64,
}
#[cfg(feature = "monitoring-structs")]
unsafe impl aya::Pod for MemAlloc {}

#[cfg(feature = "monitoring-structs")]
#[repr(C, packed)]
#[derive(Clone, Copy, Zeroable)]
pub struct SchedStatWait {
    pub tgid: u32,
    pub delay: u64,
    pub command: [u8; TASK_COMM_LEN],
    pub cgroup_id: u64,
}
#[cfg(feature = "monitoring-structs")]
unsafe impl aya::Pod for SchedStatWait {}

#[cfg(feature = "monitoring-structs")]
#[repr(C, packed)]
#[derive(Clone, Copy, Zeroable)]
pub struct SchedStatRuntime {
    pub tgid: u32,
    pub runtime: u64,
    pub command: [u8; TASK_COMM_LEN],
    pub cgroup_id: u64,
}
#[cfg(feature = "monitoring-structs")]
unsafe impl aya::Pod for SchedStatRuntime {}

#[cfg(feature = "monitoring-structs")]
#[repr(C, packed)]
#[derive(Clone, Copy, Zeroable)]
pub struct CpuIdle {
    pub cpu_id: u32,
    pub state: u32,
}
#[cfg(feature = "monitoring-structs")]
unsafe impl aya::Pod for CpuIdle {}

#[cfg(feature = "monitoring-structs")]
#[repr(C, packed)]
#[derive(Copy, Clone, Zeroable)]
pub struct SslEvent {
    pub tgid: u32,
    pub comm: [u8; TASK_COMM_LEN],
    pub ts_us: u64,
    pub direction: u8,  // 0 = read, 1 = write
    pub size: i32,      // return value (bytes transferred or <0 on error)
    pub requested: i32, // num argument passed to SSL_read/SSL_write
    pub cgroup_id: u64,
}
#[cfg(feature = "monitoring-structs")]
unsafe impl aya::Pod for SslEvent {}

/// Perform a byte swap from little-endian to big-endian.
///
/// Used to reconstruct the correct IPv4 address from the u32 representation.
/// Takes a `u32` address in big-endian format and returns an [`Ipv4Addr`] with reversed octets.
#[inline(always)]
pub fn reverse_be_addr(addr: u32) -> Ipv4Addr {
    let octects = addr.to_be_bytes();
    let [a, b, c, d] = [octects[3], octects[2], octects[1], octects[0]];
    Ipv4Addr::new(a, b, c, d)
}

/// Buffer size presets for per-CPU perf-buffer allocation.
///
/// Each variant carries a multiplier that determines how many struct-sized
/// slots are pre-allocated per CPU in [`BufferSize::set_buffer`].
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
    #[cfg(feature = "monitoring-structs")]
    CpuFrequency,
    #[cfg(feature = "monitoring-structs")]
    MemAlloc,
    #[cfg(feature = "monitoring-structs")]
    SchedStatWait,
    #[cfg(feature = "monitoring-structs")]
    SchedStatRuntime,
    #[cfg(feature = "monitoring-structs")]
    CpuIdle,
    #[cfg(feature = "monitoring-structs")]
    SslEvents,
}

#[cfg(feature = "buffer-reader")]
impl BufferSize {
    /// Return the size in bytes of the struct associated with this variant.
    pub fn get_size(&self) -> usize {
        match self {
            #[cfg(feature = "network-structs")]
            BufferSize::ClassifierNetEvents => std::mem::size_of::<PacketLog>(),
            #[cfg(feature = "network-structs")]
            BufferSize::VethEvents => std::mem::size_of::<VethLog>(),
            #[cfg(feature = "network-structs")]
            BufferSize::TcpEvents => std::mem::size_of::<TcpPacketRegistry>(),
            #[cfg(feature = "monitoring-structs")]
            BufferSize::NetworkMetricsEvents => std::mem::size_of::<PacketLossMetrics>(),
            #[cfg(feature = "monitoring-structs")]
            BufferSize::TimeMetricsEvents => std::mem::size_of::<TimeStampMetrics>(),
            #[cfg(feature = "monitoring-structs")]
            BufferSize::CpuFrequency => std::mem::size_of::<CpuFrequency>(),
            #[cfg(feature = "monitoring-structs")]
            BufferSize::MemAlloc => std::mem::size_of::<MemAlloc>(),
            #[cfg(feature = "monitoring-structs")]
            BufferSize::SchedStatWait => std::mem::size_of::<SchedStatWait>(),
            #[cfg(feature = "monitoring-structs")]
            BufferSize::SchedStatRuntime => std::mem::size_of::<SchedStatRuntime>(),
            #[cfg(feature = "monitoring-structs")]
            BufferSize::CpuIdle => std::mem::size_of::<CpuIdle>(),
            #[cfg(feature = "monitoring-structs")]
            BufferSize::SslEvents => std::mem::size_of::<SslEvent>(),
        }
    }

    /// Allocate one `BytesMut` per CPU with capacity tuned to the event type.
    pub fn set_buffer(&self) -> Vec<BytesMut> {
        use aya::util::online_cpus;

        let tot_cpu = online_cpus().iter().len();

        // TODO: finish buffer size calculations
        match self {
            #[cfg(feature = "network-structs")]
            BufferSize::ClassifierNetEvents => {
                let capacity = self.get_size() * 200;
                return vec![BytesMut::with_capacity(capacity); tot_cpu];
            }
            #[cfg(feature = "network-structs")]
            BufferSize::VethEvents => {
                let capacity = self.get_size() * 100;
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
            #[cfg(feature = "monitoring-structs")]
            BufferSize::CpuFrequency => {
                let capacity = self.get_size() * 1024;
                return vec![BytesMut::with_capacity(capacity); tot_cpu];
            }
            #[cfg(feature = "monitoring-structs")]
            BufferSize::MemAlloc => {
                let capacity = self.get_size() * 1024;
                return vec![BytesMut::with_capacity(capacity); tot_cpu];
            }
            #[cfg(feature = "monitoring-structs")]
            BufferSize::SchedStatWait => {
                let capacity = self.get_size() * 1024;
                return vec![BytesMut::with_capacity(capacity); tot_cpu];
            }
            #[cfg(feature = "monitoring-structs")]
            BufferSize::SchedStatRuntime => {
                let capacity = self.get_size() * 1024;
                return vec![BytesMut::with_capacity(capacity); tot_cpu];
            }
            #[cfg(feature = "monitoring-structs")]
            BufferSize::CpuIdle => {
                let capacity = self.get_size() * 1024;
                return vec![BytesMut::with_capacity(capacity); tot_cpu];
            }
            #[cfg(feature = "monitoring-structs")]
            BufferSize::SslEvents => {
                let capacity = self.get_size() * 1024;
                return vec![BytesMut::with_capacity(capacity); tot_cpu];
            }
        }
    }
}

/// Open a [`PerfEventArrayBuffer`] for every online CPU and append them to `vec_of_buffers`.
#[cfg(feature = "buffer-reader")]
pub fn fill_buffers(
    mut vec_of_buffers: Vec<PerfEventArrayBuffer<MapData>>,
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
