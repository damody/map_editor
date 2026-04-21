use crate::schema::CreepWaveData;
use std::path::PathBuf;

/// 用 rfd 檔案對話框開啟 map.json；失敗回傳 Err(msg)
pub fn pick_and_load() -> Result<(PathBuf, CreepWaveData), String> {
    let path = rfd::FileDialog::new()
        .add_filter("Map JSON", &["json"])
        .set_title("Open map.json")
        .pick_file()
        .ok_or_else(|| "User cancelled".to_string())?;
    let bytes = std::fs::read_to_string(&path).map_err(|e| format!("read: {}", e))?;
    // map.json 可能有 C-style 註解（// 和 /* */），移除後再 parse
    let cleaned = strip_json_comments(&bytes);
    let data: CreepWaveData =
        serde_json::from_str(&cleaned).map_err(|e| format!("parse: {}", e))?;
    Ok((path, data))
}

/// 用 rfd 對話框挑選另存新檔
pub fn pick_save_path() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("Map JSON", &["json"])
        .set_title("Save map.json")
        .save_file()
}

/// 存檔到已知路徑
pub fn save_to(path: &PathBuf, data: &CreepWaveData) -> Result<(), String> {
    let json = serde_json::to_string_pretty(data).map_err(|e| format!("serialize: {}", e))?;
    std::fs::write(path, json).map_err(|e| format!("write: {}", e))
}

pub fn strip_json_comments_public(src: &str) -> String {
    strip_json_comments(src)
}

/// 移除 C-style 註解（// 和 /* */），保留字串內容
fn strip_json_comments(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let chars: Vec<char> = src.chars().collect();
    let mut i = 0;
    let mut in_str = false;
    let mut escape = false;
    while i < chars.len() {
        let c = chars[i];
        if in_str {
            out.push(c);
            if escape {
                escape = false;
            } else if c == '\\' {
                escape = true;
            } else if c == '"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        if c == '"' {
            in_str = true;
            out.push(c);
            i += 1;
            continue;
        }
        // 單行 //
        if c == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }
        // 多行 /* */
        if c == '/' && i + 1 < chars.len() && chars[i + 1] == '*' {
            i += 2;
            while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                i += 1;
            }
            i += 2;
            continue;
        }
        out.push(c);
        i += 1;
    }
    out
}
