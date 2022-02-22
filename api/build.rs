fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("../proto/grpc/simple_service/v1/simple_service.proto")?;
    Ok(())
}
