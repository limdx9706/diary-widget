use std::path::{Path, PathBuf};
use tauri_plugin_dialog::DialogExt;

use crate::config::{self, ConfigResponse};
use crate::diary::{self, WriteDiaryPayload, WriteDiaryResponse};

#[tauri::command]
pub fn read_config() -> Result<ConfigResponse, String> {
    let cfg = config::read_cached_config()?;
    let env_path = PathBuf::from(cfg.env_file_path);
    let diary_dir = config::diary_dir_from_env_path(&env_path)?;
    Ok(ConfigResponse { diary_dir })
}

#[tauri::command]
pub fn select_env_file(app: tauri::AppHandle) -> Result<ConfigResponse, String> {
    let file = app
        .dialog()
        .file()
        .add_filter("Env", &["env"])
        .blocking_pick_file()
        .ok_or_else(|| "未选择文件".to_string())?;
    let env_path = file
        .into_path()
        .map_err(|_| "无法读取选择的文件路径".to_string())?;
    let diary_dir = config::diary_dir_from_env_path(&env_path)?;
    config::save_cached_config(&env_path)?;
    Ok(ConfigResponse { diary_dir })
}

#[tauri::command]
pub fn write_diary(payload: WriteDiaryPayload) -> Result<WriteDiaryResponse, String> {
    let cfg = config::read_cached_config()?;
    let env_path = PathBuf::from(cfg.env_file_path);
    let diary_dir = config::diary_dir_from_env_path(&env_path)?;
    let path = diary::write_diary_to_file(Path::new(&diary_dir), &payload)?;
    Ok(WriteDiaryResponse {
        path: path.to_string_lossy().to_string(),
    })
}
