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

use crate::buffer_type::{NetworkMetrics, TimeStampMetrics};
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
}

impl Metrics {
    /// Initialise all instruments backed by the supplied [`Meter`].
    pub fn new(meter: &Meter) -> Self {
        // total events
        let events_total = meter
            .u64_counter("cortexbrain_events_total")
            .with_description("Total number of eBPF events processed")
            .build();

        // total packets
        let packets_total = meter
            .u64_counter("cortexbrain_packets_total")
            .with_description("Total number of network events processed")
            .build();

        // socket drops
        let sk_drops = meter
            .i64_gauge("cortexbrain_sk_drops")
            .with_description("Socket drop count per event")
            .build();

        // socket errors
        let sk_err = meter
            .i64_gauge("cortexbrain_sk_err")
            .with_description("Socket error count per event")
            .build();

        // delta microseconds
        let delta_us = meter
            .u64_histogram("cortexbrain_delta_us")
            .with_description("Distribution of delta_us values from timestamp events")
            .build();

        // timestamp microseconds grouped
        let ts_us = meter
            .u64_histogram("cortexbrain_ts_us")
            .with_description("Distribution of timestamp values from eBPF events")
            .build();

        Self {
            events_total,
            packets_total,
            sk_drops,
            sk_err,
            delta_us,
            ts_us,
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
}
