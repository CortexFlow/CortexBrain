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
    CpuFrequency, CpuIdle, MemAlloc, NetworkMetrics, SchedStatRuntime, SchedStatWait,
    TimeStampMetrics,
};
use opentelemetry::KeyValue;
use opentelemetry::metrics::{Counter, Gauge, Histogram, Meter};
pub struct Metrics {
    /// Total number of eBPF events processed across all perf buffers.
    pub events_total: Counter<u64>,

    /// Total number of network-related events produced by the `net_metrics`
    /// eBPF map.
    pub packets_total: Counter<u64>,

    /// Observed socket drop count (`sk_drops`) from the kernel sock struct.
    pub sk_drops: Gauge<i64>,

    /// Observed socket error count (`sk_err`) from the kernel sock struct.
    pub sk_err: Gauge<i64>,

    /// Histogram of `delta_us` values supplied by the `time_stamp_events`
    /// perf buffer.
    pub delta_us: Histogram<u64>,

    /// Histogram of `ts_us` values seen in both `net_metrics` and
    /// `time_stamp_events`.
    pub ts_us: Histogram<u64>,

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

impl Metrics {
    /// Initialise all instruments backed by the supplied [`Meter`].
    pub fn new(meter: &Meter) -> Self {
        // total events
        let events_total = meter
            .u64_counter("events_total")
            .with_description("Total number of eBPF events processed")
            .build();

        // total packets
        let packets_total = meter
            .u64_counter("packets_total")
            .with_description("Total number of network events processed")
            .build();

        // socket drops
        let sk_drops = meter
            .i64_gauge("sk_drops")
            .with_description("Socket drop count per event")
            .build();

        // socket errors
        let sk_err = meter
            .i64_gauge("sk_err")
            .with_description("Socket error count per event")
            .build();

        // delta microseconds
        let delta_us = meter
            .u64_histogram("delta_us")
            .with_description("Distribution of delta_us values from timestamp events")
            .build();

        // timestamp microseconds grouped
        let ts_us = meter
            .u64_histogram("ts_us")
            .with_description("Distribution of timestamp values from eBPF events")
            .build();

        // cpu bytes alloc total events
        let cpu_bytes_alloc_events_total = meter
            .u64_counter("bytes_alloc_events_total")
            .with_description("Total bytes_alloc events occuring in the CPU")
            .build();

        // cpu bytes allocation
        let cpu_bytes_alloc = meter
            .i64_gauge("cpu_bytes_alloc")
            .with_description("Cpu bytes allocation per event")
            .build();

        // memory allocation (mmap) events total
        let mem_alloc_events_total = meter
            .u64_counter("mem_alloc_events_total")
            .with_description("Total number of memory allocation (mmap) events processed")
            .build();

        // bytes requested via mmap syscalls
        let enter_mem_alloc = meter
            .i64_gauge("enter_mem_alloc")
            .with_description("Bytes requested via mmap syscalls")
            .build();

        // scheduler wait time in nanoseconds
        let sched_stat_wait = meter
            .i64_gauge("sched_stat_wait")
            .with_description("Scheduler wait time in nanoseconds from sched_stat_wait")
            .build();

        // scheduler runtime in nanoseconds
        let sched_stat_runtime = meter
            .i64_gauge("sched_stat_runtime")
            .with_description("Scheduler runtime in nanoseconds from sched_stat_runtime")
            .build();

        // current CPU idle C-state per cpu_id
        let cpu_idle_state = meter
            .i64_gauge("cpu_idle_state")
            .with_description("Current CPU idle C-state per cpu_id, updated only on state change")
            .build();

        Self {
            events_total,
            packets_total,
            sk_drops,
            sk_err,
            delta_us,
            ts_us,
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
    pub fn record_network_metrics(&self, m: &NetworkMetrics) {
        let comm = String::from_utf8_lossy(&m.comm);
        let comm_trimmed = comm.trim_end_matches('\0').to_string();
        let attrs = &[
            KeyValue::new("tgid", m.tgid as i64),
            KeyValue::new("comm", comm_trimmed),
        ];

        self.events_total.add(1, attrs);
        self.packets_total.add(1, attrs);
        self.sk_drops.record(m.sk_drops as i64, attrs);
        self.sk_err.record(m.sk_err as i64, attrs);
        self.ts_us.record(m.ts_us, attrs);
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
        self.delta_us.record(m.delta_us, attrs);
        self.ts_us.record(m.ts_us, attrs);
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

        self.sched_stat_runtime.record(m.runtime as i64, attrs);
    }

    /// Record a single [`CpuIdle`] event.
    ///
    /// Updates `cpu_idle_state` gauge to the latest C-state for the given
    /// `cpu_id`. Events are only emitted by eBPF when the state changes.
    pub fn record_cpu_idle(&self, m: &CpuIdle) {
        let attrs = &[KeyValue::new("cpu_id", m.cpu_id as i64)];

        self.cpu_idle_state.record(m.state as i64, attrs);
    }
}
