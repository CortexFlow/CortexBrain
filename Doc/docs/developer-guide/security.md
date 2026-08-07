# Security

This page documents the current security posture of CortexBrain and the roadmap for hardening the data plane and the control plane. CortexBrain is still in active development; several components ship with `FIXME` markers indicating known limitations to address before production use.

## Current state

### gRPC agent (plaintext, no auth)

The `agent.Agent` gRPC service (`core/api/`) currently runs without transport security or authentication:

- The server binds to `0.0.0.0:9090` (`core/api/src/main.rs`, flagged `FIXME: binding on 0.0.0.0 address is not ideal for a production environment`).
- The `cfcli` client connects to `http://127.0.0.1:9090` over plaintext HTTP/2 (`core/api/src/client.rs`, flagged `FIXME: this will require an update to ensure a protected connection`).
- There is no TLS, no mTLS, and no token-based authentication on any RPC.

This means any client that can reach the agent port can call all 6 RPCs, including the blocklist mutators (`AddIpToBlocklist`, `RmIpFromBlocklist`).

### MCP server (plaintext HTTP to Prometheus)

The [MCP server](mcp-server.md) queries Prometheus at the hardcoded URL `http://localhost:9090/api/v1/query`. The Prometheus connection is plaintext HTTP and unauthenticated. The MCP server itself runs as a local stdio child process spawned by the MCP host (e.g. opencode), so the JSON-RPC stream never leaves the host.

### eBPF and kernel access

The `cortexflow-agent`, `cortexflow-identity`, and `cortexflow-metrics` pods run with:

- `hostPID: true`, `hostNetwork: true`
- `privileged: true` with the `BPF`, `SYS_ADMIN`, `NET_ADMIN`, `SYS_PTRACE`, and `SYS_RESOURCE` capabilities
- Bidirectional hostPath mounts of `/sys/fs/bpf`, `/proc`, and `/lib/modules`

This is required for eBPF map access and kprobe attachment, but it grants broad host access. The deployments must be restricted to a dedicated, trusted node pool in production.

### ConfigMap side-channel

The `cortexbrain-client-config` ConfigMap (namespace `cortexflow`) stores the blocklist as a newline-separated list of IPs. Both `cfcli` and the Identity service read and write it. The RBAC `configmap-reader` Role grants `get`/`list` on ConfigMaps in the `cortexflow` namespace to the `default` ServiceAccount used by the deployments.

## Deployment mitigations

The current state is acceptable for a local development cluster, but the following mitigations reduce risk in a shared environment:

| Risk | Mitigation |
|------|-----------|
| Agent reachable from any pod in the cluster | The `cortexflow-agent` Service is `ClusterIP` only; apply a [NetworkPolicy](https://kubernetes.io/docs/concepts/services-networking/network-policies/) to restrict ingress to port 9090 from trusted namespaces only |
| Agent bound to `0.0.0.0` | Constrain the bind address in `core/api/src/main.rs` to `127.0.0.1` or a specific interface (FIXME in code) |
| `cfcli` port-forward exposes the agent on the developer host | The port-forward is SSH-like and bound to `127.0.0.1`; do not share it over the network |
| Privileged pods with hostPID/hostNetwork | Taint the GPU/eBPF nodes and use `nodeSelector` / `nodeAffinity` to schedule CortexBrain pods only on dedicated nodes |
| ConfigMap stores IPs in clear | IPs are not secrets, but if the blocklist becomes sensitive, encrypt the ConfigMap with an external tool (e.g. [SOPS](https://github.com/getsops/sops)) or use a Secret |

## Roadmap

The following hardening steps are planned:

### Transport security

- **TLS on the agent gRPC**: the [tonic](https://docs.rs/tonic/) stack supports TLS natively via `rustls` or `openssl`. The plan is to add a `TlsAcceptor` to the agent server and a `TlsConnector` to the `cfcli` client, with certificates mounted from a Kubernetes Secret.
- **mTLS for client/server authentication**: mutual TLS so the agent only accepts connections from `cfcli` (or the MCP server) presenting a known client certificate. This also enables per-client identity for future RBAC.
- **TLS for OTLP export**: the OpenTelemetry exporter (incoming on `metrics-patch`) supports TLS to the collector; the collector endpoint should use `https://` in production.

### Authentication and authorization

- **Token-based auth on the agent gRPC**: a bearer token or JWT validated in a tonic interceptor, so that read-only consumers (dashboard, MCP server) and write consumers (cfcli policy) can be distinguished.
- **RBAC on the agent RPCs**: `ActiveConnections`, `GetLatencyMetrics`, `GetDroppedPacketsMetrics`, `CheckBlocklist` are read-only; `AddIpToBlocklist`, `RmIpFromBlocklist` are mutators. The plan is to require an elevated token for the mutator RPCs.
- **Kubernetes ServiceAccount per component**: today all deployments use the `default` ServiceAccount. A dedicated SA per component with minimal RBAC (the Identity service needs ConfigMap read; the agent needs nothing beyond BPF map access) reduces blast radius.

### Hardening

- **Restrict the agent bind address** from `0.0.0.0` to `127.0.0.1` (or a Unix domain socket in a shared volume) - already flagged as a `FIXME`.
- **NetworkPolicy** in the `cortexflow` namespace to limit which pods can reach `cortexflow-agent:9090`.
- **cert-manager** integration to rotate the TLS certificates automatically.
- **Drop unused capabilities** once the eBPF loader no longer needs `SYS_PTRACE` (only required for the `metrics_tracer` kprobe on `tcp_rcv_state_process` if the kernel lacks BTF support).
- **Prometheus auth**: when the MCP server gains a configurable Prometheus URL (currently hardcoded), add support for bearer-token or basic-auth headers.

## Reporting vulnerabilities

Security vulnerabilities must **not** be reported via public GitHub issues. Email `lorenzotettamanti5@gmail.com` or `lorenzolollobrada@gmail.com` directly (see `SECURITY.md` in the repo root).

## See also

- [Agent API Overview](agent-api.md) - the gRPC service and the `FIXME` markers on the binding and client.
- [MCP Server](mcp-server.md) - the Prometheus HTTP client and the hardcoded URL.
- [Architecture Overview](architecture.md) - the deployment topology and pod capabilities.
- [Development Goals & Discussions](discussions.md) - the roadmap tracks.