#![cfg(feature = "monitoring-structs")]

use bytemuck::Zeroable;
use cortexbrain_common::buffer_type::{
    CpuFrequency, CpuIdle, MemAlloc, PacketLossMetrics, SchedStatRuntime, SchedStatWait, SslEvent,
    TimeStampMetrics,
};
use cortexbrain_common::metadata::Metadata;
use cortexbrain_common::otel_metrics::Metrics;
use opentelemetry::KeyValue;
use opentelemetry::metrics::MeterProvider;
use opentelemetry_sdk::metrics::data::{AggregatedMetrics, Metric, MetricData, ResourceMetrics};
use opentelemetry_sdk::metrics::{InMemoryMetricExporter, SdkMeterProvider};

fn collect(record: impl FnOnce(&Metrics)) -> ResourceMetrics {
    let exporter = InMemoryMetricExporter::default();
    let provider = SdkMeterProvider::builder()
        .with_periodic_exporter(exporter.clone())
        .build();
    let metrics = Metrics::new(&provider.meter("cortexbrain-common-test"));

    record(&metrics);
    provider
        .force_flush()
        .expect("metric export should succeed");
    let snapshot = exporter
        .get_finished_metrics()
        .expect("in-memory metrics should be readable")
        .pop()
        .expect("metric export should contain a snapshot");
    provider
        .shutdown()
        .expect("meter provider should shut down");
    snapshot
}

fn metric<'a>(snapshot: &'a ResourceMetrics, name: &str) -> &'a Metric {
    snapshot
        .scope_metrics()
        .flat_map(|scope| scope.metrics())
        .find(|metric| metric.name() == name)
        .unwrap_or_else(|| panic!("missing metric {name}"))
}

fn sum(snapshot: &ResourceMetrics, name: &str, expected: u64) -> Vec<KeyValue> {
    let AggregatedMetrics::U64(MetricData::Sum(sum)) = metric(snapshot, name).data() else {
        panic!("{name} should be a u64 counter");
    };
    let points: Vec<_> = sum.data_points().collect();
    assert_eq!(points.len(), 1, "{name} should have one attribute set");
    assert_eq!(points[0].value(), expected, "wrong {name} count");
    points[0].attributes().cloned().collect()
}

fn gauge(snapshot: &ResourceMetrics, name: &str, expected: i64) -> Vec<KeyValue> {
    let AggregatedMetrics::I64(MetricData::Gauge(gauge)) = metric(snapshot, name).data() else {
        panic!("{name} should be an i64 gauge");
    };
    let points: Vec<_> = gauge.data_points().collect();
    assert_eq!(points.len(), 1, "{name} should have one attribute set");
    assert_eq!(points[0].value(), expected, "wrong {name} value");
    points[0].attributes().cloned().collect()
}

fn histogram(snapshot: &ResourceMetrics, name: &str, count: u64, total: u64) {
    let AggregatedMetrics::U64(MetricData::Histogram(histogram)) = metric(snapshot, name).data()
    else {
        panic!("{name} should be a u64 histogram");
    };
    let points: Vec<_> = histogram.data_points().collect();
    assert_eq!(points.len(), 1, "{name} should have one attribute set");
    assert_eq!(points[0].count(), count, "wrong {name} observation count");
    assert_eq!(points[0].sum(), total, "wrong {name} sum");
}

#[test]
fn packet_loss_counts_events_and_exports_process_and_container_attributes() {
    let mut metadata = Metadata::from_ebpf(Some(42), Some(123), b"worker\0");
    metadata.container_name = Some("container-a".into());
    metadata.container_id = Some("container-id".into());
    metadata.pod_name = Some("pod-a".into());
    metadata.namespace = Some("team-a".into());

    let snapshot = collect(|metrics| {
        let mut event = PacketLossMetrics::zeroed();
        event.sk_drops = 3;
        event.sk_err = -2;
        metrics.record_packet_loss_metrics(&event, &metadata);
        event.sk_drops = 7;
        event.sk_err = -4;
        metrics.record_packet_loss_metrics(&event, &metadata);
    });

    let attrs = sum(&snapshot, "events_total", 2);
    sum(&snapshot, "socket_events_total", 2);
    gauge(&snapshot, "sk_drops", 7);
    gauge(&snapshot, "sk_err", -4);
    assert_eq!(attrs.len(), 6);
    for expected in [
        KeyValue::new("tgid", 42_i64),
        KeyValue::new("command", "worker"),
        KeyValue::new("container.name", "container-a"),
        KeyValue::new("container.id", "container-id"),
        KeyValue::new("k8s.pod.name", "pod-a"),
        KeyValue::new("k8s.namespace.name", "team-a"),
    ] {
        assert!(attrs.contains(&expected), "missing {expected:?}");
    }
}

#[test]
fn latency_histogram_aggregates_observations_without_optional_attributes() {
    let metadata = Metadata::from_ebpf(None, None, b"host-process\0");
    let snapshot = collect(|metrics| {
        let mut event = TimeStampMetrics::zeroed();
        event.delta_us = 7;
        metrics.record_timestamp_metrics(&event, &metadata);
        event.delta_us = 11;
        metrics.record_timestamp_metrics(&event, &metadata);
    });

    let attrs = sum(&snapshot, "events_total", 2);
    histogram(&snapshot, "latency_us", 2, 18);
    assert_eq!(attrs.len(), 2);
    assert!(attrs.contains(&KeyValue::new("tgid", -1_i64)));
    assert!(attrs.contains(&KeyValue::new("command", "host-process")));
}

#[test]
fn memory_allocation_records_its_dedicated_counter_and_gauge() {
    let metadata = Metadata::from_ebpf(Some(7), None, b"allocator");
    let snapshot = collect(|metrics| {
        let mut event = MemAlloc::zeroed();
        event.length = 4096;
        metrics.record_enter_mem_alloc(&event, &metadata);
    });

    sum(&snapshot, "mem_alloc_events_total", 1);
    gauge(&snapshot, "enter_mem_alloc", 4096);
    // Do not freeze events_total here: its documentation and implementation disagree.
}

#[test]
fn scheduler_events_record_latest_values_and_distributions() {
    let metadata = Metadata::from_ebpf(Some(8), None, b"scheduler");
    let snapshot = collect(|metrics| {
        let mut wait = SchedStatWait::zeroed();
        wait.delay = 10;
        metrics.record_sched_stat_wait(&wait, &metadata);
        wait.delay = 20;
        metrics.record_sched_stat_wait(&wait, &metadata);

        let mut runtime = SchedStatRuntime::zeroed();
        runtime.runtime = 15;
        metrics.record_sched_stat_runtime(&runtime, &metadata);
    });

    sum(&snapshot, "events_total", 3);
    gauge(&snapshot, "sched_stat_wait", 20);
    histogram(&snapshot, "sched_stat_wait_distribution", 2, 30);
    gauge(&snapshot, "sched_stat_runtime", 15);
    histogram(&snapshot, "sched_stat_runtime_distribution", 1, 15);
}

#[test]
fn cpu_idle_and_ssl_events_keep_their_dedicated_series() {
    let metadata = Metadata::from_ebpf(Some(9), None, b"monitor");
    let snapshot = collect(|metrics| {
        let mut cpu = CpuFrequency::zeroed();
        cpu.bytes_alloc = 256;
        metrics.record_cpu_bytes_alloc(&cpu, &metadata);

        let mut idle = CpuIdle::zeroed();
        idle.cpu_id = 2;
        idle.state = 3;
        metrics.record_cpu_idle(&idle, &metadata);

        let mut ssl = SslEvent::zeroed();
        ssl.size = 64;
        metrics.record_ssl_read_bytes(&ssl, &metadata);
        ssl.size = 128;
        metrics.record_ssl_write_bytes(&ssl, &metadata);
    });

    sum(&snapshot, "bytes_alloc_events_total", 1);
    gauge(&snapshot, "cpu_bytes_alloc", 256);
    let idle_attrs = gauge(&snapshot, "cpu_idle_state", 3);
    assert!(idle_attrs.contains(&KeyValue::new("cpu_id", 2_i64)));
    gauge(&snapshot, "ssl_read_bytes", 64);
    gauge(&snapshot, "ssl_write_bytes", 128);
    let AggregatedMetrics::U64(MetricData::Sum(total)) = metric(&snapshot, "events_total").data()
    else {
        panic!("events_total should be a u64 counter");
    };
    let points: Vec<_> = total.data_points().collect();
    assert_eq!(points.len(), 2);
    assert!(points.iter().any(|point| {
        point.value() == 1
            && point
                .attributes()
                .any(|attr| attr == &KeyValue::new("cpu_id", 2_i64))
    }));
    assert!(points.iter().any(|point| {
        point.value() == 2 && point.attributes().all(|attr| attr.key.as_str() != "cpu_id")
    }));
}
