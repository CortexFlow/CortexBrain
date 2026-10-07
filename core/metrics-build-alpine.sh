#! /bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "Building cortexflow-metrics image from core workspace context"
cd "$SCRIPT_DIR"

docker build -f src/components/metrics/Dockerfile.alpine -t alpine-cf-metrics:latest --provenance=false --sbom=false .
