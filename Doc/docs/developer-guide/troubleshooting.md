# Troubleshooting

This page consolidates the most common issues you may hit while running CortexBrain, with their symptoms, root causes, and fixes. For the initial setup, see the [Quickstart](../getting-started/quickstart.md) and the [Installation guide](../getting-started/installation.md).

## Quick diagnostic table

| Symptom | Likely cause | Fix |
|---------|--------------|-----|
| `cfcli status` reports the `cortexflow` namespace missing | Components not installed | `cfcli install cortexflow` |
| `cfcli monitoring *` returns `connection refused` | No port-forward to the agent | `kubectl port-forward svc/cortexflow-agent 9090:9090 -n cortexflow` |
| Agent pod `CrashLoopBackOff`, log: `No such file or directory` | Pinned BPF map path mismatch | See [Pinned map path mismatch](#pinned-map-path-mismatch) |
| Agent log: `permission denied` opening `/sys/fs/bpf/...` | bpffs not mounted or wrong permissions | See [BPF filesystem not mounted](#bpf-filesystem-not-mounted-minikube) |
| `cfcli monitoring list` returns an error | Agent not running or reflection not registered | Check `cfcli status` and agent logs |
| MCP server tools return empty / error | Prometheus not reachable at `localhost:9090` | Port-forward Prometheus or run the docker-compose stack |

## BPF filesystem not mounted (Minikube)

Minikube does not mount the BPF filesystem automatically. CortexBrain uses BPF maps with the pinning operation ([What's Pinning?](https://docs.ebpf.io/linux/concepts/pinning/)), so `/sys/fs/bpf` must be mounted and writable.

```bash
minikube ssh
mount -t bpf bpffs /sys/fs/bpf
ls -ld /sys/fs/bpf
```

Expected output:

```
drwx-----T 3 root root 0 Sep  4 16:34 /sys/fs/bpf
```

If the permissions are wrong, fix them:

```bash
chmod 700 /sys/fs/bpf
```

The `identity` Deployment includes an initContainer `bpf-map-permissions` that runs `mount -t bpf bpf /sys/fs/bpf` on startup, so in a Kubernetes deployment this is handled automatically as long as the pod is privileged and mounts the host `/sys/fs/bpf` as a bidirectional volume.

## Pinned map path mismatch

!!! warning "Known caveat"
    The Agent hardcodes the pinned map paths `/sys/fs/bpf/maps/events_map`, `/sys/fs/bpf/maps/blocklist_map`, `/sys/fs/bpf/trace_maps/net_metrics`, and `/sys/fs/bpf/trace_maps/time_stamp_events` (see `core/api/src/api.rs`). The Identity and Metrics services pin under `${PIN_MAP_PATH}`, which the Dockerfiles set to `/sys/fs/bpf/maps` and `/sys/fs/bpf/trace_maps` respectively. The deployment must reconcile these paths.

**Symptoms**: the `cortexflow-agent` pod crashes on startup with `No such file or directory` when calling `MapData::from_pin(...)`.

**Diagnosis**:

1. Check which maps are actually pinned on the node:
   ```bash
   kubectl exec -it cortexflow-identity-<pod> -n cortexflow -- ls -R /sys/fs/bpf/
   ```
2. Confirm the Identity and Metrics env vars:
   ```bash
   kubectl exec -it cortexflow-identity-<pod> -n cortexflow -- env | grep PIN_MAP_PATH
   kubectl exec -it cortexflow-metrics-<pod> -n cortexflow -- env | grep PIN_MAP_PATH
   ```
3. Confirm the agent expects maps at `/sys/fs/bpf/maps/...` and `/sys/fs/bpf/trace_maps/...`.

**Fix**: ensure the `PIN_MAP_PATH` of Identity is `/sys/fs/bpf/maps` and of Metrics is `/sys/fs/bpf/trace_maps` (these are the defaults in their Dockerfiles). If you changed them, update the agent's hardcoded paths in `core/api/src/api.rs` accordingly, or create symlinks under `/sys/fs/bpf/`.

## Port-forward to the agent

The `cortexflow-agent` Service is `ClusterIP` only (port 9090, gRPC). The `cfcli` client connects to `127.0.0.1:9090`, so you must open a port-forward from your host:

```bash
kubectl port-forward svc/cortexflow-agent 9090:9090 -n cortexflow
```

Keep this running in a separate terminal while you use `cfcli monitoring *` commands. If the forward is missing, every `cfcli monitoring` command returns `connection refused`.

## Permissions on `/sys/fs/bpf`

The agent needs read-write access to `/sys/fs/bpf`. The deployment manifests set `privileged: true` with the `BPF`, `SYS_ADMIN`, `NET_ADMIN`, `SYS_PTRACE`, and `SYS_RESOURCE` capabilities and mount `/sys/fs/bpf` as a bidirectional hostPath volume. If you run outside Kubernetes (or with a reduced security context), verify:

```bash
ls -ld /sys/fs/bpf
# must be drwx for root
```

## Debugging with bpftool

The `cortexflow-identity` and `cortexflow-metrics` Deployments ship a `bpftool-control-manager` sidecar (`danielpacak/bpftool-runner:latest`) so you can inspect the BPF state from inside the cluster.

List all pinned maps:

```bash
kubectl exec -it cortexflow-identity-<pod> -c bpftool-control-manager -n cortexflow -- bpftool map show
```

Inspect a specific pinned map:

```bash
kubectl exec -it cortexflow-identity-<pod> -c bpftool-control-manager -n cortexflow -- bpftool map show pinned /sys/fs/bpf/maps/blocklist_map
```

List loaded BPF programs:

```bash
kubectl exec -it cortexflow-identity-<pod> -c bpftool-control-manager -n cortexflow -- bpftool prog show
```

## Reading pod logs

CortexBrain pods run in the `cortexflow` namespace. Use `cfcli logs` or `kubectl logs` directly:

```bash
cfcli logs --component agent --namespace cortexflow
kubectl logs -n cortexflow -l app=cortexflow-agent
kubectl logs -n cortexflow -l app=cortexflow-identity
kubectl logs -n cortexflow -l app=cortexflow-metrics
```

The agent logs startup messages on stderr (`Starting cortexflow-mcp`, `cortexflow-mcp running`) and gRPC traffic on the configured logger.

## Namespace not found

If `cfcli status` reports that the `cortexflow` namespace does not exist, the components have not been installed:

```bash
cfcli install cortexflow
```

This creates the namespace, applies the RBAC, deploys the agent, identity, and metrics components, and seeds the `cortexbrain-client-config` ConfigMap. See the [CLI overview](../cfcli/overview.md) for the full command list.

## MCP server cannot reach Prometheus

The MCP server (see [MCP Server](mcp-server.md)) queries Prometheus at the hardcoded URL `http://localhost:9090/api/v1/query`. If the MCP tools return empty results or connection errors:

1. Ensure Prometheus is running and reachable on `localhost:9090` (e.g. via the `Examples/run-with-docker/docker-compose.yaml` stack, or `kubectl port-forward svc/prometheus 9090:9090`).
2. Verify the OTLP collector is exporting metrics into Prometheus with the `cortexbrain_` namespace prefix (configured in `otel-collector-config.yaml`).
3. Test the Prometheus query directly:
   ```bash
   curl 'http://localhost:9090/api/v1/query?query=up'
   ```

## See also

- [Architecture Overview](architecture.md) - the four-stage pipeline and the BPF map paths.
- [Agent API Overview](agent-api.md) - the gRPC service and the BPF maps behind each RPC.
- [MCP Server](mcp-server.md) - the Prometheus-backed MCP server.
- [Quickstart](../getting-started/quickstart.md) - the 5-minute end-to-end flow.