pub fn update_cli() {
    println!("Updating CortexFlow CLI");
    println!("Looking for a newer version");
}

pub fn version() {
    println!("CortexFlow CLI v{}", env!("CARGO_PKG_VERSION"));
}

pub fn info() {}
