# Agent API Overview

The CortexBrain **Agent** is the user-space gRPC server that exposes the data collected by the eBPF kernel programs to clients such as [`cfcli`](../cfcli/overview.md). This page gives an overview of the service surface and the data flow behind it. The full message schema lives in [`core/api/protos/agent.proto`](https://github.com/CortexFlow/CortexBrain/blob/main/core/api/protos/agent.proto) in the repository.

## What the Agent is

The Agent is the `cortexflow_agent_api` crate (`core/api/` in the workspace). It is a [tonic](https://docs.rs/tonic/) gRPC server that:

- Listens on `0.0.0.0:9090` (`core/api/src/main.rs`).
- Registers the `agent.Agent` service together with a tonic **server reflection** service, so clients can enumerate the available RPCs at runtime - this is what `cfcli monitoring list` uses.
- On startup, opens a set of **pinned BPF maps** from `/sys/fs/bpf/...` and spawns background reader tasks that drain the perf buffers into mpsc channels.
- Serves 6 RPC methods (listed below) that drain those channels and return the data to the caller.

The Agent is deployed as the `cortexflow-agent` Kubernetes service (ClusterIP, port 9090). The CLI reaches it via `kubectl port-forward svc/cortexflow-agent 9090:9090 -n cortexflow`.

## Data flow

The Agent is a **consumer and aggregator** of data produced by two other services, the **Identity** service and the **Metrics** service, which load the eBPF programs and pin the maps. The Agent never loads eBPF itself; it only opens maps that someone else has already pinned.

```
eBPF kernel programs          (conntracker, metrics_tracer)
        │  output() to PerfEventArray
        ▼
User-space loaders           (identity, metrics)  ── pin maps to /sys/fs/bpf/...
        │
        ▼
Agent (core/api)             opens pinned maps
   spawns 3 reader tasks     ── decode structs ── push into mpsc channels
        │
        ▼
Agent gRPC handlers          drain mpsc channels on each RPC call
        │
        ▼
cfcli (cli/)                 gRPC client ── user terminal
```

### The four BPF maps the Agent reads

| Map (kernel name) | Pinned path | Producer | Userspace struct | Feeds RPC |
|-------------------|-------------|----------|------------------|-----------|
| `EventsMap` | `/sys/fs/bpf/maps/events_map` | Identity (`identity/src/map_handlers.rs`) | `PacketLog` | `ActiveConnections` |
| `net_metrics` | `/sys/fs/bpf/trace_maps/net_metrics` | Metrics (`metrics/src/maps_handlers.rs`) | `NetworkMetrics` | `GetDroppedPacketsMetrics` |
| `time_stamp_events` | `/sys/fs/bpf/trace_maps/time_stamp_events` | Metrics (`metrics/src/maps_handlers.rs`) | `TimeStampMetrics` | `GetLatencyMetrics` |
| `Blocklist` | `/sys/fs/bpf/maps/blocklist_map` | Identity (seeded from the `cortexbrain-client-config` ConfigMap) | `[u8;4] -> [u8;4]` | `AddIpToBlocklist` / `CheckBlocklist` / `RmIpFromBlocklist` |

!!! warning "Known caveat - pinned map paths"
    The Agent hardcodes the paths `/sys/fs/bpf/maps/...` and `/sys/fs/bpf/trace_maps/...` (`core/api/src/api.rs`), while the Identity and Metrics services pin under `${PIN_MAP_PATH}` which defaults to `/sys/fs/bpf/cortexbrain-identity-service/...`. This implies a deployment-time mount/symlink convention that is not captured in code. If you change the pin paths, update both sides.

## The `agent.Agent` service

The gRPC service is defined in `core/api/protos/agent.proto` (package `agent`, service name `Agent`). It exposes **6 unary RPC methods**. All methods are discoverable via server reflection.

| RPC method | Request | Response | Description |
|------------|---------|----------|-------------|
| `ActiveConnections` | `RequestActiveConnections { optional string pod_ip }` | `ActiveConnectionResponse { string status; repeated ConnectionEvent events }` | Returns the connection events drained from `events_map`. Each `ConnectionEvent` carries `event_id` (the PID), `src_ip_port`, and `dst_ip_port`. |
| `AddIpToBlocklist` | `AddIpToBlocklistRequest { optional string ip }` | `BlocklistResponse { string status; map<string,string> events }` | Inserts an IPv4 into the `Blocklist` BPF map. If `ip` is omitted, just returns the current blocklist. |
| `CheckBlocklist` | `google.protobuf.Empty` | `BlocklistResponse` | Reads the entire `Blocklist` BPF map and returns it as a `map<ip, ip>`. |
| `RmIpFromBlocklist` | `RmIpFromBlocklistRequest { string ip }` | `RmIpFromBlocklistResponse { string status; map<string,string> events }` | Removes an IPv4 from the `Blocklist` BPF map and returns the remaining entries. |
| `GetLatencyMetrics` | `google.protobuf.Empty` | `LatencyMetricsResponse { string status; repeated LatencyMetric metrics; uint32 total_count; double average_latency_us; double min_latency_us; double max_latency_us }` | Returns TCP connection latency metrics (`delta_us` per event, plus aggregate stats) drained from `time_stamp_events`. |
| `GetDroppedPacketsMetrics` | `google.protobuf.Empty` | `DroppedPacketsResponse { string status; repeated DroppedPacketMetric metrics; uint32 total_drops }` | Returns socket-level drop/error metrics (only entries where `sk_drops > 0`) drained from `net_metrics`. |

!!! note "Full message schema"
    The detailed field-by-field schema for every message (`ConnectionEvent`, `LatencyMetric`, `DroppedPacketMetric`, ...) lives in [`core/api/protos/agent.proto`](https://github.com/CortexFlow/CortexBrain/blob/main/core/api/protos/agent.proto). The [Integrated Metrics](metrics.md) page also describes the metric fields in tabular form.

## How `cfcli` calls the Agent

The CLI (`cli/`) is a tonic gRPC client. The mapping between `cfcli` commands and the Agent RPCs:

| `cfcli` command | Agent RPC |
|-----------------|-----------|
| `cfcli monitoring list` | (server reflection - lists all `agent.Agent` methods) |
| `cfcli monitoring connections` | `ActiveConnections` |
| `cfcli monitoring latencymetrics` | `GetLatencyMetrics` |
| `cfcli monitoring droppedpackets` | `GetDroppedPacketsMetrics` |
| `cfcli policy create-blocklist --flags <IP>` | `AddIpToBlocklist` |
| `cfcli policy check-blocklist` | `CheckBlocklist` |
| `cfcli policy remove-ip --flags <IP>` | `RmIpFromBlocklist` |

!!! note "Stale docs example"
    The `cfcli monitoring list` example output in the [Quick Start Guide](../cfcli/quick-start-guide.md) shows only `ActiveConnections`, but the proto actually defines 6 methods. The reflection output will list all of them.

## Relationship to the Identity service

The **Identity** service (`core/src/components/identity/`) is the **producer** of the connection events and the blocklist. It:

1. Loads the `conntracker` eBPF object.
2. Extracts and pins `EventsMap` (-> `events_map`), `Blocklist` (-> `blocklist_map`), and other maps.
3. Seeds the `Blocklist` map from the `cortexbrain-client-config` Kubernetes ConfigMap on startup.
4. Attaches the TC classifier and kprobes that fill the perf buffers.

The **Agent** never calls Identity directly - they communicate solely through the pinned BPF maps in `/sys/fs/bpf/`. The Agent is also a **mutator** of the `Blocklist` map that Identity owns: `AddIpToBlocklist` and `RmIpFromBlocklist` write to it at runtime, on top of the initial ConfigMap seeding.

## Security notes

The gRPC channel is currently **plaintext** HTTP/2:

- The server binds to `0.0.0.0:9090` (`core/api/src/main.rs`, flagged with a `FIXME`).
- The client connects to `http://127.0.0.1:9090` (`core/api/src/client.rs`, also flagged with a `FIXME`).

Both are known limitations to be addressed before production use. The `RequestActiveConnections.pod_ip` field is also currently read but unused by the server - all events are returned regardless of the requested pod IP.