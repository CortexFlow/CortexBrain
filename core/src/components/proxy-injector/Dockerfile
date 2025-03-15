# Phase 1: Build image
FROM rust:1.85 AS builder

# Set working directory
WORKDIR /usr/src/app

# Then create the proxy-injector project structure
WORKDIR /usr/src/app/proxy-injector
COPY Cargo.toml .
COPY src ./src

# Build the project
RUN cargo build --release

# Phase 2: Create final image
FROM ubuntu:22.04

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create directory for the proxy-injector
WORKDIR /usr/src/cortexbrain-proxy-injector

# Copy the binary from builder
COPY --from=builder /usr/src/app/proxy-injector/target/release/proxy-injector /usr/local/bin/cortexbrain-proxy-injector

# Set the proxy execution command
CMD ["cortexbrain-proxy-injector"]
