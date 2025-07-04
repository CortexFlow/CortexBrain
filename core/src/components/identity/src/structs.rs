/*
 * Structure PacketLog
 * This structure is used to store the packet information
 */
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PacketLog {
    pub proto: u8,
    pub  src_ip: u32,
    pub  src_port: u16,
    pub dst_ip: u32,
    pub dst_port: u16,
    pub event_id: u16,
}
/*
 * Connection Array that contains the hash_id associated with an active connection
 */
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ConnArray {
    pub src_ip: u32,
    pub dst_ip: u32,
    pub src_port: u16,
    pub dst_port: u16,
    pub proto: u8,
}

unsafe impl aya::Pod for ConnArray {}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct VethLog {
    pub name: [u8; 16],
    pub state: u64,
    pub dev_addr: [u32;8],
    pub event_type: u8,
}