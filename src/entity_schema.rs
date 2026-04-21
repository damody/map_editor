//! Mirror of `omb/src/ue4/import_campaign.rs` 裡的 HeroJD / EnemyJD
//! （entity.json 的 snake_case 格式）。
//!
//! 本模組只包含 entity.json 必要的子集（heroes + enemies），其他欄位
//! （abilities / mission / creeps / neutrals / summons 等）透過 serde
//! 的 flatten `extra` 欄位原樣保留，確保存檔不會遺失資料。

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct EntityConfig {
    #[serde(default)]
    pub heroes: Vec<HeroJD>,
    #[serde(default)]
    pub enemies: Vec<EnemyJD>,
    /// 原樣保留 entity.json 其他欄位（creeps / neutrals / summons 等），
    /// 避免存檔時被截掉。
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct HeroJD {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub background: String,

    pub strength: i32,
    pub agility: i32,
    pub intelligence: i32,
    #[serde(default)]
    pub primary_attribute: String,

    pub attack_range: f32,
    pub base_damage: i32,
    pub base_armor: f32,
    pub base_hp: i32,
    pub base_mana: i32,
    pub move_speed: f32,
    #[serde(default)]
    pub turn_speed: Option<f32>,
    #[serde(default)]
    pub collision_radius: Option<f32>,

    #[serde(default)]
    pub abilities: Vec<String>,
    #[serde(default)]
    pub level_growth: LevelGrowthJD,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct LevelGrowthJD {
    pub strength_per_level: f32,
    pub agility_per_level: f32,
    pub intelligence_per_level: f32,
    pub damage_per_level: f32,
    pub hp_per_level: f32,
    pub mana_per_level: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct EnemyJD {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub enemy_type: String,

    pub hp: i32,
    pub armor: f32,
    pub magic_resistance: f32,
    pub damage: i32,
    pub attack_range: f32,
    pub move_speed: f32,

    #[serde(default)]
    pub ai_type: String,
    #[serde(default)]
    pub abilities: Vec<String>,

    #[serde(default)]
    pub exp_reward: i32,
    #[serde(default)]
    pub gold_reward: i32,

    #[serde(default)]
    pub collision_radius: Option<f32>,
}
