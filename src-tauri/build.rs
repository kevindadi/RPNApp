fn main() -> Result<(), Box<dyn std::error::Error>> {
    tauri_build::build();

    tonic_build::configure().build_server(false).compile(&["proto/rustmir.proto"], &["proto"])?;
    Ok(())
}
