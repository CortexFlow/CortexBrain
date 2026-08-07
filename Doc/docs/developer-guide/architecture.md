# Architecture Overview

This page describes the CortexBrain architecture with a focus on the **general pipeline**: how data flows from the Linux kernel, through eBPF programs and pinned BPF maps, to the user-space gRPC agent, and finally to the `cfcli` client (and, in the future, the dashboard).

The conceptual diagram on the [home page](../index.md#architecture) gives the high-level picture; this page is the implementation-accurate reference.

## The pipeline at a glance

CortexBrain is built around a four-stage pipeline. Each stage has a clear responsibility and a well-defined boundary with the next.

| Stage | Components | Role | Output |
|-------|------------|------|--------|
| **1. Kernel instrumentation** | `conntracker`, `metrics_tracer` (eBPF) | Observe at kernel level via TC classifiers and kprobes | PerfEventArray fills |
| **2. Map pinning** | `identity`, `metrics` (user-space loaders) | Load + attach the eBPF programs, pin the maps to `/sys/fs/bpf/`, seed policy | Pinned maps in bpffs |
| **3. Aggregation & serving** | `agent` (`core/api`) - gRPC on `:9090` | Open pinned maps, drain perf buffers into mpsc channels, serve 6 RPCs | gRPC responses |
| **4. Consumption** | `cfcli` (live), `dashboard` (placeholder) | Present the data to the operator | Terminal / UI |

!!! tip "Reading order"
    If you want the detail behind each stage, after this page read the [Agent API Overview](agent-api.md) for the gRPC surface and the [Integrated Metrics](metrics.md) page for the metric field schemas.

## Full pipeline (current - `main` branch)

```
┌─────────────────────────── KERNEL SPACE (eBPF) ───────────────────────────┐
│                                                                            │
│  conntracker object - loaded by the identity service                       │
│                                                                            │
│   #[classifier] identity_classifier   ── TC ingress ── veth interfaces      │
│       reads BLOCKLIST map; drops packet (TC_ACT_SHOT) if src_ip blocklisted│
│       └─► EventsMap            [PerfEventArray]  ── PacketLog               │
│            (proto, src_ip, src_port, dst_ip, dst_port, pid)                 │
│                                                                            │
│   #[kprobe] veth_creation_trace     ── register_netdevice                   │
│   #[kprobe] veth_deletion_trace     ── unregister_netdevice_queue           │
│       └─► veth_identity_map      [PerfEventArray]  ── VethLog                │
│            (name, state, dev_addr, event_type 1=create/2=delete, netns, pid) │
│                                                                            │
│   #[kprobe] tcp_message_tracer     ── tcp_v4_rcv + tcp_v4_connect           │
│       └─► TcpPacketRegistry     [PerfEventArray]  ── TcpPacketRegistry      │
│            (proto, src_ip, dst_ip, src_port, dst_port, pid, comm, cgroup_id) │
│                                                                            │
│  metrics_tracer object - loaded by the metrics service                     │
│                                                                            │
│   #[kprobe] metrics_tracer         ── tcp_identify_packet_loss               │
│       └─► net_metrics           [PerfEventArray]  ── NetworkMetrics          │
│            (tgid, comm, ts_us, sk_err, sk_err_soft, sk_backlog_len,        │
│             sk_wmem_queued, sk_rcvbuf, sk_ack_backlog, sk_drops)            │
│            (reads struct sock at hardcoded kernel offsets)                  │
│                                                                            │
│   #[kprobe] tcp_connect            ── tcp_v4_connect + tcp_v6_connect       │
│       └─► time_stamp_start       [HashMap sk*->TimeStampStartInfo]            │
│            (comm, ts_ns, tgid) - internal start-timestamp state              │
│                                                                            │
│   #[kprobe] tcp_rcv_state_process  ── tcp_rcv_state_process                 │
│       reads time_stamp_start, computes delta_us (SYN -> established)         │
│       └─► time_stamp_events      [PerfEventArray]  ── TimeStampEvent         │
│            (delta_us, ts_us, tgid, comm, lport, dport_be, af,                │
│             saddr_v4, daddr_v4, saddr_v6[4], daddr_v6[4])                   │
│                                                                            │
│   Blocklist map: HashMap<[u8;4]->[u8;4]>  (read by TC, written by userspace)  │
└────────────────────────────────────────────────────────────────────────────┘
        │  perf buffer output / map reads
        ▼
┌─────────────────── USERSPACE LOADERS (pin maps) ──────────────────────────┐
│                                                                            │
│  identity service (core/src/components/identity)                          │
│   BPF_PATH env ──► Ebpf::load(conntracker)                                 │
│   init_bpf_maps ──► map_pinner to ${PIN_MAP_PATH}:                          │
│     EventsMap          ──► events_map          (consumed by agent)         │
│     veth_identity_map  ──► veth_map             (display-only, identity)   │
│     Blocklist          ──► blocklist_map        (consumed + mutated agent) │
│     TcpPacketRegistry  ──► tcp_packet_registry  (display-only, identity)   │
│   populate_blocklist: seeds Blocklist from                                  │
│     ConfigMap cortexbrain-client-config (namespace cortexflow)             │
│   attaches TC classifier to discovered veths; kprobes veth/tcp             │
│   dynamically re-attaches TC on veth creation events                        │
│                                                                            │
│  metrics service (core/src/components/metrics)                             │
│   BPF_PATH env ──► Ebpf::load(metrics_tracer)                              │
│   init_ebpf_maps ──► map_pinner to ${PIN_MAP_PATH}:                         │
│     net_metrics        ──► net_metrics         (consumed by agent)         │
│     time_stamp_events  ──► time_stamp_events   (consumed by agent)         │
│   attaches kprobes (tcp_identify_packet_loss, tcp_v4/v6_connect,           │
│     tcp_rcv_state_process)                                                 │
└────────────────────────────────────────────────────────────────────────────┘
        │  pinned maps in /sys/fs/bpf/{maps,trace_maps}/...
        ▼
┌─────────────────── AGENT (core/api) - gRPC :9090 ─────────────────────────┐
│                                                                            │
│  AgentApi::default() opens pinned maps (hardcoded paths):                  │
│   /sys/fs/bpf/maps/events_map             ──► PacketLog reader            │
│   /sys/fs/bpf/trace_maps/net_metrics       ──► NetworkMetrics reader      │
│   /sys/fs/bpf/trace_maps/time_stamp_events ──► TimeStampMetrics reader     │
│  3 tokio::spawn reader tasks ──► mpsc channels:                            │
│     active_connection_event_tx  (capacity 1024)                            │
│     latency_metrics_tx          (capacity 2048)                            │
│     dropped_packet_metrics_tx   (capacity 2048)                            │
│  /sys/fs/bpf/maps/blocklist_map opened per blocklist RPC                    │
│                                                                            │
│  agent.Agent gRPC service - 6 RPCs + tonic server reflection:              │
│     ActiveConnections / AddIpToBlocklist / CheckBlocklist /                │
│     RmIpFromBlocklist / GetLatencyMetrics / GetDroppedPacketsMetrics        │
└────────────────────────────────────────────────────────────────────────────┘
        │  gRPC (plaintext HTTP/2, 127.0.0.1:9090 via kubectl port-forward)
        ▼
┌─────────────────── CONSUMPTION ───────────────────────────────────────────┐
│  cfcli (cli/) ── tonic gRPC client ── terminal output                       │
│     cfcli monitoring list            ──► reflection                        │
│     cfcli monitoring connections     ──► ActiveConnections                 │
│     cfcli monitoring latencymetrics  ──► GetLatencyMetrics                 │
│     cfcli monitoring droppedpackets  ──► GetDroppedPacketsMetrics           │
│     cfcli policy create-blocklist --flags <IP> ──► AddIpToBlocklist        │
│     cfcli policy check-blocklist              ──► CheckBlocklist           │
│     cfcli policy remove-ip --flags <IP>       ──► RmIpFromBlocklist        │
│     (also mirrors blocklist into cortexbrain-client-config ConfigMap)       │
│                                                                            │
│  dashboard (dashboard/) ── STATIC PLACEHOLDER, no agent/OTLP wiring        │
│     React/Electron shell; hardcoded values; future integration             │
└────────────────────────────────────────────────────────────────────────────┘

Side-channel:  ConfigMap cortexbrain-client-config (namespace cortexflow)
               ─ seeded by cfcli, read by identity to populate Blocklist
               ─ updated by cfcli to mirror blocklist policy changes
```

## Future pipeline (incoming - `metrics-patch` branch)

!!! warning "Not yet merged to `main`
    The OpenTelemetry export pipeline below lives on the `origin/metrics-patch` and `origin/cpu-metrics` branches. It is documented here so contributors know what is coming, but it is not yet available on the default branch.

```
agent (core/api) ──(future)──► OTLP exporter
   controlled by OTEL_EXPORTER_OTLP_ENDPOINT env
   gRPC http://localhost:4317  or  HTTP http://localhost:4318
   PeriodicReader, 5s export interval, meter "cortexbrain-metrics"
        │
        ▼
otel-collector (NOT deployed on main) ──(future)──► dashboard / Grafana / Prometheus
```

The `metrics-patch` branch adds 16 OpenTelemetry instruments across socket/network, CPU, memory, scheduler, and SSL - see the [Integrated Metrics](metrics.md#opentelemetry-metrics-incoming-metrics-patch) page for the full instrument list.

## Kernel hooks

CortexBrain attaches eBPF programs to the following kernel hook points. The diagram below shows where these hooks sit in the Linux network stack.

![Linux network stack and eBPF hooks](../assets/linux-net-stack.svg "Linux network stack with eBPF hook points")

| Program | Attach type | Target function | BPF map written | Data produced |
|---------|-------------|-----------------|------------------|---------------|
| `identity_classifier` | TC classifier (ingress) | veth interfaces | `EventsMap` | `PacketLog` (per-packet metadata) |
| `veth_creation_trace` | kprobe | `register_netdevice` | `veth_identity_map` | `VethLog` (event_type=1) |
| `veth_deletion_trace` | kprobe | `unregister_netdevice_queue` | `veth_identity_map` | `VethLog` (event_type=2) |
| `tcp_message_tracer` | kprobe | `tcp_v4_rcv`, `tcp_v4_connect` | `TcpPacketRegistry` | `TcpPacketRegistry` (TCP flow metadata) |
| `metrics_tracer` | kprobe | `tcp_identify_packet_loss` | `net_metrics` | `NetworkMetrics` (socket stats + drops) |
| `tcp_connect` | kprobe | `tcp_v4_connect`, `tcp_v6_connect` | `time_stamp_start` | start timestamp keyed by socket pointer |
| `tcp_rcv_state_process` | kprobe | `tcp_rcv_state_process` | `time_stamp_events` | `TimeStampEvent` (latency `delta_us`) |

**Source files**:
- `core/src/components/conntracker/src/main.rs` and sub-modules (`tc.rs`, `veth_tracer.rs`, `tcp_analyzer.rs`, `data_structures.rs`)
- `core/src/components/metrics_tracer/src/main.rs` and `data_structures.rs`

## Deployment topology

All CortexBrain components run in the `cortexflow` namespace. The core deployments use `hostPID: true`, `hostNetwork: true`, and `privileged: true` with the `BPF`, `SYS_ADMIN`, `NET_ADMIN`, `SYS_PTRACE`, and `SYS_RESOURCE` capabilities, and mount the host `/sys/fs/bpf` (bidirectional), `/proc`, and `/lib/modules`.

| Pod (Deployment) | Container & binary | Image | Network exposure | BPF maps (producer / consumer) |
|------------------|--------------------|-------|------------------|--------------------------------|
| `cortexflow-agent` | `agent` -> `/usr/local/bin/agent-api` | `lorenzotettamanti/cortexflow-agent:latest` | Service `cortexflow-agent` ClusterIP TCP 9090 (grpc); reached by `cfcli` via `kubectl port-forward` | **Consumer only** - reads `events_map`, `net_metrics`, `time_stamp_events`, `blocklist_map` |
| `cortexflow-identity` | initContainer `bpf-map-permissions` (mounts bpffs); `identity` -> `/usr/local/bin/cortexflow-identity-service`; sidecar `bpftool-control-manager` | `lorenzotettamanti/cortexflow-identity:latest` | None (no Service) | **Producer** - pins `events_map`, `veth_map`, `blocklist_map`, `tcp_packet_registry`; seeds `blocklist_map` from the `cortexbrain-client-config` ConfigMap |
| `cortexflow-metrics` | `metrics` -> `/usr/local/bin/cortexflow-metrics`; sidecar `bpftool-control-manager` | `lorenzotettamanti/cortexflow-metrics:latest` | None (no Service) | **Producer** - pins `net_metrics`, `time_stamp_events` |

**Manifests**: `core/src/testing/agent.yaml`, `core/src/testing/identity.yaml`, `core/src/testing/metrics.yaml` (plus the RBAC and test-pod manifests in the same directory).

## Caveats

!!! warning "Known limitations"
    - **Pinned map path mismatch**: the agent hardcodes `/sys/fs/bpf/maps/...` and `/sys/fs/bpf/trace_maps/...`, while the Identity and Metrics loaders pin under `${PIN_MAP_PATH}`. The deployment must reconcile these paths (the manifests mount `/sys/fs/bpf` as a bidirectional hostPath volume).
    - **Plaintext gRPC**: the agent binds to `0.0.0.0:9090` and the client connects to `http://127.0.0.1:9090` - both flagged with `FIXME` in the code. TLS is not yet implemented.
    - **Dashboard not wired**: the dashboard is a static React/Electron shell with hardcoded values. It does not yet call the agent gRPC or any OTLP collector.
    - **Unused `pod_ip` field**: `RequestActiveConnections.pod_ip` is read by the server but not used for filtering - all events are returned regardless.

## Where to go next

- **[Agent API Overview](agent-api.md)** - the 6 gRPC RPCs and the BPF maps behind them, in detail.
- **[Integrated Metrics](metrics.md)** - the field schemas for `ConnectionEvent`, `LatencyMetric`, `DroppedPacketMetric`, and the incoming OpenTelemetry instruments.
- **[Development Workflow](dev-workflow.md)** - how to build the components locally and submit changes.
- **[Development Goals & Discussions](discussions.md)** - milestones, roadmap, and how to propose new features.