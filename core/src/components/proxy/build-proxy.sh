#!/bin/bash

# Create temporary shared directory
mkdir -p .shared

# Copy shared files
cp -r ../../shared/src .shared/
cp -r ../../shared/Cargo.toml .shared/
cp -r ../../client/config.yaml config.yaml
 
# Run docker build
docker build -t proxy:0.0.1 .

# Cleanup
rm -rf .shared
rm -rf config.yaml