use std::env;
use std::path::PathBuf;
use tonic_prost_build::configure;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_file = "protos/agent.proto";
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    configure()
        .build_client(true)
        .build_server(true)
        .file_descriptor_set_path(out_dir.join("agent_api_descriptor.bin"))
        .out_dir("./src")
        .compile_protos(&[proto_file], &["protos"])?;

    Ok(())
}
