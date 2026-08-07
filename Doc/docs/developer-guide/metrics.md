# Integrated Metrics

CortexBrain's metrics are documented below. Since version **0.1.5** we've introduced the OpenTelemetry procol (OTLP) to process and transport the metrics accross the full pipeline. Below we've documented all the featured metrics 

See the [Agent API Overview](agent-api.md) for how the gRPC methods map to the BPF maps behind these metrics.

## CLI-Featured Metrics

These metrics are collected by eBPF perf buffers and surfaced through the cortexflow agent. There is no Prometheus endpoint on the current branch - the data is pulled via `cfcli` or any tonic gRPC client.

### Connection events - `ActiveConnections`

Connection events are produced by the `conntracker` TC classifier and read from the `events_map` perf buffer. The agent converts each `PacketLog` into a `ConnectionEvent` proto.

| Field | Type | Description |
|-------|------|-------------|
| `event_id` | `string` | The PID of the process that triggered the event |
| `src_ip_port` | `string` | Source IP and port, formatted as `ip:port` |
| `dst_ip_port` | `string` | Destination IP and port, formatted as `ip:port` |

**Source**: `core/api/protos/agent.proto` (`ConnectionEvent`), `core/src/components/conntracker/src/data_structures.rs` (`PacketLog`), `core/api/src/api.rs` (handler).

**CLI**: `cfcli monitoring connections`

### Latency metrics - `GetLatencyMetrics`

Latency metrics measure the time spent in `TCP_SYN_SENT` for IPv4/IPv6 connections. The `metrics_tracer` eBPF program kprobes `tcp_v4_connect`/`tcp_v6_connect` (start timestamp) and `tcp_rcv_state_process` (delta computation), then emits `TimeStampEvent`s to the `time_stamp_events` perf buffer.

Each `LatencyMetric`:

| Field | Type | Description |
|-------|------|-------------|
| `delta_us` | `uint64` | Latency in microseconds (SYN → established) |
| `timestamp_us` | `uint64` | Event timestamp, µs since boot (`bpf_ktime_get_ns`) |
| `tgid` | `uint32` | Thread group ID (process) |
| `process_name` | `string` | Process name (`comm`, 16 bytes) |
| `local_port` | `uint32` | Local port |
| `remote_port` | `uint32` | Remote port (big-endian) |
| `address_family` | `uint32` | `AF_INET=2` / `AF_INET6=10` |
| `src_address_v4` / `dst_address_v4` | `string` | IPv4 source / destination |
| `src_address_v6` / `dst_address_v6` | `string` | IPv6 source / destination |

The response (`LatencyMetricsResponse`) also includes aggregates:

| Field | Type | Description |
|-------|------|-------------|
| `total_count` | `uint32` | Number of metrics returned |
| `average_latency_us` | `double` | Average `delta_us` |
| `min_latency_us` | `double` | Minimum `delta_us` |
| `max_latency_us` | `double` | Maximum `delta_us` |

**Source**: `core/api/protos/agent.proto` (`LatencyMetric`, `LatencyMetricsResponse`), `core/src/components/metrics_tracer/src/main.rs` (kprobes), `core/api/src/api.rs` (handler).

**CLI**: `cfcli monitoring latencymetrics`

### Dropped packet metrics - `GetDroppedPacketsMetrics`

Dropped packet metrics come from the `metrics_tracer` kprobe that reads `struct sock` fields at hardcoded kernel offsets. The agent only forwards entries where `sk_drops > 0`. Each `DroppedPacketMetric`:

| Field | Type | Kernel offset | Description |
|-------|------|---------------|-------------|
| `tgid` | `uint32` | - | Thread group ID |
| `process_name` | `string` | - | Process name (`comm`) |
| `timestamp_us` | `uint64` | - | Event timestamp (µs) |
| `sk_drops` | `int32` | 136 | Socket drops |
| `sk_err` | `int32` | 284 | Socket errors |
| `sk_err_soft` | `int32` | 600 | Soft errors |
| `sk_backlog_len` | `int32` | 196 | Backlog length (congestion indicator) |
| `sk_wmem_queued` | `int32` | 376 | Write memory queued |
| `sk_rcvbuf` | `int32` | 244 | Receive buffer size |
| `sk_ack_backlog` | `uint32` | 604 | ACK backlog |

The response (`DroppedPacketsResponse`) also includes `total_drops` (`uint32`), the sum of `sk_drops` across all returned metrics.

**Source**: `core/api/protos/agent.proto` (`DroppedPacketMetric`, `DroppedPacketsResponse`), `core/src/components/metrics_tracer/src/main.rs` (kprobe + offsets), `core/api/src/api.rs` (handler).

**CLI**: `cfcli monitoring droppedpackets`

The **0.1.5** version adds a full OpenTelemetry pipeline:

- **Exporter**: OTLP, via gRPC (`http://localhost:4317`) or HTTP (`http://localhost:4318`), controlled by the `OTEL_EXPORTER_OTLP_ENDPOINT` environment variable.
- **Reader**: `PeriodicReader` with a 5-second export interval.
- **Meter name**: `cortexbrain-metrics`.

The instrument definitions live in `core/common/src/otel_metrics.rs` and `core/common/src/semantic.rs` (on the `metrics-patch` branch). They are grouped below by category.

## Prometheus-Featured metrics

These metrics are collected by eBPF perf buffers and forwarded to the userspace using the OTLP protocol. These metrics are collected and aggregated by the OpenTelemetry agent, the collector expose the metrics through the API and the Prometheus scraper crawls and make them available in the Grafana Dashboard. The `CortexBrain Dashboard` use PromQL to create the visualizations.

### Common attributes (labels)

All OTel instruments share the following attribute set:

| Attribute | Description |
|-----------|-------------|
| `tgid` | Thread group ID |
| `command` | Process name |
| `container.name` | Container name |
| `container.id` | Container ID (optional) |
| `k8s.pod.name` | Pod name (optional) |
| `k8s.namespace.name` | Namespace name (optional) |

### Socket / network metrics

| Instrument name | Type | Description |
|-----------------|------|-------------|
| `events_total` | `Counter<u64>` | Total eBPF events processed across all perf buffers |
| `socket_events_total` | `Counter<u64>` | Total socket state events processed |
| `sk_drops` | `Gauge<i64>` | Socket drop count per event |
| `sk_err` | `Gauge<i64>` | Socket error count per event |
| `latency_us` | `Histogram<u64>` | Distribution of latency values from timestamp events |

### CPU metrics

| Instrument name | Type | Description |
|-----------------|------|-------------|
| `bytes_alloc_events_total` | `Counter<u64>` | Total `bytes_alloc` events in the CPU |
| `cpu_bytes_alloc` | `Gauge<i64>` | CPU bytes allocation per event |
| `cpu_idle_state` | `Gauge<i64>` | Current CPU idle C-state per `cpu_id`, updated on state change |

### Memory metrics

| Instrument name | Type | Description |
|-----------------|------|-------------|
| `mem_alloc_events_total` | `Counter<u64>` | Total memory allocation (`mmap`) events |
| `enter_mem_alloc` | `Gauge<i64>` | Bytes requested via `mmap` syscalls |

### Scheduler metrics

| Instrument name | Type | Description |
|-----------------|------|-------------|
| `sched_stat_wait` | `Gauge<i64>` | Scheduler wait time (ns) from `sched_stat_wait` |
| `sched_stat_wait_distribution` | `Histogram<u64>` | Distribution of scheduler wait times (ns) |
| `sched_stat_runtime` | `Gauge<i64>` | Scheduler runtime (ns) from `sched_stat_runtime` |
| `sched_stat_runtime_distribution` | `Histogram<u64>` | Distribution of scheduler runtimes (ns) |

### SSL metrics

| Instrument name | Type | Description |
|-----------------|------|-------------|
| `ssl_read_bytes` | `Gauge<i64>` | Total bytes requested by `SSL_read` |
| `ssl_write_bytes` | `Gauge<i64>` | Total bytes requested by `SSL_write` |

### Metrics source code

The **0.1.5** version update the following modules (`core/src/components/metrics_tracer/src/`):

- `cpu.rs` - CPU frequency and bytes-alloc events
- `memory.rs` - `mmap` syscall tracing
- `network.rs` - packet-loss and timestamp events (extended from current)
- `ssl.rs` - `SSL_read` / `SSL_write` tracing via `ssl_ctx_map`
