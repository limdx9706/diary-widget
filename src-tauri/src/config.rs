use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const DIARY_DIR_NAME: &str = "日记";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigResponse {
    pub diary_dir: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredConfig {
    pub env_file_path: String,
}

pub fn app_config_dir() -> Result<PathBuf, String> {
    let base = dirs::config_dir().ok_or_else(|| "无法确定配置目录".to_string())?;
    let dir = base.join("diary-widget");
    fs::create_dir_all(&dir).map_err(|e| format!("创建配置目录失败: {e}"))?;
    Ok(dir)
}

pub fn app_config_file() -> Result<PathBuf, String> {
    Ok(app_config_dir()?.join("config.json"))
}

pub fn parse_rag_base_path_from_env(env_path: &Path) -> Result<String, String> {
    let raw = fs::read_to_string(env_path).map_err(|e| format!("读取 .env 失败: {e}"))?;
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }
        if let Some((key, value)) = trimmed.split_once('=') {
            if key.trim() == "RAG_BASE_PATH" {
                let clean = value.trim().trim_matches('"').trim_matches('\'').to_string();
                if clean.is_empty() {
                    return Err("RAG_BASE_PATH 为空".to_string());
                }
                return Ok(clean);
            }
        }
    }
    Err("未在 .env 中找到 RAG_BASE_PATH".to_string())
}

pub fn diary_dir_from_env_path(env_path: &Path) -> Result<String, String> {
    let base = parse_rag_base_path_from_env(env_path)?;
    let base_path = Path::new(&base);
    let resolved_base = if base_path.is_absolute() {
        base_path.to_path_buf()
    } else {
        let parent = env_path
            .parent()
            .ok_or_else(|| "无法确定 .env 所在目录".to_string())?;
        parent.join(base_path)
    };
    Ok(resolved_base
        .join(DIARY_DIR_NAME)
        .to_string_lossy()
        .to_string())
}

pub fn read_cached_config() -> Result<StoredConfig, String> {
    let cfg = app_config_file()?;
    let raw = fs::read_to_string(cfg).map_err(|e| format!("读取配置缓存失败: {e}"))?;
    serde_json::from_str::<StoredConfig>(&raw).map_err(|e| format!("解析配置缓存失败: {e}"))
}

pub fn save_cached_config(env_file_path: &Path) -> Result<(), String> {
    let cfg = app_config_file()?;
    let payload = StoredConfig {
        env_file_path: env_file_path.to_string_lossy().to_string(),
    };
    let json =
        serde_json::to_string_pretty(&payload).map_err(|e| format!("序列化配置失败: {e}"))?;
    fs::write(cfg, json).map_err(|e| format!("写入配置缓存失败: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;

    fn make_temp_dir(name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "diary_widget_test_{}_{}",
            name,
            Local::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        fs::create_dir_all(&p).expect("create temp");
        p
    }

    #[test]
    fn relative_rag_base_path_should_resolve_from_env_parent() {
        let raw = "RAG_BASE_PATH=./data/knowledge\n";
        let env_dir = make_temp_dir("env_relative");
        let real_env = env_dir.join(".env");
        fs::write(&real_env, raw).expect("write env");
        let diary_dir = diary_dir_from_env_path(&real_env).expect("resolve diary dir");
        assert!(diary_dir.replace("\\", "/").ends_with("/data/knowledge/日记"));
    }
}
