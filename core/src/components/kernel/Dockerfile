# Phase 1: Build image
FROM rust:1.83 AS builder

# Set working directory
WORKDIR /usr/src/app

# First, create the shared project structure
WORKDIR /usr/src/app/shared
COPY .shared/Cargo.toml .
COPY .shared/src ./src

# Then create the kernel project structure
WORKDIR /usr/src/app/kernel
COPY Cargo.toml .
COPY src ./src
COPY config.yaml .


# Build the project
RUN cargo build --release

# Phase 2: Create final image
FROM ubuntu:22.04

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create directory for the 
WORKDIR /usr/src/cortexbrain-kernel

# Copy the binary from builder
COPY --from=builder /usr/src/app/kernel/target/release/kernel /usr/local/bin/cortexflow-kernel

# Copy config file
COPY config.yaml /usr/src/cortexbrain-kernel/config.yaml

# Set config path environment variable
ENV CONFIG_PATH="/usr/src/cortexbrain-kernel/config.yaml"

# Set the kernel execution command
CMD ["cortexflow-kernel"]
