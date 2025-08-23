#!/bin/bash
set -e

echo "Building CortexFlow Agent"
pushd ./core
./agent-api-build.sh
popd

sleep 1

echo "Building CortexFlow Identity"
pushd ./core/src/components/identity
./build-identity.sh
popd

sleep 1

echo "Building CortexFlow Metrics"
pushd ./core/src/components/metrics
./build-metrics.sh
popd

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
