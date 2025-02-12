// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::env;

use tonic::transport::Channel;

pub mod rustmir {
    include!(concat!(env!("OUT_DIR"), "/rustmir.rs"));
}

use rustmir::rust_mir_service_client::RustMirServiceClient;
use rustmir::RustSourceRequest; 
use rustmir::ListFilesRequest;
use rustmir::FileContentRequest;
use rustmir::PnAnalysisRequest;

const ENDPOINT: &str = "http://106.14.126.126:50051";

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[derive(Debug,serde::Serialize)]
pub struct FileListResponse {
    filenames: Vec<String>
}

#[tauri::command]
async fn list_files(dir_path: String) -> Result<FileListResponse, String> {    
    let channel = Channel::from_shared(ENDPOINT.to_string())
        .map_err(|e| format!("无效的 endpoint URL: {}", e))?
        .connect()
        .await
        .map_err(|e| format!("连接服务器失败: {}", e))?;

    let mut client = RustMirServiceClient::new(channel);

    let request = tonic::Request::new(ListFilesRequest {
        dir_path,
    });
    
    let response = client.list_files(request).await
        .map_err(|e| e.to_string())
        .map(|response| response.into_inner())?;

    let filelists = 
    FileListResponse {
        filenames: response.filenames
    };
    println!("filelists: {:?}", filelists);
    Ok(filelists)
}

#[tauri::command]
async fn read_example(filename: &str) -> Result<String, String> {
    let channel = Channel::from_shared(ENDPOINT.to_string())
        .map_err(|e| format!("无效的 endpoint URL: {}", e))?
        .connect()
        .await
        .map_err(|e| format!("连接服务器失败: {}", e))?;

    let mut client = RustMirServiceClient::new(channel);

    let full_path = format!("/home/kevin/KevinServer/examples/{}", filename);
    let request = tonic::Request::new(FileContentRequest {
        file_path: full_path,
    });
    
    let response = client.get_file_content(request).await
        .map_err(|e| e.to_string())?
        .into_inner();

    Ok(response.content)
}

#[tauri::command]
async fn generate_mir(source_code: String) -> Result<String, String> {
    let channel = Channel::from_shared(ENDPOINT.to_string())
        .map_err(|e| format!("无效的 endpoint URL: {}", e))?
        .connect()
        .await
        .map_err(|e| format!("连接服务器失败: {}", e))?;

    let mut client = RustMirServiceClient::new(channel);

    let request = tonic::Request::new(RustSourceRequest {
        source_code,
    });

    let response = client
        .get_mir(request)
        .await
        .map_err(|e| format!("RPC 调用失败: {}", e))?;
    
    let mir_response = response.into_inner();
    Ok(mir_response.content)
}

#[tauri::command]
async fn run_pn_analysis(source_code: String, mode: String) -> Result<serde_json::Value, String> {
    let channel = Channel::from_shared(ENDPOINT)
    .map_err(|e| format!("无效的 endpoint URL: {}", e))?
    .connect()
    .await
    .map_err(|e| format!("连接服务器失败: {}", e))?;

    let mut client = RustMirServiceClient::new(channel);

    let request = tonic::Request::new(PnAnalysisRequest {
        source_code,
        mode,
    });

    let response = client
        .run_pn_analysis(request)
        .await
        .map_err(|e| format!("RPC 调用失败: {}", e))?;

    let analysis_result = response.into_inner();

    let result = serde_json::json!({
        "graphContent": analysis_result.graph_content,
        "output": analysis_result.output,
        "error": analysis_result.error
    });

    Ok(result)
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            greet,
            read_example,
            generate_mir,
            run_pn_analysis,
            list_files
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
