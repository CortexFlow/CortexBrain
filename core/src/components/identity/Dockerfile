# Build this Dockerfile from the `core` directory as context:
# docker build -f src/components/identity/Dockerfile -t identity:0.0.1 \
#   --provenance=false --sbom=false .

# ==============================================================================
# PHASE 1: Build the eBPF bytecode (Kernel-space)
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

RUN bpftool btf dump file /sys/kernel/btf/vmlinux format c > src/components/conntracker/vmlinux.h \
    && bindgen src/components/conntracker/vmlinux.h \
        -o src/components/conntracker/src/bindings.rs \
        --use-core \
        --allowlist-type 'sk_buff' \
        --opaque-type 'ieee80211_sband_iftype_data'

RUN cargo +nightly build -Z build-std=core --target bpfel-unknown-none --release -p conntracker


# ==============================================================================
# PHASE 2: Build the Main Identity Service Application (User-space)
# ==============================================================================
FROM rust:1.90 AS app-builder

WORKDIR /usr/src/app
COPY . .

RUN cargo build --release -p cortexflow_identity


# ==============================================================================
# PHASE 3: Final minimal image (Production Runtime)
# ==============================================================================
FROM ubuntu:24.04

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/cortexbrain-identity-service

COPY --from=app-builder /usr/src/app/target/release/cortexflow_identity /usr/local/bin/cortexflow-identity-service
COPY --from=ebpf-builder /usr/src/app/target/bpfel-unknown-none/release/conntracker /usr/src/cortexbrain-identity-service/conntracker

ENV BPF_PATH="/usr/src/cortexbrain-identity-service/conntracker"
ENV PIN_MAP_PATH="/sys/fs/bpf/maps"

CMD ["cortexflow-identity-service"]
