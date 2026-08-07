# GPU Metrics (Upcoming)

This page documents the **upcoming GPU observability metrics** that CortexBrain will introduce to support AI/ML workloads. The focus is on tracing the CUDA runtime API (`cudaMalloc`, `cudaFree`, `cudaMemcpy`, kernel launches) and on collecting GPU device-level metrics via NVML.

!!! warning "Upcoming - not yet implemented"
    No GPU/CUDA code, eBPF probe, or OpenTelemetry instrument exists in the repository on any branch as of this writing. This page is a **technical specification** of what will be built, aligned with the "GPU observability" experimental track on the [roadmap](discussions.md). It is documented in advance so contributors can plan and discuss the design.

## Tracing architecture

The GPU metrics pipeline combines two complementary mechanisms:

```
┌──────────────────────────────────────────────────────────────────────┐
│  Application container (AI/ML workload)                              │
│                                                                        │
│   libcudart.so  ── cudaMalloc / cudaFree / cudaMemcpy / cudaLaunch     │
│        │                                                              │
│        ▼  (user-space uprobes)                                        │
│   eBPF uprobe programs ──► PerfEventArray ──► agent (core/api)        │
│   (attached to libcudart.so symbols)                                  │
│                                                                        │
└──────────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────────┐
│  GPU device (NVIDIA)                                                 │
│                                                                        │
│   NVML (NVIDIA Management Library)                                    │
│        │                                                              │
│        ▼  (userspace polling, Rust binding e.g. nvml-wrapper)         │
│   agent (core/api) ── reads utilization / memory / temp / power / ECC │
│                                                                        │
└──────────────────────────────────────────────────────────────────────┘

        │  both feed the agent
        ▼
   agent.Agent gRPC service ──(future)──► OTLP exporter
   (extension of the existing metrics-patch OTel pipeline)
```

- **eBPF uprobes** capture every call to the CUDA runtime API at the userspace symbol boundary, with near-zero overhead and no application modification. This mirrors the approach already used by CortexBrain for kernel functions (kprobes) and aligns with the existing eBPF + Aya model.
- **NVML** provides device-level hardware counters that are not observable from the API surface alone (utilization, temperature, power, ECC errors). NVML is a userspace library polled by the agent at a configurable interval.

See the [Architecture Overview](architecture.md) for how the existing pipeline (eBPF -> pinned maps -> agent -> gRPC/OTLP) is structured.

## CUDA runtime API metrics (eBPF uprobes)

These metrics are produced by uprobes attached to the exported symbols of `libcudart.so`. Each uprobe captures the call arguments and emits an event to a perf buffer; the agent aggregates them and exposes them as OpenTelemetry instruments.

| Instrument name | Type | Description |
|-----------------|------|-------------|
| `cuda_malloc_total` | `Counter<u64>` | Total number of `cudaMalloc` calls |
| `cuda_malloc_bytes` | `Counter<u64>` | Total bytes allocated via `cudaMalloc` |
| `cuda_free_total` | `Counter<u64>` | Total number of `cudaFree` calls |
| `cuda_free_bytes` | `Counter<u64>` | Total bytes freed via `cudaFree` |
| `cuda_memcpy_total` | `Counter<u64>` | Total number of `cudaMemcpy` calls |
| `cuda_memcpy_bytes` | `Histogram<u64>` | Bytes transferred per `cudaMemcpy`, tagged by direction (`H2D`, `D2H`, `D2D`) |
| `cuda_memcpy_duration_us` | `Histogram<u64>` | Duration of each `cudaMemcpy` in microseconds |
| `cuda_kernel_launch_total` | `Counter<u64>` | Total number of `cudaLaunch` / `cudaKernelLaunch` calls |
| `cuda_kernel_duration_us` | `Histogram<u64>` | Duration of each kernel execution in microseconds |

## GPU device metrics (NVML)

These metrics are polled from the GPU device via the NVIDIA Management Library. They describe the hardware state rather than the API call stream.

| Instrument name | Type | Description |
|-----------------|------|-------------|
| `gpu_utilization` | `Gauge<i64>` | GPU utilization percentage (0-100) |
| `gpu_memory_used` | `Gauge<i64>` | Memory currently in use (bytes) |
| `gpu_memory_total` | `Gauge<i64>` | Total memory capacity (bytes) |
| `gpu_temperature` | `Gauge<i64>` | GPU temperature in Celsius |
| `gpu_power_usage` | `Gauge<i64>` | Power draw in milliwatts |
| `gpu_ecc_errors` | `Counter<u64>` | Total ECC error count |

## Common attributes (labels)

All GPU instruments share the following attribute set, mirroring the convention used by the existing OpenTelemetry metrics (see [Integrated Metrics](metrics.md#common-attributes-labels)):

| Attribute | Description |
|-----------|-------------|
| `gpu_id` | GPU index on the node (0, 1, 2, ...) |
| `gpu_uuid` | NVIDIA GPU UUID (e.g. `GPU-xxxxxxxx-...`) |
| `gpu_name` | GPU model name (e.g. `NVIDIA A100-SXM4-40GB`) |
| `tgid` | Thread group ID (process) - applies to uprobe metrics |
| `process_name` | Process name (`comm`) - applies to uprobe metrics |
| `container.name` | Container name |
| `container.id` | Container ID (optional) |
| `k8s.pod.name` | Pod name (optional) |
| `k8s.namespace.name` | Namespace name (optional) |
| `cuda_memcpy_kind` | Transfer direction for `cudaMemcpy` metrics: `H2D`, `D2H`, `D2D`, `H2H` |

## Target uprobes

The eBPF uprobe loader will attach to the following exported symbols in `libcudart.so`:

| Symbol | Library | Attach type | Data captured |
|--------|---------|-------------|--------------|
| `cudaMalloc` | `libcudart.so` | uprobe (entry + return) | pointer, size; duration |
| `cudaFree` | `libcudart.so` | uprobe (entry + return) | pointer; duration |
| `cudaMemcpy` | `libcudart.so` | uprobe (entry + return) | dst, src, count, kind (H2D/D2H/D2D); duration |
| `cudaLaunch` | `libcudart.so` | uprobe (entry + return) | func, grid, block; duration |
| `cudaKernelLaunch` | `libcudart.so` | uprobe (entry + return) | kernel args; duration |

The entry probe captures the arguments; the return probe computes the duration. This is the same entry/return pairing pattern used by the existing `tcp_connect` / `tcp_rcv_state_process` kprobes in `metrics_tracer`.

## Export

The GPU metrics will be exported through the same OpenTelemetry pipeline as the existing `metrics-patch` instrument suite:

- **Meter name**: `cortexbrain-gpu-metrics` (sibling of `cortexbrain-metrics`)
- **Exporter**: OTLP, via gRPC (`http://localhost:4317`) or HTTP (`http://localhost:4318`)
- **Reader**: `PeriodicReader` with a 5-second export interval

See the [Integrated Metrics - OpenTelemetry pipeline](metrics.md#opentelemetry-metrics-incoming-metrics-patch) section for the exporter configuration details.

## Implementation roadmap

The following steps are planned to bring this from specification to integration:

- [ ] eBPF uprobe loader for `libcudart.so` symbols (new module under `core/src/components/metrics_tracer/src/`, likely `cuda.rs`)
- [ ] Perf buffer structs for CUDA events (`CudaMallocEvent`, `CudaFreeEvent`, `CudaMemcpyEvent`, `CudaKernelLaunchEvent`) in `data_structures.rs`
- [ ] Rust NVML binding integration (e.g. the [`nvml-wrapper`](https://crates.io/crates/nvml-wrapper) crate) into the agent or a dedicated `gpu-metrics` userspace component
- [ ] OTel instrument definitions in `core/common/src/` (new `gpu_metrics.rs` / `gpu_semantic.rs` modules, mirroring the existing `otel_metrics.rs` / `semantic.rs`)
- [ ] Agent wiring: open the CUDA perf buffers, poll NVML, expose via gRPC (new RPCs or extend the existing `Agent` service)
- [ ] K8s deployment: node-affinity for GPU nodes, `nvidia.com/gpu` resource requests, hostPath mounts for `/dev/nvidia*` and NVML

## See also

- [Integrated Metrics](metrics.md) - the live eBPF/gRPC metrics and the incoming OpenTelemetry suite
- [Architecture Overview](architecture.md) - the four-stage pipeline (kernel -> maps -> agent -> consumption)
- [Development Goals & Discussions](discussions.md) - the "GPU observability" roadmap track