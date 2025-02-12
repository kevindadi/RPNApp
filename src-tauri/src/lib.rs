// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use log::{debug, error};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::env;

use tonic::transport::Channel;

pub mod rustmir {
    include!(concat!(env!("OUT_DIR"), "/rustmir.rs"));
}

use rustmir::rust_mir_service_client::RustMirServiceClient;
use rustmir::RustSourceRequest;

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

// 添加一个辅助函数来获取命令路径
fn get_command_path(cmd: &str) -> Result<String, String> {
    // 首先检查环境变量
    if let Ok(path) = env::var(format!("{}_PATH", cmd.to_uppercase())) {
        return Ok(path);
    }

    // 然后检查预定义的路径
    let default_paths = match cmd {
        "rustc" => vec![
            "/Users/kevin/.rustup/toolchains/nightly-aarch64-apple-darwin/bin/rustc",
            "/Users/kevin/.cargo/bin/rustc",
        ],
        "pn" => vec![
            "/Users/kevin/.cargo/bin/pn",
        ],
        "dot" => vec![
            "/usr/local/bin/dot",
            "/opt/homebrew/bin/dot",
        ],
        _ => vec![],
    };

    for path in default_paths {
        if std::path::Path::new(path).exists() {
            return Ok(path.to_string());
        }
    }

    Err(format!("Command {} not found", cmd))
}

#[tauri::command]
async fn generate_mir(source_code: String) -> Result<String, String> {
    let temp_dir = tempfile::tempdir()
        .map_err(|e| format!("Failed to create temp directory: {}", e))?;
    let source_path = temp_dir.path().join("main.rs");
    fs::write(&source_path, source_code)
        .map_err(|e| format!("Failed to write source code: {}", e))?;

    let rustc_path = get_command_path("rustc")?;
    let output = Command::new(rustc_path)
        .arg("--crate-name=rust_mir")
        .arg("-Zunpretty=mir")
        .arg(&source_path)
        .output()
        .map_err(|e| format!("Failed to execute rustc: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Ok(String::from_utf8_lossy(&output.stderr).into_owned())
    }
}

// #[tauri::command]
// async fn generate_mir(source_code: String) -> Result<String, String> {
//     let endpoint = "http://106.14.126.126:50051";
//     let channel = Channel::from_shared(endpoint.to_string())
//         .map_err(|e| format!("无效的 endpoint URL: {}", e))?
//         .connect()
//         .await
//         .map_err(|e| format!("连接服务器失败: {}", e))?;

//     let mut client = RustMirServiceClient::new(channel);

//     let request = tonic::Request::new(RustSourceRequest {
//         source_code,
//     });

//     let response = client
//         .get_mir(request)
//         .await
//         .map_err(|e| format!("RPC 调用失败: {}", e))?;
    
//     let mir_response = response.into_inner();
//     Ok(mir_response.content)
// }

// 获取用户临时目录的辅助函数
fn get_temp_dir() -> Result<PathBuf, String> {
    let home = env::var("HOME").map_err(|e| format!("Failed to get HOME directory: {}", e))?;
    let temp_dir = PathBuf::from(home).join(".rust_analyzer_temp");
    
    // 如果目录已存在，先删除它
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir)
            .map_err(|e| format!("Failed to remove existing temp directory: {}", e))?;
    }
    
    // 创建新的临时目录
    fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("Failed to create temp directory: {}", e))?;
    
    Ok(temp_dir)
}

// 修改 save_source_code 函数
async fn save_source_code(code: String) -> Result<PathBuf, String> {
    let tmp_dir = get_temp_dir()?;
    let tmp_file = tmp_dir.join("main.rs");
    fs::write(tmp_file, code)
        .map_err(|e| format!("Failed to write source code: {}", e))?;
    Ok(tmp_dir)
}

#[tauri::command]
async fn run_pn_analysis(source_code: String, mode: String) -> Result<serde_json::Value, String> {
    let tmp_dir = save_source_code(source_code).await?;
    println!("tmp_dir: {:?}", tmp_dir);
    let source_file = tmp_dir.join("main.rs");
    let graph_dot = tmp_dir.join("tmp").join("main").join("graph.dot");
    println!("graph_dot path: {:?}", graph_dot);
    
    let pn_flags = format!("{} -p main --pn-analysis-dir={}/tmp/ --viz-petrinet", mode, tmp_dir.to_string_lossy()    );
    println!("PN FLAGS ARE: {:?}", pn_flags);
    let dyld_path = "/Users/kevin/.rustup/toolchains/nightly-2024-12-11-aarch64-apple-darwin/lib:$DYLD_LIBRARY_PATH".to_string();

    let pn_path = get_command_path("pn")?;
    let output = Command::new(pn_path)
        .env("PN_FLAGS", pn_flags)
        .env("DYLD_LIBRARY_PATH", &dyld_path)
        .arg(&source_file)
        .output()
        .map_err(|e| format!("Failed to execute pn: {}", e))?;

    let dot_path = get_command_path("dot")?;
    let mut dot_process = Command::new(dot_path)
        .args(["-Tsvg"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to execute dot: {}", e))?;

    // 读取 dot 文件内容
    let dot_content = fs::read_to_string(&graph_dot)
        .map_err(|e| format!("Failed to read dot file: {}", e))?;

    // 将 dot 内容写入到 dot 命令的标准输入
    if let Some(mut stdin) = dot_process.stdin.take() {
        use std::io::Write;
        stdin.write_all(dot_content.as_bytes())
            .map_err(|e| e.to_string())?;
    }

    // 获取 svg 输出
    let output_svg = dot_process.wait_with_output()
        .map_err(|e| e.to_string())?;
    
    let svg_content = String::from_utf8_lossy(&output_svg.stdout).to_string();

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
            run_pn_analysis
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
