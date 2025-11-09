
pub const TASK_COMM_LEN: usize = 16; // linux/sched.h

#[repr(C)]
#[derive(Clone, Copy)]
pub struct NetworkMetrics {
    pub tgid: u32,
    pub comm: [u8; TASK_COMM_LEN],
    pub ts_us: u64,
    pub sk_err: i32,          // Offset 284
    pub sk_err_soft: i32,     // Offset 600
    pub sk_backlog_len: i32,  // Offset 196
    pub sk_write_memory_queued: i32,  // Offset 376
    pub sk_receive_buffer_size: i32,       // Offset 244
    pub sk_ack_backlog: u32,  // Offset 604
    pub sk_drops: i32,        // Offset 136
}

#[repr(C)]
#[derive(Clone, Copy)]
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