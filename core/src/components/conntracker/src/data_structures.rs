use aya_ebpf::{
    macros::map,
    maps::{LruPerCpuHashMap, PerfEventArray,HashMap},
};

// docs:
// PacketLog structure used to track an incoming network packet
// 
// proto: packet protol (ex. TCP,UDP,ICMP)
// src_ip: source address ip 
// src_port: source address port
// dst_ip: destination ip
// dst_port: destination port
// pid: kernel process ID
//


#[repr(C)]
#[derive(Clone, Copy)]
pub struct PacketLog {
    pub proto: u8,
    pub src_ip: u32,
    pub src_port: u16,
    pub dst_ip: u32,
    pub dst_port: u16,
    pub pid: u32,
}

// This structure is only for active connections (TODO: investigate if this is really useful)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ConnArray {
    pub src_ip: u32,
    pub dst_ip: u32,
    pub src_port: u16,
    pub dst_port: u16,
    pub proto: u8,
}


// docs:
// VethLog structure used to track virtual ethernet interfaces creation and deletion
// 
// name: veth name
// state: socket state 
// dev_addr: veth device addresses
// event_type: creation or deletion
// netns: veth network namespace
// pid: kernel process ID
//

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct VethLog {
    pub name: [u8; 16],
    pub state: u64, // state var type: long unsigned int
    pub dev_addr: [u32; 8],
    pub event_type: u8, // i choose 1 for veth creation or 2 for veth destruction
    pub netns: u32,
    pub pid: u32

}

// docs:
//
// BPF maps used in the conntracker programs 
// 
// VETH_EVENTS: PerfEventArray used in the veth_tracer functions (veth_tracer.rs module)
//
// BLOCKLIST: an hashmap used to block addresses -----> TODO: key and values are the same for semplicity but we need to 
//            investigate the possibility to save the service name or the timestamp registered when the command was executed or a simple int index
//


#[map(name = "EventsMap", pinning = "by_name")]
pub static mut EVENTS: PerfEventArray<PacketLog> = PerfEventArray::new(0);

// FIXME: this might be useless 
#[map(name = "ConnectionMap")]
pub static mut ACTIVE_CONNECTIONS: LruPerCpuHashMap<u16, ConnArray> =
    LruPerCpuHashMap::with_max_entries(65536, 0);

// FIXME: this might be useless
#[map(name = "ConnectionTrackerMap")]
pub static mut CONNTRACKER: LruPerCpuHashMap<ConnArray, u8> =
    LruPerCpuHashMap::with_max_entries(65536, 0);

#[map(name = "veth_identity_map")]
pub static mut VETH_EVENTS: PerfEventArray<VethLog> = PerfEventArray::new(0);

#[map(name = "Blocklist")]
pub static mut BLOCKLIST: HashMap<[u8;4], [u8;4]> = HashMap::<[u8;4], [u8;4]>::with_max_entries(1024, 0);
//here i need to pass an address like this: [135,171,168,192]