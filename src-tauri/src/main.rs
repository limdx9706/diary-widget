use chrono::{Datelike, Local, NaiveDate};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri_plugin_dialog::DialogExt;

const DIARY_DIR_NAME: &str = "日记";
const ENTRY_H1_PATTERN: &str = r"(?m)^#\s+(\d{1,2}):(\d{2})\s*$";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ConfigResponse {
    diary_dir: String,
}

#[derive(Debug, Serialize)]
struct WriteDiaryResponse {
    path: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct StoredConfig {
    env_file_path: String,
}

#[derive(Debug, Deserialize)]
struct WriteDiaryPayload {
    date: String,
    time: String,
    title: String,
    content: String,
}

#[derive(Debug, Clone)]
struct EntrySection {
    time_minutes: i32,
    raw: String,
}

fn app_config_file() -> Result<PathBuf, String> {
    let appdata = std::env::var("APPDATA").map_err(|_| "无法读取 APPDATA 环境变量".to_string())?;
    let dir = Path::new(&appdata).join("diary-widget");
    fs::create_dir_all(&dir).map_err(|e| format!("创建配置目录失败: {e}"))?;
    Ok(dir.join("config.json"))
}

fn parse_rag_base_path_from_env(env_path: &Path) -> Result<String, String> {
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

fn diary_dir_from_env_path(env_path: &Path) -> Result<String, String> {
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

fn read_cached_config() -> Result<StoredConfig, String> {
    let cfg = app_config_file()?;
    let raw = fs::read_to_string(cfg).map_err(|e| format!("读取配置缓存失败: {e}"))?;
    serde_json::from_str::<StoredConfig>(&raw).map_err(|e| format!("解析配置缓存失败: {e}"))
}

fn save_cached_config(env_file_path: &Path) -> Result<(), String> {
    let cfg = app_config_file()?;
    let payload = StoredConfig {
        env_file_path: env_file_path.to_string_lossy().to_string(),
    };
    let json = serde_json::to_string_pretty(&payload).map_err(|e| format!("序列化配置失败: {e}"))?;
    fs::write(cfg, json).map_err(|e| format!("写入配置缓存失败: {e}"))
}

fn shift_headings_down_one_if_has_h1(content: &str) -> (String, bool) {
    let has_h1 = Regex::new(r"(?m)^#\s+[^#]").expect("regex compile").is_match(content);
    if !has_h1 {
        return (content.to_string(), false);
    }
    let shifted = Regex::new(r"(?m)^(#{1,5})\s+")
        .expect("regex compile")
        .replace_all(content, "#$1 ")
        .to_string();
    (shifted, true)
}

fn parse_diary_body_entries(body: &str) -> (String, Vec<EntrySection>) {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return (String::new(), vec![]);
    }
    let re = Regex::new(ENTRY_H1_PATTERN).expect("regex compile");
    let matches: Vec<_> = re.find_iter(trimmed).collect();
    if matches.is_empty() {
        return (trimmed.to_string(), vec![]);
    }

    let first_index = matches.first().map(|m| m.start()).unwrap_or(0);
    let prefix = trimmed[..first_index].trim_end().to_string();
    let mut entries = Vec::with_capacity(matches.len());

    for (i, m) in matches.iter().enumerate() {
        let start = m.start();
        let end = if i + 1 < matches.len() {
            matches[i + 1].start()
        } else {
            trimmed.len()
        };
        let raw = trimmed[start..end].trim_end().to_string();
        let caps = re.captures(&trimmed[start..end]).expect("entry capture");
        let hour: i32 = caps
            .get(1)
            .and_then(|x| x.as_str().parse::<i32>().ok())
            .unwrap_or(0);
        let minute: i32 = caps
            .get(2)
            .and_then(|x| x.as_str().parse::<i32>().ok())
            .unwrap_or(0);
        entries.push(EntrySection {
            time_minutes: hour * 60 + minute,
            raw,
        });
    }
    (prefix, entries)
}

fn build_entry_block(time: &str, shifted_content: &str, title: &str) -> String {
    let mut block = format!("# {time}\n");
    if !title.trim().is_empty() {
        block.push('\n');
        block.push_str(&format!("## {}\n", title.trim()));
    }
    let body = shifted_content.trim_end();
    if !body.is_empty() {
        block.push('\n');
        block.push_str(body);
        block.push('\n');
    }
    block
}

fn should_append_without_sorting(existing_entries: &[EntrySection], new_time_minutes: i32) -> bool {
    if existing_entries.is_empty() {
        return false;
    }
    let last = existing_entries.last().expect("last entry exists");
    new_time_minutes >= last.time_minutes
}

fn date_title_and_filename(date: &NaiveDate) -> (String, String) {
    let title = format!("{}年{}月{}日日记", date.year(), date.month(), date.day());
    let file_name = format!("{title}.md");
    (title, file_name)
}

fn write_diary_to_file(base_diary_dir: &Path, payload: &WriteDiaryPayload) -> Result<PathBuf, String> {
    let date = NaiveDate::parse_from_str(payload.date.trim(), "%Y-%m-%d")
        .map_err(|_| "date 格式应为 YYYY-MM-DD".to_string())?;
    let time_parts: Vec<_> = payload.time.trim().split(':').collect();
    if time_parts.len() != 2 {
        return Err("time 格式应为 HH:MM".to_string());
    }
    let hour: i32 = time_parts[0]
        .parse::<i32>()
        .map_err(|_| "time 小时无效".to_string())?;
    let minute: i32 = time_parts[1]
        .parse::<i32>()
        .map_err(|_| "time 分钟无效".to_string())?;
    if !(0..=23).contains(&hour) || !(0..=59).contains(&minute) {
        return Err("time 超出有效范围".to_string());
    }
    let time_str = format!("{:02}:{:02}", hour, minute);
    let new_minutes = hour * 60 + minute;

    fs::create_dir_all(base_diary_dir).map_err(|e| format!("创建日记目录失败: {e}"))?;
    let (diary_title, file_name) = date_title_and_filename(&date);
    let file_path = base_diary_dir.join(file_name);

    let now_iso = Local::now().to_rfc3339();
    let (shifted_content, _) = shift_headings_down_one_if_has_h1(&payload.content);
    let entry_block = build_entry_block(&time_str, &shifted_content, &payload.title);

    let new_raw = if !file_path.exists() {
        let frontmatter = serde_json::json!({
            "title": diary_title,
            "date": date.format("%Y-%m-%d").to_string(),
            "tags": ["日记"],
            "entryCount": 1,
            "lastModified": now_iso,
        });
        format!(
            "---\n{}\n---\n\n{}",
            serde_yaml::to_string(&frontmatter).map_err(|e| format!("序列化 frontmatter 失败: {e}"))?,
            entry_block.trim_end()
        ) + "\n"
    } else {
        let existing = fs::read_to_string(&file_path).map_err(|e| format!("读取日记文件失败: {e}"))?;
        let fm_re = Regex::new(r"(?s)^---\s*\n(.*?)\n---\s*\n?").expect("frontmatter regex");
        let (existing_fm, body) = if let Some(caps) = fm_re.captures(&existing) {
            let whole = caps.get(0).map(|x| x.as_str()).unwrap_or("");
            let front = caps.get(1).map(|x| x.as_str()).unwrap_or("");
            (front.to_string(), existing[whole.len()..].to_string())
        } else {
            (String::new(), existing)
        };

        let mut fm_value: serde_yaml::Value = if existing_fm.trim().is_empty() {
            serde_yaml::to_value(serde_json::json!({})).expect("empty yaml")
        } else {
            serde_yaml::from_str(&existing_fm).unwrap_or_else(|_| serde_yaml::to_value(serde_json::json!({})).expect("yaml object"))
        };

        let (prefix, entries) = parse_diary_body_entries(&body);
        let append_without_sort = should_append_without_sorting(&entries, new_minutes);
        let next_count = entries.len() + 1;

        if let serde_yaml::Value::Mapping(ref mut map) = fm_value {
            map.insert(
                serde_yaml::Value::String("title".to_string()),
                serde_yaml::Value::String(diary_title.clone()),
            );
            map.insert(
                serde_yaml::Value::String("date".to_string()),
                serde_yaml::Value::String(date.format("%Y-%m-%d").to_string()),
            );
            map.insert(
                serde_yaml::Value::String("tags".to_string()),
                serde_yaml::to_value(vec!["日记"]).expect("yaml tags"),
            );
            map.insert(
                serde_yaml::Value::String("entryCount".to_string()),
                serde_yaml::Value::Number(serde_yaml::Number::from(next_count as i64)),
            );
            map.insert(
                serde_yaml::Value::String("lastModified".to_string()),
                serde_yaml::Value::String(now_iso.clone()),
            );
        }

        let new_body = if append_without_sort {
            let base = if prefix.is_empty() {
                entries.iter().map(|e| e.raw.clone()).collect::<Vec<_>>().join("\n\n")
            } else {
                format!(
                    "{prefix}\n\n{}",
                    entries.iter().map(|e| e.raw.clone()).collect::<Vec<_>>().join("\n\n")
                )
            };
            format!("{}\n\n{}", base.trim_end(), entry_block.trim_end())
        } else {
            let mut next_entries = entries;
            next_entries.push(EntrySection {
                time_minutes: new_minutes,
                raw: entry_block.trim_end().to_string(),
            });
            next_entries.sort_by_key(|e| e.time_minutes);
            let sorted_body = next_entries
                .iter()
                .map(|e| e.raw.clone())
                .collect::<Vec<_>>()
                .join("\n\n");
            if prefix.is_empty() {
                sorted_body
            } else {
                format!("{prefix}\n\n{sorted_body}")
            }
        };

        format!(
            "---\n{}\n---\n\n{}\n",
            serde_yaml::to_string(&fm_value).map_err(|e| format!("写回 frontmatter 失败: {e}"))?,
            new_body.trim_end()
        )
    };

    fs::write(&file_path, new_raw).map_err(|e| format!("写入日记文件失败: {e}"))?;
    Ok(file_path)
}

#[tauri::command]
fn read_config() -> Result<ConfigResponse, String> {
    let cfg = read_cached_config()?;
    let env_path = PathBuf::from(cfg.env_file_path);
    let diary_dir = diary_dir_from_env_path(&env_path)?;
    Ok(ConfigResponse { diary_dir })
}

#[tauri::command]
fn select_env_file(app: tauri::AppHandle) -> Result<ConfigResponse, String> {
    let file = app
        .dialog()
        .file()
        .add_filter("Env", &["env"])
        .blocking_pick_file()
        .ok_or_else(|| "未选择文件".to_string())?;
    let env_path = file
        .into_path()
        .map_err(|_| "无法读取选择的文件路径".to_string())?;
    let diary_dir = diary_dir_from_env_path(&env_path)?;
    save_cached_config(&env_path)?;
    Ok(ConfigResponse { diary_dir })
}

#[tauri::command]
fn write_diary(payload: WriteDiaryPayload) -> Result<WriteDiaryResponse, String> {
    let cfg = read_cached_config()?;
    let env_path = PathBuf::from(cfg.env_file_path);
    let diary_dir = diary_dir_from_env_path(&env_path)?;
    let path = write_diary_to_file(Path::new(&diary_dir), &payload)?;
    Ok(WriteDiaryResponse {
        path: path.to_string_lossy().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    fn make_temp_dir(name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("diary_widget_test_{}_{}", name, Local::now().timestamp_nanos_opt().unwrap_or_default()));
        fs::create_dir_all(&p).expect("create temp");
        p
    }

    #[test]
    fn shift_headings_no_h1_unchanged() {
        let text = "## title\ntext";
        let (shifted, applied) = shift_headings_down_one_if_has_h1(text);
        assert!(!applied);
        assert_eq!(shifted, text);
    }

    #[test]
    fn shift_headings_with_h1_shift_all() {
        let text = "# h1\n## h2\n### h3";
        let (shifted, applied) = shift_headings_down_one_if_has_h1(text);
        assert!(applied);
        assert_eq!(shifted, "## h1\n### h2\n#### h3");
    }

    #[test]
    fn shift_headings_h5_to_h6() {
        let text = "# root\n##### h5";
        let (shifted, applied) = shift_headings_down_one_if_has_h1(text);
        assert!(applied);
        assert_eq!(shifted, "## root\n###### h5");
    }

    #[test]
    fn shift_headings_edge_cases() {
        let (a, b) = shift_headings_down_one_if_has_h1("");
        assert!(!b);
        assert_eq!(a, "");
        let (c, d) = shift_headings_down_one_if_has_h1("# only");
        assert!(d);
        assert_eq!(c, "## only");
    }

    #[test]
    fn write_diary_new_file_and_frontmatter() {
        let root = make_temp_dir("new_file");
        let payload = WriteDiaryPayload {
            date: "2026-03-27".to_string(),
            time: "09:30".to_string(),
            title: "晨间".to_string(),
            content: "今天状态不错".to_string(),
        };
        let path = write_diary_to_file(&root, &payload).expect("write");
        let raw = fs::read_to_string(path).expect("read");
        assert!(raw.contains("title: 2026年3月27日日记"));
        assert!(raw.contains("entryCount: 1"));
        assert!(raw.contains("# 09:30"));
    }

    #[test]
    fn append_without_sorting_when_later() {
        let root = make_temp_dir("append_later");
        let p1 = WriteDiaryPayload {
            date: "2026-03-27".to_string(),
            time: "09:30".to_string(),
            title: "".to_string(),
            content: "A".to_string(),
        };
        let p2 = WriteDiaryPayload {
            date: "2026-03-27".to_string(),
            time: "10:30".to_string(),
            title: "".to_string(),
            content: "B".to_string(),
        };
        let path = write_diary_to_file(&root, &p1).expect("first");
        write_diary_to_file(&root, &p2).expect("second");
        let raw = fs::read_to_string(path).expect("read");
        assert!(raw.find("# 09:30").unwrap() < raw.find("# 10:30").unwrap());
    }

    #[test]
    fn insert_middle_and_sort() {
        let root = make_temp_dir("insert_middle");
        let p1 = WriteDiaryPayload {
            date: "2026-03-27".to_string(),
            time: "09:30".to_string(),
            title: "".to_string(),
            content: "A".to_string(),
        };
        let p2 = WriteDiaryPayload {
            date: "2026-03-27".to_string(),
            time: "11:30".to_string(),
            title: "".to_string(),
            content: "B".to_string(),
        };
        let p3 = WriteDiaryPayload {
            date: "2026-03-27".to_string(),
            time: "10:00".to_string(),
            title: "".to_string(),
            content: "C".to_string(),
        };
        let path = write_diary_to_file(&root, &p1).expect("first");
        write_diary_to_file(&root, &p2).expect("second");
        write_diary_to_file(&root, &p3).expect("third");
        let raw = fs::read_to_string(path).expect("read");
        let i1 = raw.find("# 09:30").unwrap();
        let i2 = raw.find("# 10:00").unwrap();
        let i3 = raw.find("# 11:30").unwrap();
        assert!(i1 < i2 && i2 < i3);
    }

    #[test]
    fn preserve_prefix_content() {
        let body = "前置说明\n\n# 09:30\nA\n\n# 11:00\nB";
        let (prefix, entries) = parse_diary_body_entries(body);
        assert_eq!(prefix, "前置说明");
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn compare_sort_behavior_like_ts_logic() {
        let body = "# 11:00\nlate\n\n# 08:00\nearly";
        let (_prefix, mut entries) = parse_diary_body_entries(body);
        entries.sort_by_key(|e| e.time_minutes);
        assert!(entries[0].raw.contains("# 08:00"));
        assert!(entries[1].raw.contains("# 11:00"));
    }

    #[test]
    fn file_name_format_matches_requirement() {
        let date = NaiveDate::from_ymd_opt(2026, 3, 27).expect("valid date");
        let (title, file) = date_title_and_filename(&date);
        assert_eq!(title, "2026年3月27日日记");
        assert_eq!(file, "2026年3月27日日记.md");
        assert_eq!(date.year(), 2026);
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
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
