# ==============================================================================
# PHASE 1: Build the eBPF bytecode (Kernel-space Connection Tracker)
# ==============================================================================
FROM rust:1.90 AS ebpf-builder

RUN apt-get update && apt-get install -y \
    clang \
    libclang-dev \
    llvm \
    bpftool \
    && rm -rf /var/lib/apt/lists/*

RUN cargo install --locked bindgen-cli
RUN cargo install --locked bpf-linker
RUN rustup toolchain install nightly --component rust-src

WORKDIR /usr/src/app
COPY . .

# Generate bindings from local Linux VM host kernel definitions
RUN bpftool btf dump file /sys/kernel/btf/vmlinux format c > src/components/conntracker/vmlinux.h \
    && bindgen src/components/conntracker/vmlinux.h \
        -o src/components/conntracker/src/bindings.rs \
        --use-core \
        --allowlist-type 'sk_buff' \
        --opaque-type 'ieee80211_sband_iftype_data'

# Compile the raw eBPF connection tracker target
RUN cargo +nightly build -Z build-std=core --target bpfel-unknown-none --release -p conntracker


# ==============================================================================
# PHASE 2: Build the Userspace Binaries (agent-api & cortexflow_identity)
# ==============================================================================
FROM rust:1.90 AS app-builder

# Install system dependencies including protoc for gRPC/Protobuf generation
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

WORKDIR /usr/src/app
COPY . .

# Fetch dependencies and build release components
RUN cargo fetch
RUN cargo build -p cortexflow_agent_api --release
RUN cargo build -p cortexflow_identity --release


# ==============================================================================
# PHASE 3: Final Minimal Production Runtime Image
# ==============================================================================
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

# 1. Copy the compiled binaries from Phase 2 (App Builder)
COPY --from=app-builder /usr/src/app/target/release/agent-api /usr/local/bin/agent-api
COPY --from=app-builder /usr/src/app/target/release/cortexflow_identity /usr/local/bin/cortexflow_identity

# 2. Copy ONLY the compiled raw eBPF binary from Phase 1 (eBPF Builder)
COPY --from=ebpf-builder /usr/src/app/target/bpfel-unknown-none/release/conntracker /usr/src/cortexbrain-agent/conntracker

# Set env vars for your app
ENV BPF_PATH="/usr/src/cortexbrain-agent/conntracker"
ENV PIN_MAP_PATH="/sys/fs/bpf/cortexbrain-identity-service/"
ENV PIN_BLOCKLIST_MAP_PATH="/sys/fs/bpf/cortexbrain-agent/"

# Default command
CMD ["agent-api"]