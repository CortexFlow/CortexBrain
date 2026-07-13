//! OpenTelemetry metric instruments for eBPF perf-buffer events.
//!
//! This module centralises every [`Meter`]-backed instrument that the
//! `metrics` crate uses to observe raw eBPF events.  It provides a single
//! [`Metrics`] handle that is cheap to [`Arc`]-clone and safe to use from
//! multiple asynchronous tasks concurrently.
//!
//!  - An [`Arc<Metrics>`] is moved into each Tokio
//!   task that reads a perf buffer.  All instrument operations are lock-free.
//!  - Every observation is tagged with `tgid` and `comm`
//!   extracted from the eBPF struct, allowing downstream collectors to group
//!   telemetry by process.

use crate::buffer_type::{
    CpuFrequency, CpuIdle, MemAlloc, PacketLossMetrics, SchedStatRuntime, SchedStatWait,
    TimeStampMetrics,
};
use crate::semantic::Semantic;
use opentelemetry::KeyValue;
use opentelemetry::metrics::{Counter, Gauge, Histogram, Meter};
pub struct Metrics {
    /// Total number of eBPF events processed across all perf buffers.
    pub events_total: Counter<u64>,

    /// Total number of network-related events produced by the `net_metrics`
    /// eBPF map.
    pub socket_events_total: Counter<u64>,

    /// Observed socket drop count (`sk_drops`) from the kernel sock struct.
    pub sk_drops: Gauge<i64>,

    /// Observed socket error count (`sk_err`) from the kernel sock struct.
    pub sk_err: Gauge<i64>,

    /// Histogram of `delta_us` values supplied by the `time_stamp_events`
    /// perf buffer.
    pub tcp_latency_us: Histogram<u64>,

    /// Cpu bytes alloc total events
    pub cpu_bytes_alloc_events_total: Counter<u64>,

    /// Cpu bytes allocation
    pub cpu_bytes_alloc: Gauge<i64>,

    /// Total number of memory allocation (mmap) events processed.
    pub mem_alloc_events_total: Counter<u64>,

    /// Observed bytes requested via mmap syscalls.
    pub enter_mem_alloc: Gauge<i64>,

    /// Observed scheduler wait time in nanoseconds (sched_stat_wait).
    pub sched_stat_wait: Gauge<i64>,

    /// Observed scheduler runtime in nanoseconds (sched_stat_runtime).
    pub sched_stat_runtime: Gauge<i64>,

    /// Current CPU idle C-state per cpu_id, updated only on state change.
    pub cpu_idle_state: Gauge<i64>,
}

// TODO: add identity metrics with TC classifier packet counts
// TODO: introduce a metric called total_tcp_packets total_udp_packets
impl Metrics {
    /// Initialise all instruments backed by the supplied [`Meter`].
    pub fn new(meter: &Meter) -> Self {
        // total events
        let events_total = meter
            .u64_counter(Semantic::TOTAL_EVENTS.title())
            .with_description(Semantic::TOTAL_EVENTS.description())
            .build();

        // total socket events
        let socket_events_total = meter
            .u64_counter(Semantic::SOCKET_TOTAL_EVENTS.title())
            .with_description(Semantic::SOCKET_TOTAL_EVENTS.description())
            .build();

        // socket drops
        let sk_drops = meter
            .i64_gauge(Semantic::SOCKET_DROPS.title())
            .with_description(Semantic::SOCKET_DROPS.description())
            .build();

        // socket errors
        let sk_err = meter
            .i64_gauge(Semantic::SOCKET_ERRORS_COUNT.title())
            .with_description(Semantic::SOCKET_ERRORS_COUNT.description())
            .build();

        // tcp latency microseconds
        let tcp_latency_us = meter
            .u64_histogram(Semantic::LATENCY.title())
            .with_description(Semantic::LATENCY.description())
            .build();

        // tcp timestamp microseconds grouped
        //let tcp_ts_us = meter
        //    .u64_histogram("ts_us")
        //    .with_description("Distribution of timestamp values from eBPF events")
        //    .build();

        // cpu bytes alloc total events
        let cpu_bytes_alloc_events_total = meter
            .u64_counter(Semantic::PERCPU_TOTAL_EVENTS.title())
            .with_description(Semantic::PERCPU_TOTAL_EVENTS.description())
            .with_unit("n")
            .build();

        // cpu bytes allocation
        let cpu_bytes_alloc = meter
            .i64_gauge(Semantic::PERCPU_BYTES_ALLOCATED.title())
            .with_description(Semantic::PERCPU_BYTES_ALLOCATED.description())
            .with_unit("bytes")
            .build();

        // memory allocation (mmap) events total
        let mem_alloc_events_total = meter
            .u64_counter(Semantic::TOTAL_MEMORY_ALLOCATION_EVENTS.title())
            .with_description(Semantic::TOTAL_MEMORY_ALLOCATION_EVENTS.description())
            .with_unit("n")
            .build();

        // bytes requested via mmap syscalls
        let enter_mem_alloc = meter
            .i64_gauge(Semantic::REQUESTED_MEMORY_BYTES.title())
            .with_description(Semantic::REQUESTED_MEMORY_BYTES.description())
            .with_unit("bytes")
            .build();

        // scheduler wait time in nanoseconds
        let sched_stat_wait = meter
            .i64_gauge(Semantic::SCHEDULER_WAIT_TIME.title())
            .with_description(Semantic::SCHEDULER_WAIT_TIME.description())
            .with_unit("ns")
            .build();

        // scheduler runtime in nanoseconds
        let sched_stat_runtime = meter
            .i64_gauge(Semantic::SCHEDULER_RUNTIME.title())
            .with_description(Semantic::SCHEDULER_RUNTIME.description())
            .with_unit("ns")
            .build();

        // current CPU idle C-state per cpu_id
        let cpu_idle_state = meter
            .i64_gauge(Semantic::CPU_IDLE_STATE.title())
            .with_description(Semantic::CPU_IDLE_STATE.description())
            .build();
        Self {
            events_total,
            socket_events_total,
            sk_drops,
            sk_err,
            tcp_latency_us,
            //tcp_ts_us,
            cpu_bytes_alloc,
            cpu_bytes_alloc_events_total,
            mem_alloc_events_total,
            enter_mem_alloc,
            sched_stat_wait,
            sched_stat_runtime,
            cpu_idle_state,
        }
    }

    /// Record a single [`NetworkMetrics`] event.
    ///
    /// Increments `events_total` and `packets_total`, records `sk_drops` and
    /// `sk_err` as gauges, and observes `ts_us` in the timestamp histogram.
    ///
    /// Every observation carries:
    ///
    /// -`tgid` – task group ID.
    /// - `comm` – command name (null-terminated bytes converted to a UTF-8
    ///   string and trimmed).
    pub fn record_packet_loss_metrics(&self, m: &PacketLossMetrics) {
        let comm = String::from_utf8_lossy(&m.comm);
        let comm_trimmed = comm.trim_end_matches('\0').to_string();
        let attrs = &[
            KeyValue::new("tgid", m.tgid as i64),
            KeyValue::new("comm", comm_trimmed),
        ];

        self.events_total.add(1, attrs);
        self.socket_events_total.add(1, attrs);
        self.sk_drops.record(m.sk_drops as i64, attrs);
        //self.sk_err.record(m.sk_err as i64, attrs);
        //self.tcp_ts_us.record(m.tcp_ts_us, attrs);
    }

    /// Record a single [`TimeStampMetrics`] event.
    ///
    /// Increments `events_total`, and records `delta_us` and `ts_us` in their
    /// respective histograms.
    ///
    /// Every observation carries `tgid` and `comm` (see
    /// [`record_network_metrics`]).
    pub fn record_timestamp_metrics(&self, m: &TimeStampMetrics) {
        let comm = String::from_utf8_lossy(&m.comm);
        let comm_trimmed = comm.trim_end_matches('\0').to_string();
        let attrs = &[
            KeyValue::new("tgid", m.tgid as i64),
            KeyValue::new("comm", comm_trimmed),
        ];

        self.events_total.add(1, attrs);
        self.tcp_latency_us.record(m.delta_us, attrs);
        //self.tcp_ts_us.record(m.ts_us, attrs);
    }

    pub fn record_cpu_bytes_alloc(&self, m: &CpuFrequency) {
        let bytes_allocated = m.bytes_alloc;
        let tgid = m.pid; // percpu tracepoints expose TGID in common_pid
        let comm = String::from_utf8_lossy(&m.command);
        let command = comm.trim_end_matches('\0').to_string();
        let attrs = &[
            KeyValue::new("tgid", tgid as i64),
            KeyValue::new("command", command),
        ];
        self.cpu_bytes_alloc_events_total.add(1, attrs);
        self.cpu_bytes_alloc.record(bytes_allocated as i64, attrs);
    }

    /// Record a single [`MemAlloc`] event (mmap syscall).
    ///
    /// Increments the dedicated `mem_alloc_events_total` counter and records
    /// the requested length in the `enter_mem_alloc` gauge.  The shared
    /// `events_total` counter is intentionally **not** incremented for these
    /// events.
    pub fn record_enter_mem_alloc(&self, m: &MemAlloc) {
        let comm = String::from_utf8_lossy(&m.command);
        let command = comm.trim_end_matches('\0').to_string();
        let attrs = &[
            KeyValue::new("tgid", m.tgid as i64),
            KeyValue::new("command", command),
        ];

        self.events_total.add(1, attrs);
        self.mem_alloc_events_total.add(1, attrs);
        self.enter_mem_alloc.record(m.length as i64, attrs);
    }

    /// Record a single [`SchedStatWait`] event.
    ///
    /// Records `delay` in the `sched_stat_wait` gauge.  No shared or dedicated
    /// counter is incremented, as requested.
    pub fn record_sched_stat_wait(&self, m: &SchedStatWait) {
        let comm = String::from_utf8_lossy(&m.command);
        let command = comm.trim_end_matches('\0').to_string();
        let attrs = &[
            KeyValue::new("tgid", m.tgid as i64),
            KeyValue::new("command", command),
        ];

        self.events_total.add(1, attrs);
        self.sched_stat_wait.record(m.delay as i64, attrs);
    }

    /// Record a single [`SchedStatRuntime`] event.
    ///
    /// Records `runtime` in the `sched_stat_runtime` gauge.  No shared or
    /// dedicated counter is incremented, as requested.
    pub fn record_sched_stat_runtime(&self, m: &SchedStatRuntime) {
        let comm = String::from_utf8_lossy(&m.command);
        let command = comm.trim_end_matches('\0').to_string();
        let attrs = &[
            KeyValue::new("tgid", m.tgid as i64),
            KeyValue::new("command", command),
        ];

        self.events_total.add(1, attrs);
        self.sched_stat_runtime.record(m.runtime as i64, attrs);
    }

    /// Record a single [`CpuIdle`] event.
    ///
    /// Updates `cpu_idle_state` gauge to the latest C-state for the given
    /// `cpu_id`. Events are only emitted by eBPF when the state changes.
    pub fn record_cpu_idle(&self, m: &CpuIdle) {
        let attrs = &[KeyValue::new("cpu_id", m.cpu_id as i64)];

        self.events_total.add(1, attrs);
        self.cpu_idle_state.record(m.state as i64, attrs);
    }
}
