use aya_ebpf::{macros::map, maps::{LruPerCpuHashMap, HashMap, PerfEventArray}};

pub const TASK_COMM_LEN: usize = 16;


pub struct NetworkMetrics {
    pub sk_err: i32,                // Offset 284
    pub sk_err_soft: i32,           // Offset 600
    pub sk_backlog_len: i32,        // Offset 196
    pub sk_write_memory_queued: i32,// Offset 376
    pub sk_receive_buffer_size: i32,// Offset 244
    pub sk_ack_backlog: u32,        // Offset 604
    pub sk_drops: i32,              // Offset 136
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct TimeStampStartInfo {
    pub comm: [u8; TASK_COMM_LEN],
    pub ts_ns: u64,
    pub tgid: u32,
}

// Event we send to userspace when latency is computed
#[repr(C)]
#[derive(Copy, Clone)]
pub struct TimeStampEvent {
    pub delta_us: u64,
    pub ts_us: u64,
    pub tgid: u32,
    pub comm: [u8; TASK_COMM_LEN],
    pub lport: u16,
    pub dport_be: u16,
    pub af: u16, // AF_INET=2, AF_INET6=10
    pub saddr_v4: u32,
    pub daddr_v4: u32,
    pub saddr_v6: [u32; 4],
    pub daddr_v6: [u32; 4],
}

// Map: connect-start timestamp by socket pointer
#[map(name = "time_stamp_start")]
pub static mut TIME_STAMP_START: HashMap<*mut core::ffi::c_void, TimeStampStartInfo> =
    HashMap::<*mut core::ffi::c_void, TimeStampStartInfo>::with_max_entries(4096, 0);

// Perf event channel for emitting Event to userspace
#[map(name = "time_stamp_events")]
pub static mut TIME_STAMP_EVENTS: PerfEventArray<TimeStampEvent> = PerfEventArray::<TimeStampEvent>::new(0);

#[map(name = "net_metrics")]
pub static NET_METRICS: PerfEventArray<NetworkMetrics> = PerfEventArray::new(0);
