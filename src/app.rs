use crate::entity_schema::EntityConfig;
use crate::schema::{CreepWaveData, PointJD};

/// 選中物件
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Selection {
    None,
    // map.json
    Structure(usize),
    CheckPoint(usize),
    BlockedRegion(usize),
    /// (region_idx, point_idx)
    BlockedRegionPoint(usize, usize),
    TowerTemplate(usize),
    CreepTemplate(usize),
    // entity.json
    Hero(usize),
    Enemy(usize),
}

impl Default for Selection {
    fn default() -> Self {
        Selection::None
    }
}

/// 編輯器工具模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Select,
    AddTower,
    AddCheckPoint,
    AddBlockedRegion,
    EditBlockedRegion,
}

impl Default for Tool {
    fn default() -> Self {
        Tool::Select
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Map,
    Entities,
}

impl Default for ViewMode {
    fn default() -> Self {
        ViewMode::Map
    }
}

/// 全域狀態
pub struct AppState {
    pub map: CreepWaveData,
    pub current_path: Option<std::path::PathBuf>,
    pub dirty: bool,

    pub entity: EntityConfig,
    pub entity_path: Option<std::path::PathBuf>,
    pub entity_dirty: bool,

    pub view_mode: ViewMode,
    pub selection: Selection,
    pub tool: Tool,
    pub pan: (f32, f32),
    pub zoom: f32,
    pub drag_state: Option<DragState>,

    pub new_tower_template: String,
    pub new_tower_faction: String,
    pub new_tower_is_base: bool,

    /// 繪製中的多邊形草稿（AddBlockedRegion 工具使用）
    pub region_draft: Vec<PointJD>,

    pub prev_mouse_screen: Option<(f32, f32)>,
    pub last_click_time: Option<std::time::Instant>,
    pub last_click_pos: Option<(f32, f32)>,
}

#[derive(Debug, Clone, Copy)]
pub struct DragState {
    pub sel: Selection,
    pub orig_world_x: f32,
    pub orig_world_y: f32,
    pub start_mouse_x: f32,
    pub start_mouse_y: f32,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            map: CreepWaveData::default(),
            current_path: None,
            dirty: false,
            entity: EntityConfig::default(),
            entity_path: None,
            entity_dirty: false,
            view_mode: ViewMode::Map,
            selection: Selection::None,
            tool: Tool::Select,
            pan: (0.0, 0.0),
            zoom: 0.25,
            drag_state: None,
            new_tower_template: String::new(),
            new_tower_faction: "Player".to_string(),
            new_tower_is_base: false,
            region_draft: Vec::new(),
            prev_mouse_screen: None,
            last_click_time: None,
            last_click_pos: None,
        }
    }
}
