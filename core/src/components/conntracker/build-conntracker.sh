echo "🚀 Building connection tracker"

bpftool btf dump file /sys/kernel/btf/vmlinux format c > vmlinux.h

if [ $? -ne 0 ]; then
    echo "Error: Failed to dump BTF from vmlinux. Ensure bpftool is installed and has access to the kernel BTF."
    exit 1
fi

bindgen vmlinux.h -o src/bindings.rs --use-core --allowlist-type 'sk_buff'


if ! command -v bindgen &> /dev/null; then
    echo "bindgen not found, installing..."
    cargo install bindgen-cli
fi

cargo +nightly build -Z build-std=core --target bpfel-unknown-none --release --bin conntracker

rm -f vmlinux.h