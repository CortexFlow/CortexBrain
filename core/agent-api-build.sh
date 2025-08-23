#!/bin/bash

echo "Building the conntracker files"
pushd src/components/conntracker
./build-conntracker.sh
popd

echo "Copying connection tracker binaries"
cp -r target/bpfel-unknown-none/release/conntracker conntracker

# Run docker build
docker build -f api/Dockerfile -t cortexflow-agent:0.0.1 .

# Cleanup
echo "Cleaning building files"
rm -rf conntracker
