//! Build script: run by Cargo BEFORE the crate is compiled.
//! It converts the .proto files into Rust code (structs + client/server traits).

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(
            &["../../proto/auth.proto", "../../proto/user.proto"],
            &["../../proto"], // root directory for proto imports
        )?;

    // Recompile if a .proto file changes
    println!("cargo:rerun-if-changed=../../proto");
    Ok(())
}
