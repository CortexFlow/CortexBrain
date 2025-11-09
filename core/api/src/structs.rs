use bytemuck::Zeroable;
use crate::constants::TASK_COMM_LEN;


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

#[repr(C)]
#[derive(Clone, Copy, Zeroable)]
pub struct NetworkMetrics {
    pub tgid: u32,
    pub comm: [u8; TASK_COMM_LEN],
    pub ts_us: u64,
    pub sk_err: i32,
    pub sk_err_soft: i32,
    pub sk_backlog_len: u32,
    pub sk_write_memory_queued: i32,
    pub sk_receive_buffer_size: i32,
    pub sk_ack_backlog: u32,
    pub sk_drops: i32,
}
unsafe impl aya::Pod for NetworkMetrics {}

#[repr(C)]
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
unsafe impl aya::Pod for TimeStampMetrics {}