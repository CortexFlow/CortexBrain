# Phase 1: Build image
FROM rust:1.85 AS builder

# Set working directory
WORKDIR /usr/src/app/metrics

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
WORKDIR /usr/src/cortexbrain-metrics

# Copy the compiled binary
COPY --from=builder /usr/src/app/metrics/target/release/metrics /usr/local/bin/cortexflow-metrics

# Copy configuration files
COPY metrics_tracer /usr/src/cortexbrain-metrics/metrics_tracer

# Set environment variable
ENV BPF_PATH="/usr/src/cortexbrain-metrics/metrics_tracer"

# Default command
CMD ["cortexflow-metrics"]
