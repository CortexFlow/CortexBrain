/// semantic conventions

pub enum Semantic {
    TOTAL_EVENTS,
    SOCKET_TOTAL_EVENTS,
    SOCKET_DROPS,
    SOCKET_ERRORS_COUNT,
    LATENCY,
    PERCPU_TOTAL_EVENTS,
    PERCPU_BYTES_ALLOCATED,
    SCHEDULER_RUNTIME,
    SCHEDULER_WAIT_TIME,
    TOTAL_MEMORY_ALLOCATION_EVENTS,
    REQUESTED_MEMORY_BYTES,
    CPU_IDLE_STATE,
}

impl Semantic {
    pub fn title(&self) -> &'static str {
        match self {
            Semantic::TOTAL_EVENTS => "events_total",
            Semantic::SOCKET_TOTAL_EVENTS => "socket_events_total",
            Semantic::SOCKET_DROPS => "sk_drops",
            Semantic::SOCKET_ERRORS_COUNT => "sk_err",
            Semantic::LATENCY => "latency_us",
            Semantic::PERCPU_TOTAL_EVENTS => "bytes_alloc_events_total",
            Semantic::PERCPU_BYTES_ALLOCATED => "cpu_bytes_alloc",
            Semantic::SCHEDULER_RUNTIME => "sched_stat_runtime",
            Semantic::SCHEDULER_WAIT_TIME => "sched_stat_wait",
            Semantic::TOTAL_MEMORY_ALLOCATION_EVENTS => "mem_alloc_events_total",
            Semantic::REQUESTED_MEMORY_BYTES => "enter_mem_alloc",
            Semantic::CPU_IDLE_STATE => "cpu_idle_state",
        }
    }
    pub fn description(&self) -> &'static str {
        match self {
            Semantic::TOTAL_EVENTS => {
                "Total number of eBPF events processed across all perf buffers"
            }
            Semantic::SOCKET_TOTAL_EVENTS => "Total number of socket state events processed",
            Semantic::SOCKET_DROPS => "Socket drop count per event",
            Semantic::SOCKET_ERRORS_COUNT => "Socket error count per event",
            Semantic::LATENCY => "Distribution of latency values from timestamp events",
            Semantic::PERCPU_TOTAL_EVENTS => "Total bytes_alloc events occuring in the CPU",
            Semantic::PERCPU_BYTES_ALLOCATED => "Cpu bytes allocation per event",
            Semantic::SCHEDULER_RUNTIME => {
                "Scheduler runtime in nanoseconds from sched_stat_runtime"
            }
            Semantic::SCHEDULER_WAIT_TIME => {
                "Scheduler wait time in nanoseconds from sched_stat_wait"
            }
            Semantic::TOTAL_MEMORY_ALLOCATION_EVENTS => {
                "Total number of memory allocation (mmap) events processed"
            }
            Semantic::REQUESTED_MEMORY_BYTES => "Bytes requested via mmap syscalls",
            Semantic::CPU_IDLE_STATE => {
                "Current CPU idle C-state per cpu_id, updated only on state change"
            }
        }
    }
}
