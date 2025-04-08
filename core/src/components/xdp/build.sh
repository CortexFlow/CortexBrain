echo "🚀 Building xdp"
cargo +nightly build -Z build-std=core --target bpfel-unknown-none --release