# Quickstart

This page walks you through the fastest path from zero to your first CortexBrain metric. It assumes you have Rust and a local Kubernetes tool installed. For the full setup (including dashboard development and manual builds) see the [Installation guide](installation.md).

## Prerequisites

- A Linux system with kernel `>= 5.15`
- Rust (nightly) - `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- One of the local Kubernetes tools below

## Step 1 - Install `cfcli`

```bash
cargo install cortexflow-cli
cfcli --version
```

## Step 2 - Start a local cluster

=== "Minikube"

    ```bash
    minikube start --cni=calico
    ```

    Minikube does not mount the BPF filesystem automatically. If you hit permission issues later, run:
    ```bash
    minikube ssh
    mount -t bpf bpffs /sys/fs/bpf
    chmod 700 /sys/fs/bpf
    ```

=== "Kind"

    Create a `values.yaml`:
    ```yaml
    kind: Cluster
    apiVersion: kind.x-k8s.io/v1alpha4
    nodes:
    - role: control-plane
    - role: worker
    - role: worker
    networking:
      disableDefaultCNI: true
      podSubnet: 192.168.0.0/16
    ```

    ```bash
    kind create cluster --config values.yaml --name dev
    kubectl create -f https://raw.githubusercontent.com/projectcalico/calico/v3.30.3/manifests/operator-crds.yaml
    kubectl create -f https://raw.githubusercontent.com/projectcalico/calico/v3.30.3/manifests/tigera-operator.yaml
    kubectl create -f https://raw.githubusercontent.com/projectcalico/calico/v3.30.3/manifests/custom-resources.yaml
    kubectl apply -f https://raw.githubusercontent.com/projectcalico/calico/v3.30.3/manifests/calico.yaml
    ```

## Step 3 - Install CortexBrain

```bash
cfcli install cortexflow
```

You will be prompted for your cluster environment - type `Kubernetes`. The install takes under a minute and applies the `configmap`, `configmap-role`, `rolebinding`, `cortexflow-rolebinding`, `identity`, and `agent` manifests in the `cortexflow` namespace.

## Step 4 - Verify the installation

```bash
cfcli status
```

Expected output:

```
🔍 CortexFlow Status Report
==================================================

📦 Namespace Status:
  ✅ cortexflow namespace: EXISTS

🚀 Pods Status:
  ✅ cortexflow-agent-ffbb95665-l47dw: Running (1/1)
  ✅ cortexflow-identity-7579cd5974-4c9hv: Running (2/2)

🌐 Services Status:
  🔗 cortexflow-agent: ClusterIP (10.96.88.219)

==================================================
```

If a pod is not `Running`, check the [Troubleshooting](../developer-guide/troubleshooting.md) page.

## Step 5 - Open a port-forward to the agent

The `cortexflow-agent` Service is `ClusterIP` only. Open a tunnel so `cfcli` can reach it on `127.0.0.1:9090`:

```bash
kubectl port-forward svc/cortexflow-agent 9090:9090 -n cortexflow
```

Keep this running in a separate terminal.

## Step 6 - Your first metric

List the available agent endpoints:

```bash
cfcli monitoring list
```

Expected output (server reflection lists all `agent.Agent` methods):

```
====> Connected to CortexFlow Server Reflection
Available services:
ActiveConnections
AddIpToBlocklist
CheckBlocklist
RmIpFromBlocklist
GetLatencyMetrics
GetDroppedPacketsMetrics
```

Now pull the active connections detected by the Identity service:

```bash
cfcli monitoring connections
```

You will get a JSON-style map of `event_id -> ip` for the most recent detected TCP/UDP packets.

## Step 7 - Explore the other commands

| Command | What it returns |
|---------|-----------------|
| `cfcli monitoring latencymetrics` | TCP connection latency (`delta_us`) plus average/min/max |
| `cfcli monitoring droppedpackets` | Socket-level drops with `sk_drops`, `sk_err`, `sk_backlog_len` |
| `cfcli policy check-blocklist` | The current blocklist IPs |
| `cfcli policy create-blocklist --flags <IP>` | Add an IP to the blocklist |
| `cfcli policy remove-ip --flags <IP>` | Remove an IP from the blocklist |

See the [CLI overview](../cfcli/overview.md) for the full command reference.

## Next steps

- [CLI overview](../cfcli/overview.md) - every `cfcli` command documented.
- [Developer Guide](../developer-guide/index.md) - architecture, agent API, metrics, and contribution workflow.
- [Troubleshooting](../developer-guide/troubleshooting.md) - common issues and fixes.