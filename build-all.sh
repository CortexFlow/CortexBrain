#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CORE_DIR="$ROOT_DIR/core"

echo "Building CortexFlow Agent"
"$CORE_DIR/agent-api-build.sh"

echo ""
echo "Building CortexFlow Identity"
"$CORE_DIR/src/components/identity/build-identity.sh"

echo ""
echo "Building CortexFlow Metrics"
"$CORE_DIR/src/components/metrics/build-metrics.sh"

sleep 1

echo "Insert image version. e.g 0.1.2/latest or type skip to skip the uploading processing"
echo
read -p "Insert cortexflow-agent version: " agent_version
read -p "Insert cortexflow-identity version: " identity_version
read -p "Insert cortexflow-metrics version: " metrics_version

echo
echo "Tagging & pushing docker images..."
echo

if [ "$metrics_version" != "skip" ]; then
    docker tag metrics:0.0.1 lorenzotettamanti/cortexflow-metrics:$metrics_version
    docker push lorenzotettamanti/cortexflow-metrics:$metrics_version
else
    echo "Skipping cortexflow-metrics image upload"
fi

if [ "$agent_version" != "skip" ]; then
    docker tag cortexflow-agent:0.0.1 lorenzotettamanti/cortexflow-agent:$agent_version
    docker push lorenzotettamanti/cortexflow-agent:$agent_version
else
    echo "Skipping cortexflow-agent image upload"
fi

if [ "$identity_version" != "skip" ]; then
    docker tag identity:0.0.1 lorenzotettamanti/cortexflow-identity:$identity_version
    docker push lorenzotettamanti/cortexflow-identity:$identity_version
else
    echo "Skipping cortexflow-identity image upload"
fi
