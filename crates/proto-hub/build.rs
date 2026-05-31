use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    tonic_prost_build::configure()
        .file_descriptor_set_path(out_dir.join("tracking_descriptor.bin"))
        .compile_protos(&["proto/tracking.proto"], &[""])?;
    tonic_prost_build::configure()
        .file_descriptor_set_path(out_dir.join("auth_descriptor.bin"))
        .compile_protos(&["proto/auth.proto"], &[""])?;
    tonic_prost_build::configure()
        .file_descriptor_set_path(out_dir.join("price_receiver_descriptor.bin"))
        .compile_protos(&["proto/price_receiver.proto"], &[""])?;
    Ok(())
}