# Development Workflow

This page describes how to contribute to CortexBrain: the fork-to-PR flow, branch and commit conventions, and the CI pipeline that runs on every pull request.

## Contribution flow

CortexBrain follows a standard fork-and-PR model. Before opening a pull request, you **must** open an issue or a GitHub Discussion for any non-trivial change - pull requests without a related issue or discussion are not accepted (see `CONTRIBUTING.md`).

=== "Step 1 - Fork & clone"

    ```bash
    git clone https://github.com/<your-username>/CortexBrain.git
    cd CortexBrain
    git remote add upstream https://github.com/CortexFlow/CortexBrain.git
    ```

=== "Step 2 - Open an issue or discussion"

    New features and significant changes require a prior [GitHub Discussion](https://github.com/CortexFlow/CortexBrain/discussions). Bug fixes and small improvements can start directly from a [GitHub Issue](https://github.com/CortexFlow/CortexBrain/issues/new/choose). Reference the issue/discussion number in every commit and in the PR description.

=== "Step 3 - Create a dedicated branch"

    ```bash
    git checkout -b feature/<short-description>
    ```

    Pull requests without a dedicated branch are not accepted. Use a descriptive branch name prefixed by the type of change (`feature/`, `fix/`, `docs/`).

=== "Step 4 - Commit with a clear message"

    Tag the related issue in every commit using the `#issue: message` convention:

    ```bash
    git commit -m "#78: add TCP connection tracing in conntracker"
    ```

    Keep commits focused. Avoid pull requests larger than ~5000 lines of code unless explicitly justified.

=== "Step 5 - Open a Pull Request"

    Open the PR against `CortexFlow/CortexBrain:main`. The PR template asks for:

    - A description of the change
    - The type of change (bug fix, new feature, documentation, refactoring, other)
    - A checklist (tested locally, docs updated, new tests added, builds successfully)
    - Related issues using `Closes #<n>` or `Fixes #<n>`

    The PR is auto-assigned to a maintainer (`@LorenzoTettamanti`) and an auto-reviewer assignment workflow runs.

## Branch conventions

| Prefix | Use |
|--------|-----|
| `feature/` | New functionality or components |
| `fix/` | Bug fixes |
| `docs/` | Documentation-only changes |
| `core/` | Core eBPF / agent work (matches the existing `feature/core`, `feature/ebpf-core` branches) |
| `frontend/` | Dashboard work (matches `feature/frontend`) |
| `cli/` | `cfcli` work (matches `feature/cli`) |

## Commit message conventions

- Reference the issue number: `#<n>: <imperative description>`
- Keep the subject line under 72 characters
- Use the imperative mood (`add`, `fix`, `remove`, not `added` or `fixes`)

Examples:

```
#78: add IPv6 support to the conntracker TC classifier
#92: fix pinned map path mismatch in identity service
docs: update Agent API overview with the 6 RPC methods
```

## CI pipeline

Every pull request and every push to `main` runs the **core-build-checks** workflow (`.github/workflows/core-build-checks.yml`).

!!! note
    The CI pipeline focuses on **build verification, not tests**. Automated test execution is still a work in progress.

The workflow:

1. Installs Rust **nightly** and the eBPF toolchain (`clang`, `llvm`, `libelf`, `libpcap`, `libbpf`, `bpftool`, `protobuf`).
2. Builds the three core components via their build scripts:
    - `core/agent-api-build.sh` - the `agent-api` gRPC server
    - `core/src/components/identity/build-identity.sh` - the identity service
    - `core/src/components/metrics/build-metrics.sh` - the metrics service
3. Verifies that the resulting Docker images exist.

A top-level orchestrator, `build-all.sh`, builds all three components and tags/pushes Docker images to the `lorenzotettamanti/cortexflow-*` registry.

## Local build

To build the core components locally (outside CI), see the [Getting Started for developers](../getting-started/installation.md#getting-started-for-developers) section. The short version:

```bash
# From the repo root
cargo +nightly build --release -p cortexflow_agent_api
```

Each component also has its own build script under `core/` and `core/src/components/<name>/`.

## Code of conduct & security

- All contributors are expected to follow the [Contributor Covenant 2.0](https://www.contributor-covenant.org/) code of conduct (see `CODE_OF_CONDUCT.md` in the repo root). Reports go to `lorenzo.tettamanti5@gmail.com`.
- **Security vulnerabilities must not be reported via public GitHub issues.** Email `lorenzotettamanti5@gmail.com` or `lorenzolollobrada@gmail.com` directly (see `SECURITY.md`).

## Roles we are looking for

| Role | Skills |
|------|--------|
| Core Developer | Kubernetes, Networks, Rust |
| Dashboard Developer | React, Frontend Development, JavaScript/TypeScript |
| General Maintainers | GitHub, practical organization, documentation |
| Code Reviewers / Testers | Rust, JavaScript/TypeScript, Kubernetes, Docker |

See the [Development Goals & Discussions](discussions.md) page for the related GitHub labels and milestones.