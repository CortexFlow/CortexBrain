#!/bin/bash

# Building identity files
echo "Building the metrics-tracer files"
pushd ../metrics_tracer
./build-metrics-tracer.sh
popd

echo "Copying metrics_tracer binaries"
cp -r ../../../target/bpfel-unknown-none/release/metrics_tracer metrics_tracer
cp -r ../../../common common 

# Run docker build
docker build -t metrics:0.0.1 .

# Cleanup
echo "Cleaning building files"
rm -rf metrics_tracer
rm -rf common
