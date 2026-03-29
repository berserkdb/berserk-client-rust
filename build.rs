fn main() -> Result<(), Box<dyn std::error::Error>> {
    let include_dirs = &["proto/"];

    // Compile dynamic_value.proto first (berserk package)
    tonic_build::configure()
        .build_server(false)
        .compile_protos(&["proto/dynamic_value.proto"], include_dirs)?;

    // Compile query.proto, mapping the berserk package to
    // the already-generated module re-exported from grpc::berserk_proto
    tonic_build::configure()
        .build_server(false)
        .extern_path(".berserk", "crate::grpc::berserk_proto")
        .compile_protos(&["proto/query.proto"], include_dirs)?;

    Ok(())
}
