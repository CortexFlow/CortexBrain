# Developer Guide

!!! note
    CortexBrain is currently in active development. Resources on eBPF and the Aya Rust framework are sparse across the internet, and existing references such as [ebpf.io](https://ebpf.io) and [aya-rs.dev](https://aya-rs.dev) only cover the top of the iceberg. This Developer Guide aims to flatten the learning curve so that newcomers can easily learn, contribute and become maintainers of the project.

## Why this section exists
The project has a very steep curve because of the combination of eBPF and Rust at the same time. 
This section consolidates the architecture, the Agent API surface, the integrated metrics, and the contribution workflow into a single, detailed reference so that you don't have to guess what the codebase does to get started.

## What you'll find here

| Page | What it covers |
|------|----------------|
| [Glossary](glossary.md) | Definitions for eBPF, kernel, Kubernetes, networking, observability, GPU, and MCP terms used across the docs |
| [Architecture Overview](architecture.md) | Covers kernel instrumentation, maps pinning, aggregation and consumption |
| [Development Workflow](dev-workflow.md) | Covers the complete developer workflow: Fork, development branch and PR flow, CI pipeline, commit and branch conventions |
| [Agent API Overview](agent-api.md) | The Cortexflow agent service. Covers the RPC methods, the BPF maps the agent reads, and the data flow from eBPF kernel programs to `cfcli` |
| [Integrated Metrics](metrics.md) | Covers the live metrics and the OpenTelemetry metric format from `metrics-patch` |
| [MCP Server](mcp-server.md) | The MCP server that exposes CortexBrain metrics to AI assistants via Prometheus queries |
| [GPU Metrics (Upcoming)](gpu-metrics.md) | The complete CUDA observability roadmap |
| [Troubleshooting](troubleshooting.md) | Common issues (BPF fs not mounted, pinned map path mismatch, port-forward, permissions) and their fixes |
| [Development Goals & Discussions](discussions.md) | Milestones, roadmap, GitHub Discussions, labels, and how to propose new features |

## Prerequisites

Make sure you have the development environment set up. The guided setup (Rust nightly, eBPF toolchain, Minikube/Kind, Calico CNI, Docker setup) is documented in the [Getting Started for developers](../getting-started/installation.md#getting-started-for-developers) section of the installation page.

The core build requirements (kernel `>= 5.15`, `bpftool`, `bcc`, `clang`, `llvm`, `libbpf-dev`, `rustc >= 1.85.0` nightly) are also listed there.

## External resources

CortexBrain sits at the intersection of Rust and eBPF no single resource covers everything at the same time, in the same repository. The following are the best starting points beyond this guide:

- **eBPF** - [ebpf.io](https://ebpf.io/what-is-ebpf/) (concept overview), the [Cilium eBPF documentation](https://docs.cilium.io/) (practical kernel hook reference), and the [BPF and XDP reference guide](https://docs.kernel.org/networking/filter.html) in the Linux kernel docs.
- **Aya (Rust eBPF)** - [aya-rs.dev](https://aya-rs.dev/) (the framework CortexBrain uses to load and pin BPF maps), the [Aya book](https://aya-rs.dev/book/), and the [Aya examples](https://github.com/aya-rs/aya/tree/main/examples).
- **gRPC / tonic** - the [tonic](https://docs.rs/tonic/) Rust gRPC stack documentation, which powers the `agent.Agent` service.
- **Kubernetes networking** - the [Kubernetes CNI documentation](https://kubernetes.io/docs/concepts/extend-kubernetes/compute-storage-network/network-plugins/) and the [Calico docs](https://docs.tigera.io/calico/latest/about), since CortexBrain attaches eBPF programs to veth interfaces created by Calico.
- **Docker docs** - [Docker docs](https://docs.docker.com/)

## Contributing

CortexBrain is open source under Apache 2.0. We actively look for contributors and collaborators. If you have knowledge in DevOps,GPUs, Kubernetes,Docker or networking.