/// semantic conventions

pub enum Semantic {
    TotalEvents,
    SocketTotalEvents,
    SocketDrops,
    SocketErrorsCount,
    Latency,
    PerCpuTotalEvents,
    PerCpuBytesAllocated,
    SchedulerRuntime,
    SchedulerRuntimeDistribution,
    SchedulerWaitTime,
    SchedulerWaitTimeDistribution,
    TotalMemoryAllocationEvents,
    RequestedMemoryBytes,
    CpuIdleState,
    SslReadBytes,
    SslWriteBytes,
}

impl Semantic {
    pub fn title(&self) -> &'static str {
        match self {
            Semantic::TotalEvents => "events_total",
            Semantic::SocketTotalEvents => "socket_events_total",
            Semantic::SocketDrops => "sk_drops",
            Semantic::SocketErrorsCount => "sk_err",
            Semantic::Latency => "latency_us",
            Semantic::PerCpuTotalEvents => "bytes_alloc_events_total",
            Semantic::PerCpuBytesAllocated => "cpu_bytes_alloc",
            Semantic::SchedulerRuntime => "sched_stat_runtime",
            Semantic::SchedulerRuntimeDistribution => "sched_stat_runtime_distribution",
            Semantic::SchedulerWaitTime => "sched_stat_wait",
            Semantic::SchedulerWaitTimeDistribution => "sched_stat_wait_distribution",
            Semantic::TotalMemoryAllocationEvents => "mem_alloc_events_total",
            Semantic::RequestedMemoryBytes => "enter_mem_alloc",
            Semantic::CpuIdleState => "cpu_idle_state",
            Semantic::SslReadBytes => "ssl_read_bytes",
            Semantic::SslWriteBytes => "ssl_write_bytes",
        }
    }
    pub fn description(&self) -> &'static str {
        match self {
            Semantic::TotalEvents => {
                "Total number of eBPF events processed across all perf buffers"
            }
            Semantic::SocketTotalEvents => "Total number of socket state events processed",
            Semantic::SocketDrops => "Socket drop count per event",
            Semantic::SocketErrorsCount => "Socket error count per event",
            Semantic::Latency => "Distribution of latency values from timestamp events",
            Semantic::PerCpuTotalEvents => "Total bytes_alloc events occurring in the CPU",
            Semantic::PerCpuBytesAllocated => "CPU bytes allocation per event",
            Semantic::SchedulerRuntime => {
                "Scheduler runtime in nanoseconds from sched_stat_runtime"
            }
            Semantic::SchedulerRuntimeDistribution => {
                "Distribution of scheduler runtimes in nanoseconds from sched_stat_runtime"
            }
            Semantic::SchedulerWaitTime => {
                "Scheduler wait time in nanoseconds from sched_stat_wait"
            }
            Semantic::SchedulerWaitTimeDistribution => {
                "Distribution of scheduler wait times in nanoseconds from sched_stat_wait"
            }
            Semantic::TotalMemoryAllocationEvents => {
                "Total number of memory allocation (mmap) events processed"
            }
            Semantic::RequestedMemoryBytes => "Bytes requested via mmap syscalls",
            Semantic::CpuIdleState => {
                "Current CPU idle C-state per cpu_id, updated only on state change"
            }
            Semantic::SslReadBytes => "Total bytes requested by the ssl_read function",
            Semantic::SslWriteBytes => "Total bytes requested by the ssl_write function",
        }
    }
}
