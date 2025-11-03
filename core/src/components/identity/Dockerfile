# Phase 1: Build image
FROM rust:1.90 AS builder

# Set working directory
WORKDIR /usr/src/app/identity-service

# Copy Cargo manifest and sources
COPY Cargo.toml .
COPY src ./src

# Fetch dependencies and build release
RUN cargo fetch && cargo build --release

# Phase 2: Final minimal image
FROM ubuntu:24.04

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create working directory
WORKDIR /usr/src/cortexbrain-identity-service

# Copy the compiled binary
COPY --from=builder /usr/src/app/identity-service/target/release/cortexflow_identity /usr/local/bin/cortexflow-identity-service

# Copy configuration files
COPY conntracker /usr/src/cortexbrain-identity-service/conntracker

# Set environment variable
ENV BPF_PATH="/usr/src/cortexbrain-identity-service/conntracker"
ENV PIN_MAP_PATH="/sys/fs/bpf/maps"

# Default command
CMD ["cortexflow-identity-service"]
