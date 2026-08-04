//! OpenTelemetry metric instruments for eBPF perf-buffer events.
//!
//! This module centralises every [`Meter`]-backed instrument that the
//! `metrics` crate uses to observe raw eBPF events.  It provides a single
//! [`Metrics`] handle that is cheap to [`Arc`]-clone and safe to use from
//! multiple asynchronous tasks concurrently.
//!
//!  - An [`Arc<Metrics>`] is moved into each Tokio
//!   task that reads a perf buffer.  All instrument operations are lock-free.
//!  - Every observation is tagged with process and container metadata
//!   extracted from the eBPF struct via [`Metadata`].

use crate::buffer_type::{
    CpuFrequency, CpuIdle, MemAlloc, PacketLossMetrics, SchedStatRuntime, SchedStatWait, SslEvent,
    TimeStampMetrics,
};
use crate::metadata::{ContainerRuntime, Metadata};
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

    /// Distribution of scheduler wait times in nanoseconds (sched_stat_wait).
    pub sched_stat_wait_distribution: Histogram<u64>,

    /// Observed scheduler runtime in nanoseconds (sched_stat_runtime).
    pub sched_stat_runtime: Gauge<i64>,

    /// Distribution of scheduler runtimes in nanoseconds (sched_stat_runtime).
    pub sched_stat_runtime_distribution: Histogram<u64>,

    /// Current CPU idle C-state per cpu_id, updated only on state change.
    pub cpu_idle_state: Gauge<i64>,

    pub ssl_read_bytes: Gauge<i64>,
    pub ssl_write_bytes: Gauge<i64>,
}

// TODO: add identity metrics with TC classifier packet counts
// TODO: introduce a metric called total_tcp_packets total_udp_packets
impl Metrics {
    /// Initialise all instruments backed by the supplied [`Meter`].
    pub fn new(meter: &Meter) -> Self {
        // total events
        let events_total = meter
            .u64_counter(Semantic::TotalEvents.title())
            .with_description(Semantic::TotalEvents.description())
            //.with_unit("1")
            .build();

        // total socket events
        let socket_events_total = meter
            .u64_counter(Semantic::SocketTotalEvents.title())
            .with_description(Semantic::SocketTotalEvents.description())
            //.with_unit("1")
            .build();

        // socket drops
        let sk_drops = meter
            .i64_gauge(Semantic::SocketDrops.title())
            .with_description(Semantic::SocketDrops.description())
            //.with_unit("1")
            .build();

        // socket errors
        let sk_err = meter
            .i64_gauge(Semantic::SocketErrorsCount.title())
            .with_description(Semantic::SocketErrorsCount.description())
            //.with_unit("1")
            .build();

        // tcp latency microseconds
        let tcp_latency_us = meter
            .u64_histogram(Semantic::Latency.title())
            .with_description(Semantic::Latency.description())
            //.with_unit("us")
            .build();

        // cpu bytes alloc total events
        let cpu_bytes_alloc_events_total = meter
            .u64_counter(Semantic::PerCpuTotalEvents.title())
            .with_description(Semantic::PerCpuTotalEvents.description())
            //.with_unit("1")
            .build();

        // cpu bytes allocation
        let cpu_bytes_alloc = meter
            .i64_gauge(Semantic::PerCpuBytesAllocated.title())
            .with_description(Semantic::PerCpuBytesAllocated.description())
            //.with_unit("bytes")
            .build();

        // memory allocation (mmap) events total
        let mem_alloc_events_total = meter
            .u64_counter(Semantic::TotalMemoryAllocationEvents.title())
            .with_description(Semantic::TotalMemoryAllocationEvents.description())
            //.with_unit("1")
            .build();

        // bytes requested via mmap syscalls
        let enter_mem_alloc = meter
            .i64_gauge(Semantic::RequestedMemoryBytes.title())
            .with_description(Semantic::RequestedMemoryBytes.description())
            //.with_unit("bytes")
            .build();

        // scheduler wait time in nanoseconds
        let sched_stat_wait = meter
            .i64_gauge(Semantic::SchedulerWaitTime.title())
            .with_description(Semantic::SchedulerWaitTime.description())
            //.with_unit("ns")
            .build();

        // distribution of scheduler wait times
        let sched_stat_wait_distribution = meter
            .u64_histogram(Semantic::SchedulerWaitTimeDistribution.title())
            .with_description(Semantic::SchedulerWaitTimeDistribution.description())
            //.with_unit("ns")
            .build();

        // scheduler runtime in nanoseconds
        let sched_stat_runtime = meter
            .i64_gauge(Semantic::SchedulerRuntime.title())
            .with_description(Semantic::SchedulerRuntime.description())
            //.with_unit("ns")
            .build();

        // distribution of scheduler runtimes
        let sched_stat_runtime_distribution = meter
            .u64_histogram(Semantic::SchedulerRuntimeDistribution.title())
            .with_description(Semantic::SchedulerRuntimeDistribution.description())
            //.with_unit("ns")
            .build();

        // current CPU idle C-state per cpu_id
        let cpu_idle_state = meter
            .i64_gauge(Semantic::CpuIdleState.title())
            .with_description(Semantic::CpuIdleState.description())
            .build();

        let ssl_read_bytes = meter
            .i64_gauge(Semantic::SslReadBytes.title())
            .with_description(Semantic::SslReadBytes.description())
            .build();

        let ssl_write_bytes = meter
            .i64_gauge(Semantic::SslWriteBytes.title())
            .with_description(Semantic::SslWriteBytes.description())
            .build();

        Self {
            events_total,
            socket_events_total,
            sk_drops,
            sk_err,
            tcp_latency_us,
            cpu_bytes_alloc,
            cpu_bytes_alloc_events_total,
            mem_alloc_events_total,
            enter_mem_alloc,
            sched_stat_wait,
            sched_stat_wait_distribution,
            sched_stat_runtime,
            sched_stat_runtime_distribution,
            cpu_idle_state,
            ssl_read_bytes,
            ssl_write_bytes,
        }
    }

    /// Build OpenTelemetry attributes from [`Metadata`].
    fn build_attrs(&self, metadata: &Metadata) -> Vec<KeyValue> {
        let mut attrs = Vec::with_capacity(10);

        // base
        attrs.push(KeyValue::new(
            "tgid",
            metadata.tgid.map(|v| v as i64).unwrap_or(-1),
        ));
        attrs.push(KeyValue::new("command", metadata.command.clone()));

        // container metadata
        attrs.push(KeyValue::new(
            "container.name",
            match &metadata.container_name {
                Some(name) => name.clone(),
                None => "null".to_string(),
            },
        ));

        if let Some(ref id) = metadata.container_id {
            attrs.push(KeyValue::new("container.id", id.clone()));
        }
        //if let Some(ref runtime) = metadata.runtime {
        //    attrs.push(KeyValue::new("container.runtime", runtime.clone()));
        //}

        // k8s metadata
        if let Some(ref pod) = metadata.pod_name {
            attrs.push(KeyValue::new("k8s.pod.name", pod.clone()));
        }
        if let Some(ref ns) = metadata.namespace {
            attrs.push(KeyValue::new("k8s.namespace.name", ns.clone()));
        }

        attrs
    }

    /// Record a single [`PacketLossMetrics`] event.
    ///
    /// Increments `events_total` and `socket_events_total`, records `sk_drops`
    /// and `sk_err` as gauges.
    pub fn record_packet_loss_metrics(&self, m: &PacketLossMetrics, metadata: &Metadata) {
        let attrs = &self.build_attrs(metadata);

        self.events_total.add(1, attrs);
        self.socket_events_total.add(1, attrs);
        self.sk_drops.record(m.sk_drops as i64, attrs);
        self.sk_err.record(m.sk_err as i64, attrs);
    }

    /// Record a single [`TimeStampMetrics`] event.
    ///
    /// Increments `events_total`, and records `delta_us` in the latency
    /// histogram.
    pub fn record_timestamp_metrics(&self, m: &TimeStampMetrics, metadata: &Metadata) {
        let attrs = &self.build_attrs(metadata);

        self.events_total.add(1, attrs);
        self.tcp_latency_us.record(m.delta_us, attrs);
    }

    /// Record a single [`CpuFrequency`] event.
    pub fn record_cpu_bytes_alloc(&self, m: &CpuFrequency, metadata: &Metadata) {
        let bytes_allocated = m.bytes_alloc;
        let attrs = &self.build_attrs(metadata);

        self.cpu_bytes_alloc_events_total.add(1, attrs);
        self.cpu_bytes_alloc.record(bytes_allocated as i64, attrs);
    }

    /// Record a single [`MemAlloc`] event (mmap syscall).
    ///
    /// Increments the dedicated `mem_alloc_events_total` counter and records
    /// the requested length in the `enter_mem_alloc` gauge.  The shared
    /// `events_total` counter is intentionally **not** incremented for these
    /// events.
    pub fn record_enter_mem_alloc(&self, m: &MemAlloc, metadata: &Metadata) {
        let attrs = &self.build_attrs(metadata);

        self.events_total.add(1, attrs);
        self.mem_alloc_events_total.add(1, attrs);
        self.enter_mem_alloc.record(m.length as i64, attrs);
    }

    /// Record a single [`SchedStatWait`] event.
    ///
    /// Increments `events_total`, records `delay` in the `sched_stat_wait`
    /// gauge, and observes `delay` in the `sched_stat_wait_distribution`
    /// histogram.
    pub fn record_sched_stat_wait(&self, m: &SchedStatWait, metadata: &Metadata) {
        let attrs = &self.build_attrs(metadata);

        self.events_total.add(1, attrs);
        self.sched_stat_wait.record(m.delay as i64, attrs);
        self.sched_stat_wait_distribution.record(m.delay, attrs);
    }

    /// Record a single [`SchedStatRuntime`] event.
    ///
    /// Increments `events_total`, records `runtime` in the `sched_stat_runtime`
    /// gauge, and observes `runtime` in the `sched_stat_runtime_distribution`
    /// histogram.
    pub fn record_sched_stat_runtime(&self, m: &SchedStatRuntime, metadata: &Metadata) {
        let attrs = &self.build_attrs(metadata);

        self.events_total.add(1, attrs);
        self.sched_stat_runtime.record(m.runtime as i64, attrs);
        self.sched_stat_runtime_distribution
            .record(m.runtime, attrs);
    }

    /// Record a single [`CpuIdle`] event.
    ///
    /// Updates `cpu_idle_state` gauge to the latest C-state for the given
    /// `cpu_id`. Events are only emitted by eBPF when the state changes.
    pub fn record_cpu_idle(&self, m: &CpuIdle, metadata: &Metadata) {
        let mut attrs = self.build_attrs(metadata);
        attrs.push(KeyValue::new("cpu_id", m.cpu_id as i64));

        self.events_total.add(1, &attrs);
        self.cpu_idle_state.record(m.state as i64, &attrs);
    }

    pub fn record_ssl_read_bytes(&self, m: &SslEvent, metadata: &Metadata) {
        let attrs = self.build_attrs(metadata);

        self.events_total.add(1, &attrs);
        self.ssl_read_bytes.record(m.size as i64, &attrs);
    }
    pub fn record_ssl_write_bytes(&self, m: &SslEvent, metadata: &Metadata) {
        let attrs = self.build_attrs(metadata);

        self.events_total.add(1, &attrs);
        self.ssl_write_bytes.record(m.size as i64, &attrs);
    }
}
