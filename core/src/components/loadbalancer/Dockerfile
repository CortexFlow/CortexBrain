# Phase 1: Build image
FROM rust:1.85 AS builder

# Set working directory
WORKDIR /usr/src/app

# Copy the shared library in the correct location
WORKDIR /usr/src/shared
COPY .shared/Cargo.toml .
COPY .shared/src ./src

# Copying the XDP filter binaries
WORKDIR /usr/src/app/loadbalancer

# Then create the loadbalancer project structure
WORKDIR /usr/src/app/loadbalancer
COPY Cargo.toml .
COPY src ./src

# Ensure Cargo recognizes the shared dependency
RUN cargo fetch

# Build the project
RUN cargo build --release

# Phase 2: Create final image
FROM ubuntu:24.04

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create directory for the loadbalancer
WORKDIR /usr/src/cortexbrain-loadbalancer

# Copy the binary from builder
COPY --from=builder /usr/src/app/loadbalancer/target/release/loadbalancer /usr/local/bin/cortexflow-loadbalancer

# Copy config file
COPY xdp-filter /usr/src/cortexbrain-loadbalancer/xdp-filter

# Set config path environment variable
ENV BPF_PATH="/usr/src/cortexbrain-loadbalancer/xdp-filter"

# Set the loadbalancer execution command
CMD ["cortexflow-loadbalancer"]
