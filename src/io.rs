use crate::entity_schema::EntityConfig;
use crate::schema::CreepWaveData;
use std::path::PathBuf;

/// 用 rfd 檔案對話框開啟 legacy map JSON；若是 shipped story，改走 generated data directory mode。
pub fn pick_and_load() -> Result<(PathBuf, CreepWaveData), String> {
    let path = rfd::FileDialog::new()
        .add_filter("Map JSON", &["json"])
        .set_title("Open legacy map JSON")
        .pick_file()
        .ok_or_else(|| "User cancelled".to_string())?;
    let bytes = std::fs::read_to_string(&path).map_err(|e| format!("read: {}", e))?;
        // legacy JSON 可能有 C-style 註解（// 和 /* */），移除後再解析
    let cleaned = strip_json_comments(&bytes);
    let data: CreepWaveData =
        serde_json::from_str(&cleaned).map_err(|e| format!("parse: {}", e))?;
    Ok((path, data))
}

/// 用 rfd 對話框挑選另存新檔
pub fn pick_save_path() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("Map Lua", &["lua"])
        .set_title("Save map.lua")
        .save_file()
}

/// 存檔到已知路徑
pub fn save_to(path: &PathBuf, data: &CreepWaveData) -> Result<(), String> {
    let value = serde_json::to_value(data).map_err(|e| format!("serialize: {}", e))?;
    let lua = format!("return function(ctx)\n  return {}\nend\n", lua_value(&value, 1));
    std::fs::write(path, lua).map_err(|e| format!("write: {}", e))
}

pub fn strip_json_comments_public(src: &str) -> String {
    strip_json_comments(src)
}

/// 開啟 entity.json 檔案對話框並載入。
pub fn pick_and_load_entity() -> Result<(PathBuf, EntityConfig), String> {
    let path = rfd::FileDialog::new()
        .add_filter("Entity JSON", &["json"])
        .set_title("Open entity.json")
        .pick_file()
        .ok_or_else(|| "User cancelled".to_string())?;
    let bytes = std::fs::read_to_string(&path).map_err(|e| format!("read: {}", e))?;
    let cleaned = strip_json_comments(&bytes);
    let data: EntityConfig =
        serde_json::from_str(&cleaned).map_err(|e| format!("parse: {}", e))?;
    Ok((path, data))
}

/// 嘗試依 map.json 路徑旁邊自動載入同目錄下的 entity.json（沒有就傳 None）。
pub fn try_load_sibling_entity(map_path: &PathBuf) -> Option<(PathBuf, EntityConfig)> {
    let p = map_path.parent()?.join("entity.json");
    if !p.exists() {
        return None;
    }
    let bytes = std::fs::read_to_string(&p).ok()?;
    let cleaned = strip_json_comments(&bytes);
    let data: EntityConfig = serde_json::from_str(&cleaned).ok()?;
    Some((p, data))
}

pub fn save_entity_to(path: &PathBuf, data: &EntityConfig) -> Result<(), String> {
    let value = serde_json::to_value(data).map_err(|e| format!("serialize: {}", e))?;
    let lua = format!("return function(ctx)\n  return {}\nend\n", lua_value(&value, 1));
    std::fs::write(path, lua).map_err(|e| format!("write: {}", e))
}

pub fn pick_save_entity_path() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("Entity Lua", &["lua"])
        .set_title("Save entity.lua")
        .save_file()
}

/// 嘗試載入同目錄下的 ability.json（Value 原樣保留）
pub fn try_load_sibling_ability(map_path: &PathBuf) -> Option<(PathBuf, serde_json::Value)> {
    let p = map_path.parent()?.join("ability.json");
    if !p.exists() { return None; }
    let bytes = std::fs::read_to_string(&p).ok()?;
    let cleaned = strip_json_comments(&bytes);
    let v: serde_json::Value = serde_json::from_str(&cleaned).ok()?;
    Some((p, v))
}

/// 嘗試載入同目錄下的 mission.json（Value 原樣保留）
pub fn try_load_sibling_mission(map_path: &PathBuf) -> Option<(PathBuf, serde_json::Value)> {
    let p = map_path.parent()?.join("mission.json");
    if !p.exists() { return None; }
    let bytes = std::fs::read_to_string(&p).ok()?;
    let cleaned = strip_json_comments(&bytes);
    let v: serde_json::Value = serde_json::from_str(&cleaned).ok()?;
    Some((p, v))
}

pub fn save_ability_to(path: &PathBuf, data: &serde_json::Value) -> Result<(), String> {
    let lua = format!("return function(ctx)\n  return {}\nend\n", lua_value(data, 1));
    std::fs::write(path, lua).map_err(|e| format!("write: {}", e))
}

pub fn save_mission_to(path: &PathBuf, data: &serde_json::Value) -> Result<(), String> {
    let lua = format!("return function(ctx)\n  return {}\nend\n", lua_value(data, 1));
    std::fs::write(path, lua).map_err(|e| format!("write: {}", e))
}

/// 載入一個目錄。若目錄名稱是 shipped story id，優先使用 generated Rust data；
/// 否則回退到 legacy JSON 匯入流程。
pub fn load_campaign_dir(dir: &std::path::Path) -> (
    Option<(PathBuf, CreepWaveData)>,
    Option<(PathBuf, EntityConfig)>,
    Option<(PathBuf, serde_json::Value)>,
    Option<(PathBuf, serde_json::Value)>,
) {
    if let Some(story_id) = dir.file_name().and_then(|name| name.to_str()) {
        if let Some(story) = omoba_template_ids::story_by_name(story_id) {
            let mut map_value = story_value_to_json(story.map);
            normalize_map_value(&mut map_value);
            let map = serde_json::from_value::<CreepWaveData>(map_value).ok();
            return (
                map.map(|data| (dir.join("map.lua"), data)),
                None,
                Some((dir.join("ability.lua"), story_value_to_json(story.ability))),
                Some((dir.join("mission.lua"), story_value_to_json(story.mission))),
            );
        }
    }

    fn read_json<T: for<'de> serde::Deserialize<'de>>(p: &PathBuf) -> Option<T> {
        if !p.exists() { return None; }
        let bytes = std::fs::read_to_string(p).ok()?;
        let cleaned = strip_json_comments(&bytes);
        serde_json::from_str(&cleaned).ok()
    }
    let mp = dir.join("map.json");
    let ep = dir.join("entity.json");
    let ap = dir.join("ability.json");
    let misp = dir.join("mission.json");
    (
        read_json::<CreepWaveData>(&mp).map(|d| (mp, d)),
        read_json::<EntityConfig>(&ep).map(|d| (ep, d)),
        read_json::<serde_json::Value>(&ap).map(|d| (ap, d)),
        read_json::<serde_json::Value>(&misp).map(|d| (misp, d)),
    )
}

fn story_value_to_json(value: omoba_template_ids::StoryValue) -> serde_json::Value {
    match value {
        omoba_template_ids::StoryValue::Null => serde_json::Value::Null,
        omoba_template_ids::StoryValue::Bool(value) => serde_json::Value::Bool(value),
        omoba_template_ids::StoryValue::Number(value) => json_number(value),
        omoba_template_ids::StoryValue::String(value) => serde_json::Value::String(value.to_string()),
        omoba_template_ids::StoryValue::Array(values) => {
            serde_json::Value::Array(values.iter().copied().map(story_value_to_json).collect())
        }
        omoba_template_ids::StoryValue::Object(values) => {
            let mut map = serde_json::Map::new();
            for (key, value) in values.iter().copied() {
                map.insert(key.to_string(), story_value_to_json(value));
            }
            serde_json::Value::Object(map)
        }
    }
}

fn normalize_map_value(value: &mut serde_json::Value) {
    for key in ["Path", "Creep", "CheckPoint", "Tower", "CreepWave", "Structures", "BlockedRegions"] {
        ensure_array_field(value, key);
    }
    if let Some(waves) = value.get_mut("CreepWave").and_then(serde_json::Value::as_array_mut) {
        for wave in waves {
            ensure_array_field(wave, "Detail");
            if let Some(details) = wave.get_mut("Detail").and_then(serde_json::Value::as_array_mut) {
                for detail in details {
                    ensure_array_field(detail, "Creeps");
                }
            }
        }
    }
}

fn ensure_array_field(value: &mut serde_json::Value, key: &str) {
    let Some(object) = value.as_object_mut() else { return; };
    match object.get_mut(key) {
        Some(field) if field.as_object().is_some_and(serde_json::Map::is_empty) => {
            * 字段 = serde_json::Value::Array(Vec::new());
        }
        None => {
            object.insert(key.to_string(), serde_json::Value::Array(Vec::new()));
        }
        _ => {}
    }
}

fn json_number(value: f64) -> serde_json::Value {
    if value.fract() == 0.0 && value >= i64::MIN as f64 && value <= i64::MAX as f64 {
        serde_json::Value::Number(serde_json::Number::from(value as i64))
    } else {
        serde_json::Number::from_f64(value)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null)
    }
}

fn lua_value(value: &serde_json::Value, indent: usize) -> String {
    let space = "  ".repeat(indent);
    let child = "  ".repeat(indent + 1);
    match value {
        serde_json::Value::Null => "nil".to_string(),
        serde_json::Value::Bool(value) => value.to_string(),
        serde_json::Value::Number(value) => value.to_string(),
        serde_json::Value::String(value) => lua_string(value),
        serde_json::Value::Array(items) => {
            if items.is_empty() { return "{}".to_string(); }
            let mut out = String::from("{\n");
            for item in items {
                out.push_str(&child);
                out.push_str(&lua_value(item, indent + 1));
                out.push_str(",\n");
            }
            out.push_str(&space);
            out.push('}');
            out
        }
        serde_json::Value::Object(items) => {
            if items.is_empty() { return "{}".to_string(); }
            let mut out = String::from("{\n");
            for (key, item) in items {
                if item.is_null() { continue; }
                out.push_str(&child);
                out.push_str(&lua_key(key));
                out.push_str(" = ");
                out.push_str(&lua_value(item, indent + 1));
                out.push_str(",\n");
            }
            out.push_str(&space);
            out.push('}');
            out
        }
    }
}

fn lua_key(key: &str) -> String {
    let mut chars = key.chars();
    let valid = chars
        .next()
        .is_some_and(|c| c == '_' || c.is_ascii_alphabetic())
        && chars.all(|c| c == '_' || c.is_ascii_alphanumeric());
    if valid { key.to_string() } else { format!("[{}]", lua_string(key)) }
}

fn lua_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
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
        // 多行註解區塊（/* */）
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
