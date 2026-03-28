fn main() -> Result<(), Box<dyn std::error::Error>> {
    let include_dirs = &["proto/"];

    // Compile dynamic_value.proto first (segment_files package)
    tonic_build::configure()
        .build_server(false)
        .compile_protos(&["proto/dynamic_value.proto"], include_dirs)?;

    // Compile query.proto, mapping the segment_files package to
    // the already-generated module re-exported from grpc::segment_proto
    tonic_build::configure()
        .build_server(false)
        .extern_path(".segment_files", "crate::grpc::segment_proto")
        .compile_protos(&["proto/query.proto"], include_dirs)?;

    Ok(())
}
