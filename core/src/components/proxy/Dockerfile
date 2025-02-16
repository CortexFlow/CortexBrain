# Phase 1: Build image
FROM rust:1.83 AS builder

# Set working directory
WORKDIR /usr/src/app

# Copy the shared library in the correct location
WORKDIR /usr/src/shared
COPY .shared/Cargo.toml .
COPY .shared/src ./src

# Then create the proxy project structure
WORKDIR /usr/src/app/proxy
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

# Create directory for the proxy
WORKDIR /usr/src/cortexbrain-proxy

# Copy the binary from builder
COPY --from=builder /usr/src/app/proxy/target/release/proxy /usr/local/bin/cortexflow-proxy

# Set the proxy execution command
CMD ["cortexflow-proxy"]
