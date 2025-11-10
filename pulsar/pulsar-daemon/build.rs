//! Build script to compile Protocol Buffer definitions

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Compile the terminal.proto file
    tonic_build::configure()
        .build_server(true)
        .build_client(false) // Client will be in frontend (gRPC-Web)
        .out_dir("src/generated")
        .compile(&["../proto/terminal.proto"], &["../proto"])?;

    println!("cargo:rerun-if-changed=../proto/terminal.proto");

    Ok(())
}
