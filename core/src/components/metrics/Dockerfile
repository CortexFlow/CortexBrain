# Build this Dockerfile from the `core` directory as context:
# docker build -f src/components/metrics/Dockerfile -t metrics:0.0.1 \
#   --provenance=false --sbom=false .

# ==============================================================================
# PHASE 1: Build the eBPF bytecode (Kernel-space Metrics Tracer)
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

RUN bpftool btf dump file /sys/kernel/btf/vmlinux format c > src/components/metrics_tracer/vmlinux.h \
    && bindgen --use-core --no-layout-tests \
        --allowlist-type 'sk_buff' \
        --allowlist-type 'sock_common' \
        --allowlist-type 'in6_addr' \
        src/components/metrics_tracer/vmlinux.h \
        -o src/components/metrics_tracer/src/bindings.rs

RUN cargo +nightly build -Z build-std=core --target bpfel-unknown-none --release -p metrics_tracer

# ==============================================================================
# PHASE 2: Build the Main Metrics Service Application (User-space)
# ==============================================================================
FROM rust:1.90 AS app-builder

WORKDIR /usr/src/app
COPY . .

RUN cargo build --release -p cortexflow-metrics


# ==============================================================================
# PHASE 3: Final minimal image (Production Runtime)
# ==============================================================================
FROM ubuntu:24.04

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/cortexbrain-metrics

COPY --from=app-builder /usr/src/app/target/release/cortexflow-metrics /usr/local/bin/cortexflow-metrics
COPY --from=ebpf-builder /usr/src/app/target/bpfel-unknown-none/release/metrics_tracer /usr/src/cortexbrain-metrics/metrics_tracer

ENV BPF_PATH="/usr/src/cortexbrain-metrics/metrics_tracer"
ENV PIN_MAP_PATH="/sys/fs/bpf/trace_maps"

CMD ["cortexflow-metrics"]