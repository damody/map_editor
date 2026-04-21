//! Mirror of `omb/src/ue4/import_map.rs`. Serde-compatible with map.json.
//! 欄位命名維持 PascalCase 以符合既有 map.json 格式。

#![allow(non_snake_case)]

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct CreepWaveData {
    pub Path: Vec<PathJD>,
    pub Creep: Vec<CreepJD>,
    pub CheckPoint: Vec<CheckPointJD>,
    pub Tower: Vec<TowerJD>,
    pub CreepWave: Vec<CreepWaveJD>,
    #[serde(default)]
    pub Structures: Vec<StructureJD>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct StructureJD {
    pub Tower: String,
    pub Faction: String,
    pub X: f32,
    pub Y: f32,
    #[serde(default)]
    pub IsBase: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct PathJD {
    pub Name: String,
    pub Points: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct CreepJD {
    pub Name: String,
    #[serde(default)]
    pub Label: Option<String>,
    pub HP: f32,
    pub DefendPhysic: f32,
    pub DefendMagic: f32,
    pub MoveSpeed: f32,
    #[serde(default)]
    pub Faction: Option<String>,
    #[serde(default)]
    pub TurnSpeed: Option<f32>,
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
    #[serde(default)]
    pub TurnSpeed: Option<f32>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::strip_json_comments_public;

    #[test]
    fn parse_mvp1_map_json() {
        let raw = std::fs::read_to_string("../omb/Story/MVP_1/map.json")
            .expect("read map.json");
        let cleaned = strip_json_comments_public(&raw);
        let data: CreepWaveData = serde_json::from_str(&cleaned)
            .expect("parse CreepWaveData");
        assert!(!data.Path.is_empty());
        assert!(!data.CheckPoint.is_empty());
        assert!(!data.Structures.is_empty());
        println!("paths={} cps={} towers={} structures={} waves={}",
            data.Path.len(), data.CheckPoint.len(), data.Tower.len(),
            data.Structures.len(), data.CreepWave.len());
        // round-trip
        let back = serde_json::to_string_pretty(&data).expect("serialize");
        let data2: CreepWaveData = serde_json::from_str(&back).expect("reparse");
        assert_eq!(data.Structures.len(), data2.Structures.len());
    }
}
