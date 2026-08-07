# Development Goals & Discussions

This page collects the project milestones, the roadmap, and the GitHub references that shape CortexBrain development. Use it to find where to contribute and how to propose new work.

## Current milestones

The development stage of every component is tracked in the [home page](../index.md#current-development-stage). The current milestones are:

| Component | Stage | Branch | Milestone |
|-----------|-------|--------|-----------|
| Dashboard | Under development | `feature/frontend` | CortexBrain v 0.1.0 Launch |
| Identity Service | Under development | `feature/ebpf-core` | CortexBrain core v 0.1.0 |
| Agent | Under development | `feature/core` | CortexBrain core v 0.1.0 |
| CLI | Under development | `feature/cli` | CortexBrain CLI v 0.1.0 |

The Kubernetes-native "Core" milestone is tracked as [GitHub milestone #1](https://github.com/CortexFlow/CortexBrain/milestone/1). The core eBPF components (conntracker, identity, metrics_tracer, metrics) are expected to reach a pre-alpha state by the end of 2025 - see [`core/Readme.md`](https://github.com/CortexFlow/CortexBrain/blob/main/core/Readme.md) for the core project map.

## Roadmap

The broader roadmap is described in the [`README.md`](https://github.com/CortexFlow/CortexBrain/blob/main/README.md#current-development-focus) "Current Development Focus" section. The active and experimental tracks are:

| Track | Status |
|-------|--------|
| OpenTelemetry integration | Active - see the [Integrated Metrics](metrics.md#opentelemetry-metrics-incoming-metrics-patch) page for the incoming 16-instrument suite |
| Metrics enhancement | Active - expanding the current metrics landscape for deeper system understanding |
| Dashboard integration | Active - delivering user-centered data visualization from the collected metrics |
| Cloud-Edge multi-cluster | Experimental |
| GPU observability | Experimental - see the [GPU Metrics](gpu-metrics.md) page for the upcoming CUDA + NVML metric suite |
| MCP integration | Experimental - see the [MCP Server](mcp-server.md) page for the Prometheus-backed MCP server (on `origin/0.1.5` / `origin/metrics-patch`) |

## Proposing new features

New features and significant changes **must** be discussed before a pull request is opened. The flow is:

1. Open a [GitHub Discussion](https://github.com/CortexFlow/CortexBrain/discussions) describing the feature and how it aligns with the roadmap above.
2. Once there is rough agreement, open a [GitHub Issue](https://github.com/CortexFlow/CortexBrain/issues/new/choose) to track the work.
3. Reference the discussion/issue in your commits and PR (see the [Development Workflow](dev-workflow.md) page for the conventions).

Pull requests without a related issue or discussion are not accepted.

## GitHub references

### Discussions & issues

- [GitHub Discussions](https://github.com/CortexFlow/CortexBrain/discussions) - for feature proposals and design questions.
- [GitHub Issues](https://github.com/CortexFlow/CortexBrain/issues) - for bug reports and tracked work.
- [Core milestone (#1)](https://github.com/CortexFlow/CortexBrain/milestone/1) - the "Core" milestone.
- [Conntracker issue (#78)](https://github.com/CortexFlow/CortexBrain/issues/78) and [Identity issue (#92)](https://github.com/CortexFlow/CortexBrain/issues/92) - the two main core-component tracking issues.

### Labels

Issues and PRs are labeled to help routing and discovery. The canonical set (referenced from `README.md` and `contacts/contact.md`):

| Label | Use |
|-------|-----|
| [`rust`](https://github.com/CortexFlow/CortexBrain/labels/rust) | Rust / core development |
| [`ebpf`](https://github.com/CortexFlow/CortexBrain/labels/ebpf) | eBPF-specific work |
| [`javascript`](https://github.com/CortexFlow/CortexBrain/labels/javascript) | Dashboard / frontend |
| [`documentation`](https://github.com/CortexFlow/CortexBrain/labels/documentation) | Docs work |
| [`question`](https://github.com/CortexFlow/CortexBrain/labels/question) | Questions and clarifications |
| [`code refactoring`](https://github.com/CortexFlow/CortexBrain/labels/code%20refactoring) | Refactoring tasks |
| `cortexflow cli` | CLI-specific issues (also routes auto-assignment in CI) |

!!! note
    There is no checked-in `labels.yml` in the repository - the label set is configured directly on GitHub. If a label is missing, ask a maintainer to create it.

### Roles we are looking for

We are actively looking for contributors and collaborators. The open roles and their related GitHub references:

| Role | Skills | Related labels / milestones |
|------|--------|------------------------------|
| CortexBrain Core Developer | Kubernetes, Networks, Rust | [`rust`](https://github.com/CortexFlow/CortexBrain/labels/rust), [`ebpf`](https://github.com/CortexFlow/CortexBrain/labels/ebpf), [Core milestone](https://github.com/CortexFlow/CortexBrain/milestone/1) |
| CortexBrain Dashboard Developer | React, Frontend Development, JavaScript/TypeScript | [`javascript`](https://github.com/CortexFlow/CortexBrain/labels/javascript) |
| General Maintainers | GitHub, practical organization, documentation | [`documentation`](https://github.com/CortexFlow/CortexBrain/labels/documentation), [`question`](https://github.com/CortexFlow/CortexBrain/labels/question) |
| Code Reviewers / Testers | Rust, JavaScript/TypeScript, Kubernetes, Docker | [`code refactoring`](https://github.com/CortexFlow/CortexBrain/labels/code%20refactoring) |

If you have knowledge in DevOps, Kubernetes, or networking, email `lorenzotettamanti5@gmail.com` - see the [Contacts](../contacts/contact.md) page for details.

## External resources

Because the project sits at the intersection of several deep topics, the [Developer Guide index](index.md#external-resources) lists the best starting points for eBPF, the Aya Rust framework, gRPC/tonic, and Kubernetes networking.