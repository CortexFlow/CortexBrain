# Phase 1: Build image
FROM rust:1.85 AS builder

# Set working directory
WORKDIR /usr/src/app

# Copy the shared library in the correct location
WORKDIR /usr/src/shared
COPY .shared/Cargo.toml .
COPY .shared/src ./src

# Then create the kernel project structure
WORKDIR /usr/src/app/kernel
COPY Cargo.toml .
COPY src ./src

# Ensure Cargo recognizes the shared dependency
RUN cargo fetch

# Build the project
RUN cargo build --release

# Phase 2: Create final image
FROM ubuntu:22.04

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create directory for the kernel
WORKDIR /usr/src/cortexbrain-kernel

# Create the /etc/dns directory
RUN mkdir -p /etc/dns

# Copy the binary from builder
COPY --from=builder /usr/src/app/kernel/target/release/kernel /usr/local/bin/cortexflow-kernel

# Copy the config.yaml file into /etc/dns
COPY config.yaml /etc/dns/config.yaml

# Set the kernel execution command
CMD ["cortexflow-kernel"]
