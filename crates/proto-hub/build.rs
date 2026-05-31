fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::compile_protos("proto/auth.proto")?;
    tonic_prost_build::compile_protos("proto/tracking.proto")?;
    tonic_prost_build::compile_protos("proto/price_receiver.proto")?;
    Ok(())
}