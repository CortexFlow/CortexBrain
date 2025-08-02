# Phase 1: Build image
FROM rust:1.86 AS builder

# Install system dependencies including protoc
RUN apt-get update && apt-get install -y \
    build-essential \
    libprotobuf-dev \
    protobuf-compiler \
    pkg-config \
    libssl-dev \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Set environment variables for protoc
ENV PROTOC=/usr/bin/protoc
ENV PROTOC_INCLUDE=/usr/include

# Set working directory
WORKDIR /usr/src/app/agent

# Copy Cargo manifest and sources
COPY . .

# Fetch dependencies and build release
RUN cargo fetch 
RUN cargo build -p api --release
RUN cargo build -p identity --release

# Phase 2: Final minimal image
FROM ubuntu:24.04

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl-dev \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

ENV PATH="/root/.cargo/bin:/usr/local/bin:${PATH}"

# Create working directory
WORKDIR /usr/src/cortexbrain-agent

# Copy the compiled binary
COPY --from=builder /usr/src/app/agent/target/release/agent-api /usr/local/bin/agent-api
COPY --from=builder /usr/src/app/agent/target/release/identity /usr/local/bin/identity

# Copy configuration files
COPY conntracker /usr/src/cortexbrain-agent/conntracker

# Set env vars for your app
ENV BPF_PATH="/usr/src/cortexbrain-identity-service/conntracker"
ENV PIN_MAP_PATH="/sys/fs/bpf/cortexbrain-identity-service/"

# Default command
CMD ["agent-api"]
