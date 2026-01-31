use bytemuck_derive::Zeroable;

/*
 * Structure PacketLog
 * This structure is used to store the packet information
 */
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
unsafe impl aya::Pod for PacketLog {}

/*
 * Connection Array that contains the hash_id associated with an active connection
 */
//#[repr(C)]
//#[derive(Clone, Copy, Zeroable)]
//pub struct ConnArray {
//    pub src_ip: u32,
//    pub dst_ip: u32,
//    pub src_port: u16,
//    pub dst_port: u16,
//    pub proto: u8,
//}

//unsafe impl aya::Pod for ConnArray {}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct VethLog {
    pub name: [u8; 16],      // 16 bytes: veth interface name
    pub state: u64,           // 8 bytes: state variable (unsigned long in kernel)
    pub dev_addr: [u32; 8],   // 32 bytes: device address
    pub event_type: u8,       // 1 byte: 1 for veth creation, 2 for veth destruction
    pub netns: u32,           // 4 bytes: network namespace inode number
    pub pid: u32,             // 4 bytes: PID that triggered the event
}


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
