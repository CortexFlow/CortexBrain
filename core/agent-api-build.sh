#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "Building cortexflow-agent image from core workspace context"
cd "$SCRIPT_DIR"
docker build -f api/Dockerfile -t cortexflow-agent:0.0.1 --provenance=false --sbom=false .
