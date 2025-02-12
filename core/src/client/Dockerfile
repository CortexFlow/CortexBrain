# Phase 1: Build image
FROM rust:1.83 AS builder

# Set working directory
WORKDIR /usr/src/app

# First, create the shared project structure
WORKDIR /usr/src/app/shared
COPY .shared/Cargo.toml .
COPY .shared/src ./src

# Then create the client project structure
WORKDIR /usr/src/app/client
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

# Create directory for the client
WORKDIR /usr/src/cortexbrain-client

# Copy the binary from builder
COPY --from=builder /usr/src/app/client/target/release/client /usr/local/bin/cortexflow-client

# Copy config file
COPY config.yaml /usr/src/cortexbrain-client/config.yaml

# Set config path environment variable
ENV CONFIG_PATH="/usr/src/cortexbrain-client/config.yaml"

# Set the client execution command
CMD ["cortexflow-client"]
