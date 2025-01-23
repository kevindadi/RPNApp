// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use log::{debug, error};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn read_example(filename: &str) -> Result<String, String> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("examples");
    path.push(filename);

    debug!("Attempting to read file: {:?}", path);

    if !path.exists() {
        return Err(format!("File not found: {:?}", path));
    }

    match fs::read_to_string(&path) {
        Ok(content) => {
            debug!("Successfully read file with {} bytes", content.len());
            Ok(content)
        }
        Err(e) => {
            error!("Failed to read file: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn generate_mir(source_code: String) -> Result<String, String> {
    // 创建临时目录
    let temp_dir =
        tempfile::tempdir().map_err(|e| format!("Failed to create temp directory: {}", e))?;

    // 创建固定名称的源文件
    let source_path = temp_dir.path().join("main.rs");
    fs::write(&source_path, source_code)
        .map_err(|e| format!("Failed to write source code: {}", e))?;

    // 运行 rustc 命令生成 MIR
    let output = Command::new("rustc")
        .arg("+nightly")
        .arg("--crate-name=rust_mir")
        .arg("-Zunpretty=mir")
        .arg(&source_path)
        .output()
        .map_err(|e| format!("Failed to execute rustc: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        // 如果编译失败，返回错误信息
        Ok(String::from_utf8_lossy(&output.stderr).into_owned())
    }
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, read_example, generate_mir])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
