use aya_ebpf::{
    macros::map,
    maps::{LruPerCpuHashMap, PerfEventArray},
};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PacketLog {
    pub proto: u8,
    pub src_ip: u32,
    pub src_port: u16,
    pub dst_ip: u32,
    pub dst_port: u16,
    pub pid: u32
}

// This structure is only for active connections
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ConnArray {
    pub src_ip: u32,
    pub dst_ip: u32,
    pub src_port: u16,
    pub dst_port: u16,
    pub proto: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct VethLog {
    pub name: [u8; 16],
    pub state: u64, //state var type: long unsigned int
    pub dev_addr: [u32; 8],
    pub event_type: u8, //i choose 1 for veth creation or 2 for veth destruction
    pub netns: u32,
    pub pid: u32

}

#[map(name = "EventsMap", pinning = "by_name")]
pub static mut EVENTS: PerfEventArray<PacketLog> = PerfEventArray::new(0);

//TODO: ConnectionMap needs a rework after implementing issue #105
#[map(name = "ConnectionMap")]
pub static mut ACTIVE_CONNECTIONS: LruPerCpuHashMap<u16, ConnArray> =
    LruPerCpuHashMap::with_max_entries(65536, 0);

#[map(name = "ConnectionTrackerMap")]
pub static mut CONNTRACKER: LruPerCpuHashMap<ConnArray, u8> =
    LruPerCpuHashMap::with_max_entries(65536, 0);

#[map(name = "veth_identity_map")]
pub static mut VETH_EVENTS: PerfEventArray<VethLog> = PerfEventArray::new(0);
