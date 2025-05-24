echo "🚀 Building connection tracker"
cargo +nightly build -Z build-std=core --target bpfel-unknown-none --release --bin conntracker
