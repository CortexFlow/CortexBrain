# Agent API Overview

This page documents the CortexFlow Agent gRPC API for versions >= 0.1.3.

The Agent API is the interface used by cfcli to query runtime networking data and manage selected control-plane actions such as blocklist operations.

## Source of truth

- API contract: [core/api/protos/agent.proto](https://github.com/CortexFlow/CortexBrain/blob/main/core/api/protos/agent.proto)
- API implementation: [core/api/src/api.rs](https://github.com/CortexFlow/CortexBrain/blob/main/core/api/src/api.rs)
- Server startup and reflection registration: [core/api/src/main.rs](https://github.com/CortexFlow/CortexBrain/blob/main/core/api/src/main.rs)

## Quick access

### Local access

Start the Agent API server and target port 9090.

```bash
grpcurl -plaintext 127.0.0.1:9090 list
grpcurl -plaintext 127.0.0.1:9090 describe agent.Agent
```

### Kubernetes access

```bash
kubectl port-forward svc/cortexflow-agent 9090:9090 -n cortexflow
grpcurl -plaintext 127.0.0.1:9090 list
```

## grpcui documentation

The server exposes gRPC Reflection, so grpcui can discover services and message types without requiring local proto files.

### Install grpcui

```bash
# macOS
brew install grpcui

# Linux
go install github.com/fullstorydev/grpcui/cmd/grpcui@latest
```

### Run grpcui

```bash
grpcui -plaintext 127.0.0.1:9090
```

grpcui starts a local web app and opens an interactive page where you can:

- Browse the agent.Agent service and all RPC methods
- Fill request payloads in JSON form
- Invoke methods and inspect raw responses

## Service summary

Package: agent

Service: Agent

| RPC | Request | Response | Purpose |
| --- | --- | --- | --- |
| ActiveConnections | RequestActiveConnections | ActiveConnectionResponse | Returns recently collected connection events |
| AddIpToBlocklist | AddIpToBlocklistRequest | BlocklistResponse | Inserts an IPv4 address into the pinned eBPF blocklist map |
| CheckBlocklist | google.protobuf.Empty | BlocklistResponse | Returns current blocklist entries |
| RmIpFromBlocklist | RmIpFromBlocklistRequest | RmIpFromBlocklistResponse | Removes an IPv4 address from blocklist |
| GetLatencyMetrics | google.protobuf.Empty | LatencyMetricsResponse | Returns buffered latency metrics and computed aggregates |
| GetDroppedPacketsMetrics | google.protobuf.Empty | DroppedPacketsResponse | Returns buffered dropped packet metrics and total drops |
| GetTrackedVeth | google.protobuf.Empty | VethResponse | Returns tracked veth names from recent events |
| GetTrackedVethFromHashMap | google.protobuf.Empty | VethHashMapResponse | Returns tracked veth entries from pinned eBPF hash map |

## Message reference

### Active connections

- RequestActiveConnections
    - pod_ip (optional string): currently accepted but not used for server-side filtering.

- ConnectionEvent
    - event_id (string)
    - src_ip_port (string, example 192.168.1.10:54321)
    - dst_ip_port (string, example 10.0.0.12:443)

- ActiveConnectionResponse
    - status (string)
    - events (repeated ConnectionEvent)

### Blocklist

- AddIpToBlocklistRequest
    - ip (optional string, IPv4)

- BlocklistResponse
    - status (string)
    - events (map<string,string>)

- RmIpFromBlocklistRequest
    - ip (string, IPv4)

- RmIpFromBlocklistResponse
    - status (string)
    - events (map<string,string>)

### Metrics

- LatencyMetric
    - delta_us, timestamp_us, tgid, process_name, local_port, remote_port, address_family
    - src_address_v4, dst_address_v4, src_address_v6, dst_address_v6

- LatencyMetricsResponse
    - status
    - metrics
    - total_count
    - average_latency_us
    - min_latency_us
    - max_latency_us

- DroppedPacketMetric
    - tgid, process_name, sk_drops, sk_err, sk_err_soft
    - sk_backlog_len, sk_wmem_queued, sk_rcvbuf, sk_ack_backlog
    - timestamp_us

- DroppedPacketsResponse
    - status
    - metrics
    - total_drops

### Veth

- VethResponse
    - status
    - veth_names (repeated string)
    - tot_monitored_veth (int32)

- VethHashMapResponse
    - status
    - veths (map<string,string>)

## Runtime behavior notes from implementation

The following behavior is based on current code in core/api/src/api.rs:

- Responses are produced from buffered event channels, so each call returns currently queued events at call time.
- If no matching events are queued, successful responses can contain empty arrays/maps.
- Blocklist and tracked-veth map operations use pinned eBPF maps under /sys/fs/bpf/maps.
- Several map and parse operations currently use unwrap or expect, so malformed input or missing pinned maps can cause internal failures.

## Example grpcurl calls

```bash
# List services discovered via reflection
grpcurl -plaintext 127.0.0.1:9090 list

# Show full service schema
grpcurl -plaintext 127.0.0.1:9090 describe agent.Agent

# Query active connections
grpcurl -plaintext -d '{"pod_ip":""}' 127.0.0.1:9090 agent.Agent/ActiveConnections

# Add an IP to blocklist
grpcurl -plaintext -d '{"ip":"10.0.0.25"}' 127.0.0.1:9090 agent.Agent/AddIpToBlocklist

# Check blocklist
grpcurl -plaintext -d '{}' 127.0.0.1:9090 agent.Agent/CheckBlocklist

# Remove from blocklist
grpcurl -plaintext -d '{"ip":"10.0.0.25"}' 127.0.0.1:9090 agent.Agent/RmIpFromBlocklist

# Latency metrics
grpcurl -plaintext -d '{}' 127.0.0.1:9090 agent.Agent/GetLatencyMetrics

# Dropped packets metrics
grpcurl -plaintext -d '{}' 127.0.0.1:9090 agent.Agent/GetDroppedPacketsMetrics

# Tracked veth events
grpcurl -plaintext -d '{}' 127.0.0.1:9090 agent.Agent/GetTrackedVeth

# Tracked veth map
grpcurl -plaintext -d '{}' 127.0.0.1:9090 agent.Agent/GetTrackedVethFromHashMap
```

## Known limitations

- pod_ip in ActiveConnections is currently not used for filtering.
- Some response status fields are generic and do not encode detailed failure causes.
- Reflection is controlled by the AGENT_API_ENABLE_REFLECTION environment variable and should be considered primarily a debugging and development feature.
