use std::path::Path;

pub fn main() {
    let out_dir = "src/kernel";
    prost_build::Config::new()
        .out_dir(out_dir) // Specifica la directory di uscita del codice generato
        .compile_protos(&["src/kernel/proxy_msg.proto"], &["src"])
        .unwrap();
    let generated_file = Path::new(out_dir).join("proxy_msg.rs");
    if generated_file.exists(){
        println!("File generated");
    }
    else {
        println!("error generating the file");
    }
    println!("cargo:rerun-if-changed=src/kernel/proxy_msg.proto"); //compile only if the file proxy_msg.proto is changed
}
