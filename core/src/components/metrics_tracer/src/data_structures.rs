use aya_ebpf::{macros::map, maps::{LruPerCpuHashMap, PerfEventArray}};


pub struct NetworkMetrics {
    pub sk_err: i32,                // Offset 284
    pub sk_err_soft: i32,           // Offset 600
    pub sk_backlog_len: i32,        // Offset 196
    pub sk_write_memory_queued: i32,// Offset 376
    pub sk_receive_buffer_size: i32,// Offset 244
    pub sk_ack_backlog: u32,        // Offset 604
    pub sk_drops: i32,              // Offset 136
}

#[map(name = "net_metrics")]
pub static NET_METRICS: PerfEventArray<NetworkMetrics> = PerfEventArray::new(0);
