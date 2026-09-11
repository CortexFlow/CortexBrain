# CortexBrain

![Version: 0.1.0](https://img.shields.io/badge/Version-0.1.0-informational?style=flat-square)

This chart installs CortexFlow to a kubernetes cluster, instead of using the cli installation method.

## Values

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| agent.image.repository | string | `"ghcr.io/cortexflow/agent"` |  |
| agent.image.version | string | `"latest"` |  |
| agent.priorityClassName | string | `""` |  |
| agent.resources.limits.memory | string | `"200Mi"` |  |
| agent.resources.requests.cpu | string | `"100m"` |  |
| agent.resources.requests.memory | string | `"100Mi"` |  |
| agent.securityContext.allowPrivilegeEscalation | bool | `true` |  |
| agent.securityContext.capabilities.add[0] | string | `"SYS_ADMIN"` |  |
| agent.securityContext.capabilities.add[1] | string | `"NET_ADMIN"` |  |
| agent.securityContext.capabilities.add[2] | string | `"SYS_RESOURCE"` |  |
| agent.securityContext.capabilities.add[3] | string | `"BPF"` |  |
| agent.securityContext.capabilities.add[4] | string | `"SYS_PTRACE"` |  |
| agent.securityContext.privileged | bool | `true` |  |
| agent.tolerations | list | `[]` |  |
| blocklist | string | `""` |  |
| bpfMapPermissions.image.repository | string | `"ubuntu"` |  |
| bpfMapPermissions.image.version | string | `"24.04"` |  |
| bpfMapPermissions.securityContext.allowPrivilegeEscalation | bool | `true` |  |
| bpfMapPermissions.securityContext.capabilities.add[0] | string | `"SYS_ADMIN"` |  |
| bpfMapPermissions.securityContext.capabilities.add[1] | string | `"NET_ADMIN"` |  |
| bpfMapPermissions.securityContext.capabilities.add[2] | string | `"SYS_RESOURCE"` |  |
| bpfMapPermissions.securityContext.capabilities.add[3] | string | `"BPF"` |  |
| bpfMapPermissions.securityContext.capabilities.add[4] | string | `"SYS_PTRACE"` |  |
| bpfMapPermissions.securityContext.privileged | bool | `true` |  |
| bpfMapPermissions.securityContext.runAsUser | int | `0` |  |
| bpfTool.image.repository | string | `"danielpacak/bpftool-runner"` |  |
| bpfTool.image.version | string | `"latest"` |  |
| bpfTool.resources.limits.cpu | string | `"1"` |  |
| bpfTool.resources.limits.memory | string | `"200Mi"` |  |
| bpfTool.resources.requests.cpu | string | `"1"` |  |
| bpfTool.resources.requests.memory | string | `"100Mi"` |  |
| bpfTool.securityContext.allowPrivilegeEscalation | bool | `true` |  |
| bpfTool.securityContext.capabilities.add[0] | string | `"SYS_ADMIN"` |  |
| bpfTool.securityContext.capabilities.add[1] | string | `"NET_ADMIN"` |  |
| bpfTool.securityContext.capabilities.add[2] | string | `"SYS_RESOURCE"` |  |
| bpfTool.securityContext.capabilities.add[3] | string | `"BPF"` |  |
| bpfTool.securityContext.capabilities.add[4] | string | `"SYS_PTRACE"` |  |
| bpfTool.securityContext.privileged | bool | `true` |  |
| global.otel.endpoint | string | `"http://localhost:4317"` |  |
| global.otel.protocol | string | `"grpc"` |  |
| global.priorityClassName | string | `""` |  |
| global.tolerations | list | `[]` |  |
| identity.image.repository | string | `"ghcr.io/cortexflow/identity"` |  |
| identity.image.version | string | `"latest"` |  |
| identity.priorityClassName | string | `""` |  |
| identity.resources.limits.memory | string | `"200Mi"` |  |
| identity.resources.requests.cpu | string | `"100m"` |  |
| identity.resources.requests.memory | string | `"100Mi"` |  |
| identity.securityContext.allowPrivilegeEscalation | bool | `true` |  |
| identity.securityContext.capabilities.add[0] | string | `"SYS_ADMIN"` |  |
| identity.securityContext.capabilities.add[1] | string | `"NET_ADMIN"` |  |
| identity.securityContext.capabilities.add[2] | string | `"SYS_RESOURCE"` |  |
| identity.securityContext.capabilities.add[3] | string | `"BPF"` |  |
| identity.securityContext.capabilities.add[4] | string | `"SYS_PTRACE"` |  |
| identity.securityContext.privileged | bool | `true` |  |
| identity.tolerations | list | `[]` |  |
| metrics.image.repository | string | `"ghcr.io/cortexflow/metrics"` |  |
| metrics.image.version | string | `"latest"` |  |
| metrics.priorityClassName | string | `""` |  |
| metrics.resources.limits.cpu | string | `"1"` |  |
| metrics.resources.limits.memory | string | `"200Mi"` |  |
| metrics.resources.requests.cpu | string | `"1"` |  |
| metrics.resources.requests.memory | string | `"100Mi"` |  |
| metrics.securityContext.allowPrivilegeEscalation | bool | `true` |  |
| metrics.securityContext.capabilities.add[0] | string | `"SYS_ADMIN"` |  |
| metrics.securityContext.capabilities.add[1] | string | `"NET_ADMIN"` |  |
| metrics.securityContext.capabilities.add[2] | string | `"SYS_RESOURCE"` |  |
| metrics.securityContext.capabilities.add[3] | string | `"BPF"` |  |
| metrics.securityContext.capabilities.add[4] | string | `"SYS_PTRACE"` |  |
| metrics.securityContext.privileged | bool | `true` |  |
| metrics.tolerations | list | `[]` |  |
| serviceAccountName | string | `"cortexflow-sa"` |  |

----------------------------------------------
Autogenerated from chart metadata using [helm-docs v1.14.2](https://github.com/norwoodj/helm-docs/releases/v1.14.2)
