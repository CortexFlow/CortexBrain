#!/bin/bash

# Create temporary shared directory
mkdir -p .shared

# Copy shared files
echo "Copying shared files"
cp -r ../../shared/src .shared/
cp -r ../../shared/Cargo.toml .shared/
cp -r ../../client/config.yaml config.yaml

# Building XDP filter files
echo "Building the xdp filter files"
pushd ../xdp
./build.sh
popd

echo "Copying xdp-filter binaries"
cp -r ../../../target/bpfel-unknown-none/release/xdp-filter xdp-filter

# Run docker build
docker build -t loadbalancer:0.0.1 .

# Cleanup
echo "Cleaning building files"
rm -rf .shared
rm -rf config.yaml
rm -rf xdp-filter
