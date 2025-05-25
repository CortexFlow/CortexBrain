#!/bin/bash

# Building identity files
echo "Building the conntracker files"
pushd ../conntracker
./build-conntracker.sh
popd

echo "Copying connection tracker binaries"
cp -r ../../../target/bpfel-unknown-none/release/conntracker conntracker

# Run docker build
docker build -t identity:0.0.1 .

# Cleanup
echo "Cleaning building files"
rm -rf conntracker
