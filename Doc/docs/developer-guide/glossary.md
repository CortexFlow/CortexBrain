# Glossary

This page defines the terms and acronyms used across the CortexBrain documentation. It is grouped by category so you can skim the area you are less familiar with.

## eBPF & kernel

| Term | Definition |
|------|------------|
| **eBPF** | Extended Berkeley Packet Filter. A virtual machine inside the Linux kernel that runs sandboxed programs in response to events (network packets, system calls, kprobes). CortexBrain uses eBPF to observe the cluster at kernel level without modifying the kernel source. |
| **BPF verifier** | The kernel component that statically analyzes a BPF program before execution, ensuring it terminates safely, does not access out-of-bounds memory, and meets security constraints. |
| **JIT** | Just-In-Time compiler. Translates verified BPF bytecode into native CPU instructions for maximum performance. |
| **kprobe** | Kernel probe. An eBPF attach point on a kernel function entry. CortexBrain uses kprobes on `tcp_v4_connect`, `tcp_rcv_state_process`, `register_netdevice`, `tcp_identify_packet_loss`, etc. |
| **uprobe** | User-space probe. An eBPF attach point on a userspace binary symbol. The planned GPU metrics use uprobes on `libcudart.so` symbols (`cudaMalloc`, `cudaFree`, ...). |
| **TC** | Traffic Control. The Linux kernel packet scheduler. CortexBrain attaches the `identity_classifier` eBPF program as a TC classifier on veth interfaces ingress. |
| **XDP** | eXpress Data Path. An eBPF attach point on the network driver, before the skb is allocated. Mentioned in the architecture diagram as a future hook. |
| **PerfEventArray** | A BPF map type that pushes events from kernel to user space via a perf ring buffer. CortexBrain uses it to stream `PacketLog`, `NetworkMetrics`, `TimeStampEvent` to the userspace loaders and the agent. |
| **BPF map** | A kernel data structure shared between eBPF programs and userspace. CortexBrain uses `PerfEventArray` (event streams) and `HashMap` (the blocklist). |
| **bpffs** | The BPF filesystem, mounted at `/sys/fs/bpf`. Pinned BPF maps live here as files. |
| **Pinning** | Persisting a BPF map to bpffs so it survives the loading process and can be opened by another process. The Identity and Metrics services pin maps; the agent opens them. |
| **Aya** | The Rust eBPF framework CortexBrain uses to load, attach, and read BPF maps. See [aya-rs.dev](https://aya-rs.dev/). |

## CortexBrain components

| Term | Definition |
|------|------------|
| **conntracker** | The eBPF kernel crate (`core/src/components/conntracker/`) that produces connection events via a TC classifier and TCP kprobes. |
| **identity** | The userspace service (`core/src/components/identity/`) that loads the conntracker eBPF object, pins `events_map` and `blocklist_map`, seeds the blocklist from the `cortexbrain-client-config` ConfigMap, and attaches the TC classifier to veths. |
| **metrics_tracer** | The eBPF kernel crate (`core/src/components/metrics_tracer/`) that produces socket-level metrics and TCP latency via kprobes. |
| **metrics** | The userspace service (`core/src/components/metrics/`) that loads the metrics_tracer eBPF object and pins `net_metrics` and `time_stamp_events`. |
| **agent** | The gRPC server crate (`core/api/`, binary `agent-api`) that opens the pinned maps, drains the perf buffers into mpsc channels, and serves the 6 `agent.Agent` RPCs on `:9090`. |
| **cfcli** | The Rust CLI client (`cli/`, binary `cfcli`) that calls the agent gRPC and manages the install/uninstall lifecycle via `kubectl`. |
| **dashboard** | The React/Electron frontend (`dashboard/`). Currently a static placeholder with no live data wiring. |
| **MCP server** | The Model Context Protocol server (`mcp/`, binary `mcp`) that exposes 6 tools querying Prometheus, so AI assistants (opencode, Claude) can read CortexBrain metrics. See [MCP Server](mcp-server.md). |

## Kubernetes

| Term | Definition |
|------|------------|
| **ClusterIP** | A Kubernetes Service type reachable only from inside the cluster. The `cortexflow-agent` Service is ClusterIP on port 9090. |
| **port-forward** | `kubectl port-forward` opens a tunnel from the local host to a Kubernetes Service. Used to reach `cortexflow-agent:9090` from `cfcli`. |
| **hostPath** | A volume type that mounts a path from the host node. CortexBrain mounts `/sys/fs/bpf`, `/proc`, `/lib/modules` this way. |
| **hostPID / hostNetwork** | Pod options that share the host PID and network namespaces. CortexBrain pods use both for eBPF visibility. |
| **privileged** | A security context flag that gives a container nearly all host capabilities. CortexBrain pods require it for BPF map and kprobe access. |
| **ConfigMap** | A Kubernetes object holding non-secret key-value data. `cortexbrain-client-config` stores the blocklist. |
| **ServiceAccount** | A Kubernetes identity for pods. CortexBrain deployments currently use the `default` ServiceAccount. |
| **RBAC** | Role-Based Access Control. The `configmap-reader` Roles grant the deployments read access to ConfigMaps. |

## Networking

| Term | Definition |
|------|------------|
| **veth** | Virtual Ethernet pair. The per-pod network interface created by Calico. CortexBrain attaches the `identity_classifier` TC classifier to veths. |
| **TC classifier** | A Traffic Control program that classifies packets. The `identity_classifier` returns `TC_ACT_OK` (pass) or `TC_ACT_SHOT` (drop, for blocklisted IPs). |
| **ingress / egress** | Incoming / outgoing traffic directions. The TC classifier is attached on ingress. |
| **TCP handshake** | The SYN -> SYN-ACK -> ACK exchange that establishes a TCP connection. The `tcp_connect` and `tcp_rcv_state_process` kprobes measure the time spent in `TCP_SYN_SENT` as the connection latency. |
| **`struct sock`** | The kernel structure representing a socket. The `metrics_tracer` kprobe reads its fields (`sk_drops`, `sk_err`, `sk_backlog_len`, ...) at hardcoded offsets. |

## Observability

| Term | Definition |
|------|------------|
| **OpenTelemetry (OTel)** | The CNCF standard for observability data. CortexBrain's `metrics-patch` branch adds 16 OTel instruments exported via OTLP. |
| **OTLP** | OpenTelemetry Protocol. The gRPC/HTTP transport for OTel data. The exporter is at `localhost:4317` (gRPC) or `localhost:4318` (HTTP). |
| **Prometheus** | A metrics database that scrapes and stores time-series. The MCP server queries it at `localhost:9090`. |
| **Counter** | An OTel instrument that only increases (e.g. `events_total`). |
| **Gauge** | An OTel instrument that can go up or down (e.g. `gpu_memory_used`). |
| **Histogram** | An OTel instrument that records a distribution (e.g. `latency_us`, `cuda_memcpy_bytes`). |
| **mpsc channel** | Multi-producer, single-consumer channel from the Rust `tokio` library. The agent uses 3 mpsc channels (capacities 1024/2048/2048) to move events from the reader tasks to the gRPC handlers. |

## GPU

| Term | Definition |
|------|------------|
| **CUDA** | NVIDIA's parallel computing platform. The planned GPU metrics trace the CUDA runtime API. |
| **libcudart.so** | The CUDA runtime library. The planned eBPF uprobes attach to its exported symbols. |
| **NVML** | NVIDIA Management Library. Provides device-level metrics (utilization, memory, temperature, power, ECC) via userspace polling. |
| **cudaMalloc / cudaFree** | CUDA runtime API calls to allocate / free device memory. Tracked as `cuda_malloc_total`, `cuda_malloc_bytes`, `cuda_free_total`, `cuda_free_bytes`. |
| **cudaMemcpy** | CUDA runtime API call to copy memory between host and device. Direction is tagged `H2D`, `D2H`, `D2D`. |
| **kernel launch** | Scheduling a CUDA kernel for execution via `cudaLaunch` / `cudaKernelLaunch`. Tracked as `cuda_kernel_launch_total` and `cuda_kernel_duration_us`. |
| **ECC** | Error-Correcting Code memory on GPUs. The `gpu_ecc_errors` counter tracks ECC error counts. |

## Model Context Protocol (MCP)

| Term | Definition |
|------|------------|
| **Model Context Protocol** | The Anthropic standard for exposing tools, resources, and prompts to LLM-based assistants. |
| **stdio transport** | The MCP transport that uses the process stdin/stdout for JSON-RPC. The CortexBrain MCP server uses this. |
| **tool** | An MCP capability that lets the assistant call a function. The CortexBrain MCP server exposes 6 tools querying Prometheus. |
| **resource** | An MCP capability that lets the assistant read data. CortexBrain does not currently expose resources. |
| **prompt** | An MCP capability that lets the assistant use a templated prompt. CortexBrain does not currently expose prompts. |
| **ServerInfo** | The MCP `initialize` response containing the server name, version, and capabilities. |
| **ToolRouter** | The `rmcp` component that dispatches `tools/call` requests to the matching handler method. |
| **rmcp** | The Rust MCP SDK crate (v0.8.5) used by the CortexBrain MCP server. |