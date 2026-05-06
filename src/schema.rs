//! `omb/src/ue4/import_map.rs` 的 generated map 對應表。
//! 欄位命名維持 PascalCase 以符合既有 map.lua table shape。

# ![允許(non_snake_case)]

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct CreepWaveData {
    pub Path: Vec<PathJD>,
    pub Creep: Vec<CreepJD>,
    pub CheckPoint: Vec<CheckPointJD>,
    pub Tower: Vec<TowerJD>,
    pub CreepWave: Vec<CreepWaveJD>,
    # [serde（預設）]
    pub Structures: Vec<StructureJD>,
    # [serde（預設）]
    pub BlockedRegions: Vec<BlockedRegionJD>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct StructureJD {
    pub Tower: String,
    pub Faction: String,
    pub X: f32,
    pub Y: f32,
    # [serde（預設）]
    pub IsBase: bool,
    # [serde（預設）]
    pub CollisionRadius: Option<f32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct PathJD {
    pub Name: String,
    pub Points: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct CreepJD {
    pub Name: String,
    /// 僅出現在 legacy JSON 的欄位。generated map data 不包含地圖本地標籤與統計值。
    # [serde（預設）]
    pub Label: Option<String>,
    # [serde（預設）]
    pub HP: f32,
    # [serde（預設）]
    pub DefendPhysic: f32,
    # [serde（預設）]
    pub DefendMagic: f32,
    # [serde（預設）]
    pub MoveSpeed: f32,
    # [serde（預設）]
    pub Faction: Option<String>,
    # [serde（預設）]
    pub TurnSpeed: Option<f32>,
    # [serde（預設）]
    pub CollisionRadius: Option<f32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct CheckPointJD {
    pub Name: String,
    pub Class: String,
    pub X: f32,
    pub Y: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct TowerJD {
    pub Name: String,
    pub Property: PropertyJD,
    pub Attack: AttackJD,
    # [serde（預設）]
    pub TurnSpeed: Option<f32>,
    # [serde（預設）]
    pub CollisionRadius: Option<f32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AttackJD {
    pub Range: f32,
    pub AttackSpeed: f32,
    pub Physic: f32,
    pub Magic: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct PropertyJD {
    pub Hp: i32,
    pub Block: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct CreepWaveJD {
    pub Name: String,
    pub StartTime: f32,
    pub Detail: Vec<DetailJD>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct DetailJD {
    pub Path: String,
    pub Creeps: Vec<CreepsJD>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct CreepsJD {
    pub Time: f32,
    pub Creep: String,
}

/// 不可通行多邊形區域（凹/凸皆可）。至少 3 點。
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct BlockedRegionJD {
    pub Name: String,
    pub Points: Vec<PointJD>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default)]
pub struct PointJD {
    pub X: f32,
    pub Y: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_mvp1_generated_map() {
        let story = omoba_template_ids::story_by_name("MVP_1").expect("generated MVP_1 story");
        let data: CreepWaveData = serde_json::from_value(story_value_to_json(story.map))
            .expect("parse generated CreepWaveData");
        assert!(!data.Path.is_empty());
        assert!(!data.CheckPoint.is_empty());
        assert!(!data.Structures.is_empty());
        println!("paths={} cps={} towers={} structures={} waves={} regions={}",
            data.Path.len(), data.CheckPoint.len(), data.Tower.len(),
            data.Structures.len(), data.CreepWave.len(), data.BlockedRegions.len());
        // 轉換來回（round-trip）檢查
        let back = serde_json::to_string_pretty(&data).expect("serialize");
        let data2: CreepWaveData = serde_json::from_str(&back).expect("reparse");
        assert_eq!(data.Structures.len(), data2.Structures.len());
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

    fn json_number(value: f64) -> serde_json::Value {
        if value.fract() == 0.0 && value >= i64::MIN as f64 && value <= i64::MAX as f64 {
            serde_json::Value::Number(serde_json::Number::from(value as i64))
        } else {
            serde_json::Number::from_f64(value)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null)
        }
    }

    #[test]
    fn blocked_regions_round_trip() {
        let mut d = CreepWaveData::default();
        d.BlockedRegions.push(BlockedRegionJD {
            Name: "lake".into(),
            Points: vec![
                PointJD { X: 0.0, Y: 0.0 },
                PointJD { X: 100.0, Y: 0.0 },
                PointJD { X: 50.0, Y: 80.0 },
            ],
        });
        let s = serde_json::to_string(&d).unwrap();
        let d2: CreepWaveData = serde_json::from_str(&s).unwrap();
        assert_eq!(d2.BlockedRegions.len(), 1);
        assert_eq!(d2.BlockedRegions[0].Name, "lake");
        assert_eq!(d2.BlockedRegions[0].Points.len(), 3);
    }
}

