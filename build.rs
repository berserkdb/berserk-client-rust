// Proto codegen is only needed for the grpc transport. The whole
// script is feature-gated so http-only builds neither compile
// tonic-build nor require `protoc` on the build host.

#[cfg(feature = "grpc")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let include_dirs = &["proto/"];

    // Compile the berserk-package protos first; query.proto depends on
    // them via `import "common_api.proto"` and the extern_path mapping below.
    tonic_build::configure()
        .build_server(false)
        .compile_protos(
            &["proto/common_api.proto", "proto/dynamic_value.proto"],
            include_dirs,
        )?;

    // Compile query.proto, mapping the berserk package to
    // the already-generated module re-exported from grpc::berserk_proto
    tonic_build::configure()
        .build_server(false)
        .extern_path(".berserk", "crate::grpc::berserk_proto")
        .compile_protos(&["proto/query.proto"], include_dirs)?;

    Ok(())
}

#[cfg(not(feature = "grpc"))]
fn main() {}
