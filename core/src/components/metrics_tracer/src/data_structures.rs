use aya_ebpf::{
    macros::map,
    maps::{HashMap, LruPerCpuHashMap, PerfEventArray},
};

pub const TASK_COMM_LEN: usize = 16;

#[repr(C, packed)]
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
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct TimeStampStartInfo {
    pub comm: [u8; TASK_COMM_LEN],
    pub ts_ns: u64,
    pub tgid: u32,
}

/// Event we send to userspace when latency is computed
/// used to compute tcp_delta_us, tcp_ts_us metrics
#[repr(C, packed)]
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

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct CpuFrequency {
    //pub(crate) cpu_id: u32,
    //pub(crate) cpu_freq: u32,
    pub(crate) bytes_alloc: u32,
    pub(crate) tgid: u32,
    pub(crate) command: [u8; 16],
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct MemAlloc {
    pub(crate) tgid: u32,
    pub(crate) length: u64,
    pub(crate) addr: u64,
    pub(crate) command: [u8; 16],
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct SchedStatWait {
    pub(crate) tgid: u32,
    pub(crate) delay: u64,
    pub(crate) command: [u8; 16],
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct SchedStatRuntime {
    pub(crate) tgid: u32,
    pub(crate) runtime: u64,
    pub(crate) command: [u8; 16],
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct CpuIdle {
    pub(crate) cpu_id: u32,
    pub(crate) state: u32,
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SslEvent {
    pub tgid: u32,
    pub comm: [u8; TASK_COMM_LEN],
    pub ts_us: u64,
    pub direction: u8,  // 0 = read, 1 = write
    pub size: i32,      // return value (bytes transferred or <0 on error)
    pub requested: i32, // num argument passed to SSL_read/SSL_write
}

// Map: connect-start timestamp by socket pointer
#[map(name = "time_stamp_start")]
pub static mut TIME_STAMP_START: HashMap<*mut core::ffi::c_void, TimeStampStartInfo> =
    HashMap::<*mut core::ffi::c_void, TimeStampStartInfo>::with_max_entries(4096, 0);

// Perf event channel for emitting Event to userspace
#[map(name = "time_stamp_events")]
pub static mut TIME_STAMP_EVENTS: PerfEventArray<TimeStampEvent> =
    PerfEventArray::<TimeStampEvent>::new(0);

#[map(name = "net_metrics")]
pub static NET_METRICS: PerfEventArray<PacketLossMetrics> = PerfEventArray::new(0);

#[map(name = "cpu_frequency")]
pub static CPU_FREQUENCY: PerfEventArray<CpuFrequency> = PerfEventArray::new(0);

#[map(name = "mem_alloc")]
pub static MEM_ALLOC: PerfEventArray<MemAlloc> = PerfEventArray::new(0);

#[map(name = "sched_stat_wait")]
pub static SCHED_STAT_WAIT: PerfEventArray<SchedStatWait> = PerfEventArray::new(0);

#[map(name = "sched_stat_runtime")]
pub static SCHED_STAT_RUNTIME: PerfEventArray<SchedStatRuntime> = PerfEventArray::new(0);

#[map(name = "cpu_idle")]
pub static CPU_IDLE: PerfEventArray<CpuIdle> = PerfEventArray::new(0);

#[map(name = "cpu_idle_last_state")]
pub static mut CPU_IDLE_LAST_STATE: HashMap<u32, u32> =
    HashMap::<u32, u32>::with_max_entries(256, 0);

#[map(name = "ssl_ctx_map")]
pub static mut SSL_CTX_MAP: HashMap<u64, i32> =
    HashMap::<u64, i32>::with_max_entries(4096, 0);

#[map(name = "ssl_events")]
pub static SSL_EVENTS: PerfEventArray<SslEvent> = PerfEventArray::new(0);
