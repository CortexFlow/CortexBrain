#!/bin/bash

# Create temporary shared directory
mkdir -p .shared

# Copy shared files
cp -r ../shared/src .shared/
cp -r ../shared/Cargo.toml .shared/


# Run docker build
docker build -t cortexflow-client:0.0.1 .

# Cleanup
rm -rf .shared