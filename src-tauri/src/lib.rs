// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::env;
use serde::{ Serialize, Deserialize};
use serde_json::json;


const ENDPOINT: &str = "http://106.14.126.126:8080";

#[derive(Debug, Serialize, Deserialize)]
struct RustSourceRequest {
    source_code: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct MirResponse {
    success: bool,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ListFilesRequest {
    dir_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ListFilesResponse {
    filenames: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PnAnalysisRequest {
    source_code: String,
    mode: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PnAnalysisResponse {
    graph_content: String,
    output: String,
    error: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct FileContentRequest {
    file_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct FileContentResponse {
    content: String,
}

#[tauri::command]
async fn list_files(dir_path: String) -> Result<ListFilesResponse, String> {    
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/list_files", ENDPOINT))
        .json(&json!({ "dir_path": dir_path }))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    let files: ListFilesResponse = response.json()
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(files)
}

#[tauri::command]
async fn get_file_content(filename: &str) -> Result<String, String> {
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/get_file_content", ENDPOINT))
        .json(&json!({ "file_path": format!("/home/kevin/KevinServer/examples/{}", filename) }))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    let content: serde_json::Value = response.json()
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(content["content"].as_str().unwrap_or("").to_string())
}

#[tauri::command]
async fn get_mir(source_code: String) -> Result<String, String> {
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/get_mir", ENDPOINT))
        .json(&json!({
            "source_code": source_code,
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    let result: serde_json::Value = response.json()
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(result["content"].as_str().unwrap_or("").to_string())
}

#[tauri::command]
async fn run_pn_analysis(source_code: String, mode: String) -> Result<PnAnalysisResponse, String> {
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/run_pn_analysis", ENDPOINT))
        .json(&json!({
            "source_code": source_code,
            "mode": mode
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    let result: PnAnalysisResponse = response.json()
        .await
        .map_err(|e| e.to_string())?;
    // println!("{:?}", result);
    Ok(result)
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_file_content,
            get_mir,
            run_pn_analysis,
            list_files
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
