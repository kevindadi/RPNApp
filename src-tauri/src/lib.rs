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

// Only for example test
#[tauri::command]
async fn save_source_code(code: String) -> Result<(), String> {
    let tmp_dir = "/tmp/main";
    fs::create_dir_all(tmp_dir).map_err(|e| e.to_string())?;
    fs::write(format!("{}/main.rs", tmp_dir), code).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn run_pn_analysis(mode: String) -> Result<serde_json::Value, String> {
    let tmp_dir = "/tmp/main";
    let pn_flags = format!("{} -p main --viz-petrinet", mode);
    let dyld_path =
        "/Users/kevin/.rustup/toolchains/nightly-2024-12-11-aarch64-apple-darwin/lib:$DYLD_LIBRARY_PATH".to_string();

    
    // 执行 PN 分析命令
    let output = Command::new("/Users/kevin/.cargo/bin/pn")
        .env("PN_FLAGS", pn_flags)
        .env("DYLD_LIBRARY_PATH", &dyld_path)
        .arg("/tmp/main/main.rs")
        .output()
        .map_err(|e| e.to_string())?;

    // 将 dot 转换为 png
    Command::new("dot")
        .args(["-Tsvg", "-o", "/tmp/main/graph.svg"])
        .arg("/tmp/main/graph.dot")
        .output()
        .map_err(|e| e.to_string())?;

    let svg_content = fs::read_to_string("/tmp/main/graph.svg")
        .map_err(|e| e.to_string())?;
    // 构建返回结果
    let result = serde_json::json!({
        "graphContent": svg_content,
        "output": String::from_utf8_lossy(&output.stdout).to_string(),
        "error": String::from_utf8_lossy(&output.stderr).to_string()
    });

    Ok(result)
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            greet,
            read_example,
            generate_mir,
            save_source_code,
            run_pn_analysis
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
