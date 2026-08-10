#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Building identity files
echo "Building the conntracker files"
pushd ../conntracker
./build-conntracker.sh
popd

echo "Copying connection tracker binaries"
cp -r ../../../target/bpfel-unknown-none/release/conntracker conntracker
cp -r ../../../common common 

# Run docker build
docker build -t identity:0.0.1 --provenance=false --sbom=false .

# Cleanup
echo "Cleaning building files"
rm -rf conntracker
rm -rf common
