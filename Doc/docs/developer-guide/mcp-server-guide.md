# MCP Server

CortexBrain ships a **Model Context Protocol (MCP) server** that exposes its eBPF-derived metrics to AI assistants (opencode, Claude Desktop, etc.) so they can query the cluster state in natural language. The server is a thin stdio JSON-RPC process that translates tool calls into PromQL queries against a local Prometheus.

!!! warning "Not yet on `main` / `0.1.5-docs`"
    The MCP server code lives on the `origin/0.1.5` and `origin/metrics-patch` branches (byte-identical between the two). It is **not** merged into the `main` or `0.1.5-docs` branches as of this writing. This page documents the existing implementation so contributors can try it, review it, and plan its integration.

## What is the Model Context Protocol?

The [Model Context Protocol](https://modelcontextprotocol.io/) is an open standard (originated by Anthropic) for exposing **tools**, **resources**, and **prompts** to LLM-based assistants over a standard JSON-RPC transport. An MCP host (opencode, Claude Desktop, Cursor) spawns an MCP server as a local child process and calls its tools when the model needs to interact with an external system. CortexBrain uses MCP to let an assistant query Prometheus metrics collected from eBPF.

## The CortexBrain MCP server

| Property | Value |
|----------|-------|
| Crate | `mcp` (`mcp/Cargo.toml`) |
| Binary | `mcp` (built to `mcp/target/release/mcp`) |
| Version | 0.1.0 |
| Server name (MCP `Initialize`) | `cortexflow-mcp` |
| Server title | `Cortexflow MCP Server` |
| MCP SDK | `rmcp` 0.8.5 (features: `server`, `client`, `transport-io`, `transport-child-process`) |
| Transport | stdio (JSON-RPC over the process stdin/stdout) |
| Capabilities | tools only (no resources, no prompts) |
| Backing store | Prometheus HTTP query API at `http://localhost:9090/api/v1/query` (hardcoded) |

The server does **not** call the `agent.Agent` gRPC service, does **not** read BPF maps, and does **not** invoke `kubectl`. It is purely a thin HTTP client in front of Prometheus, sitting downstream of the OTLP collector that exports the eBPF metrics.

## Architecture

```
AI assistant (opencode, Claude Desktop, Cursor)
        │ stdio JSON-RPC (MCP)
        ▼
mcp binary (mcp/target/release/mcp)
        │ HTTP GET /api/v1/query?query=<PromQL>
        ▼
Prometheus (localhost:9090)
        │ scrape (Prometheus pull)
        ▼
OTLP collector (cortexbrain namespace prefix)
        ▲ OTLP/gRPC (localhost:4317)
        │
CortexBrain metrics service  ──► eBPF perf buffers  ──► kernel probes
```

The MCP server is a **local standalone binary**, not a Kubernetes workload and not a sidecar. The operator builds it and configures an MCP host to spawn it; the host owns the process lifecycle.

## The 6 tools

All 6 tools share the same input schema:

```json
{
  "type": "object",
  "properties": {
    "container_name": { "type": "string" },
    "timeframe":       { "type": "string" }
  },
  "required": ["container_name", "timeframe"]
}
```

- `container_name` - a regex substring matched against the Prometheus `container_name` label (e.g. `"grafana"`).
- `timeframe` - a PromQL range-vector duration (e.g. `"10m"`).

Each tool runs a `sum by(container_name) (rate(...[timeframe]))` query and returns the Prometheus JSON response as a pretty-printed string.

| Tool name | Description | PromQL metric | OTel instrument family |
|-----------|-------------|----------------|------------------------|
| `get_cpu_bytes` | CPU bytes allocation per event | `cortexbrain_cpu_bytes_alloc` | gauge |
| `get_memory_allocated_bytes` | Bytes requested via `mmap` syscalls | `cortexbrain_enter_mem_alloc` | gauge |
| `get_events` | Total eBPF events processed across all perf buffers | `cortexbrain_events_total` | counter |
| `get_l4_events` | Total socket state events processed | `cortexbrain_socket_events_total` | counter |
| `get_ssl_write_events` | Total bytes requested by `ssl_write` | `cortexbrain_ssl_write_bytes` | gauge |
| `get_ssl_read_events` | Total bytes requested by `ssl_read` | `cortexbrain_ssl_read_bytes` | gauge |

The `cortexbrain_` prefix comes from the OTLP collector's Prometheus exporter `namespace: cortexbrain` setting (see `Examples/run-with-docker/otel-collector-config.yaml`). The base instrument names (`cpu_bytes_alloc`, `events_total`, ...) are defined in `core/common/src/semantic.rs` on the `metrics-patch` branch.

## Configuration in opencode

The MCP server is configured in `~/.config/opencode/opencode.jsonc` (or the project-local `.opencode/opencode.jsonc`):

```jsonc
{
  "mcp": {
    "cortexflow-mcp": {
      "type": "local",
      "command": ["/home/<user>/CortexBrain/mcp/target/release/mcp"],
      "enabled": true
    }
  }
}
```

- `"type": "local"` + the `command` array tells opencode to spawn the binary as a child process and speak MCP over its stdin/stdout.
- `"enabled": true` auto-starts the server when opencode launches.
- The `"environment"` block (commented out in the example) can inject env vars into the child process - currently unused because the Prometheus URL is hardcoded.

When the assistant needs a metric, opencode sends a `tools/call` request with the tool name and arguments; the server runs the PromQL and returns the JSON result as `TextContent`.

## Building the server

From the repository root (the `mcp/` crate must be checked out - it is on `origin/0.1.5` and `origin/metrics-patch`):

```bash
cargo build --release -p mcp
```

The binary is emitted at `mcp/target/release/mcp`. Point your MCP host's `command` at that absolute path.

!!! note "Heavy transitive build"
    `mcp/Cargo.toml` declares `cortexbrain-common = { path = "../core/common" }` as a dependency, but the source never imports it. This pulls in the entire `cortexbrain-common` transitive tree (`aya`, `kube`, `k8s-openapi`, `opentelemetry-otlp`, `tonic`) and bloats the build. The `genai` and `dotenv` crates are also declared but unused. Dropping these dependencies is a pending cleanup task.

## Caveats

!!! warning "Known limitations"
    - **Hardcoded Prometheus URL**: `prometheus.rs` sets `const PROMETHEUS_SERVER: &str = "http://localhost:9090/api/v1/query";`. There is no env-var or CLI flag override. You must run Prometheus on `localhost:9090` (e.g. via `kubectl port-forward svc/prometheus 9090:9090` or the `Examples/run-with-docker/docker-compose.yaml` stack).
    - **Error handling**: every tool handler ends with `.expect("An error occured")` (note the typo). Any HTTP or JSON failure panics the Tokio task instead of returning a clean MCP error response. A Prometheus outage crashes the tool call.
    - **PromQL injection**: `container_name` and `timeframe` are interpolated into the PromQL string via `format!` with no escaping. A `container_name` containing `"` or `}` would break or alter the query.
    - **No agent gRPC surface**: the MCP server does not expose the `agent.Agent` RPCs (`ActiveConnections`, blocklist operations, etc.). It only reads Prometheus, so it cannot return live connection events or mutate the blocklist.

## Roadmap

- **Configurable Prometheus URL** via env var (e.g. `PROMETHEUS_URL`).
- **Proper error handling**: return MCP error responses instead of panicking.
- **Expose the agent gRPC RPCs as MCP tools**: `ActiveConnections`, `CheckBlocklist`, `GetLatencyMetrics`, `GetDroppedPacketsMetrics` as read-only tools; `AddIpToBlocklist` / `RmIpFromBlocklist` as mutator tools (with elevated auth - see [Security](security.md)).
- **Add MCP resources** for the BPF map state (e.g. the current blocklist, the pinned map inventory).
- **Drop unused dependencies** (`genai`, `dotenv`, `cortexbrain-common`) to slim the build.
- **TLS / auth** for the Prometheus connection when the URL is configurable.

## See also

- [Architecture Overview](architecture.md) - the four-stage pipeline and where Prometheus sits.
- [Integrated Metrics](metrics.md) - the OpenTelemetry instruments that feed Prometheus.
- [Agent API Overview](agent-api.md) - the gRPC service the MCP server currently does *not* call (planned for the roadmap).
- [Security](security.md) - the plaintext Prometheus connection and the hardening roadmap.
- [Troubleshooting](troubleshooting.md) - "MCP server cannot reach Prometheus".