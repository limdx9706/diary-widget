use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ConfigResponse {
    diary_dir: String,
}

#[derive(Debug, Deserialize)]
struct WriteDiaryPayload {
    date: String,
    time: String,
    title: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct WriteDiaryResponse {
    path: String,
}

#[tauri::command]
fn read_config() -> Result<ConfigResponse, String> {
    Ok(ConfigResponse {
        diary_dir: "未配置（最小可编译占位）".to_string(),
    })
}

#[tauri::command]
fn select_env_file() -> Result<ConfigResponse, String> {
    Ok(ConfigResponse {
        diary_dir: "未配置（最小可编译占位）".to_string(),
    })
}

#[tauri::command]
fn write_diary(payload: WriteDiaryPayload) -> Result<WriteDiaryResponse, String> {
    let safe_title = if payload.title.trim().is_empty() {
        "untitled".to_string()
    } else {
        payload.title.trim().to_string()
    };
    let path = format!(
        "stub://{}-{}-{}.md",
        payload.date.trim(),
        payload.time.trim(),
        safe_title
    );
    let _ = payload.content;
    Ok(WriteDiaryResponse { path })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            read_config,
            select_env_file,
            write_diary
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    run();
}
