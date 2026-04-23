use crate::entity_schema::EntityConfig;
use crate::schema::{CreepWaveData, PointJD};
use crate::undo::{Snapshot, UndoStack};

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
    /// 選中整波（Inspector 顯示 Name / StartTime / + Detail）
    Wave(usize),
    /// (wave_idx, detail_idx) — 選中 wave 內某條 lane
    WaveDetail(usize, usize),
    /// (wave, detail, spawn) — 選中 timeline 上的某顆 spawn 圓
    WaveSpawn(usize, usize, usize),
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
    Waves,
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

    /// ability.json 原樣保留為 Value（目前僅支援另存往返，不做 typed editing）
    pub ability: serde_json::Value,
    pub ability_path: Option<std::path::PathBuf>,
    pub ability_dirty: bool,

    /// mission.json 原樣保留為 Value
    pub mission: serde_json::Value,
    pub mission_path: Option<std::path::PathBuf>,
    pub mission_dirty: bool,

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

    /// AddCheckPoint 模式中正在連續繪製的路徑索引；切換工具時重置
    pub current_path_idx: Option<usize>,

    /// 右側 Inspector 面板寬度（px），可被分隔條拖拉調整
    pub inspector_w: f32,
    /// Inspector 分隔條拖拉中的起始寬度（拖拉起始時記 None → Some）
    pub inspector_resize_start: Option<(f32, f32)>, // (start_mouse_x, start_width)

    /// Undo/Redo 堆疊
    pub undo: UndoStack,
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
            ability: serde_json::Value::Null,
            ability_path: None,
            ability_dirty: false,
            mission: serde_json::Value::Null,
            mission_path: None,
            mission_dirty: false,
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
            current_path_idx: None,
            inspector_w: crate::style::RIGHT_W,
            inspector_resize_start: None,
            undo: UndoStack::new(),
        }
    }
}

impl AppState {
    /// 在每次修改 map/entity/selection 之前呼叫，將當前狀態壓入 undo 堆疊。
    /// tag=Some(...)：同 tag 在同一 group 內只 push 一次（slider / drag 合併）。
    /// tag=None：一次性操作（新增 / 刪除等），每次都 push。
    pub fn begin_edit(&mut self, tag: Option<&str>) {
        let snap = Snapshot {
            map: self.map.clone(),
            entity: self.entity.clone(),
            selection: self.selection,
        };
        self.undo.push(snap, tag);
    }

    pub fn current_snapshot(&self) -> Snapshot {
        Snapshot {
            map: self.map.clone(),
            entity: self.entity.clone(),
            selection: self.selection,
        }
    }

    pub fn apply_snapshot(&mut self, snap: Snapshot) {
        self.map = snap.map;
        self.entity = snap.entity;
        self.selection = snap.selection;
        self.dirty = true;
        self.entity_dirty = true;
        self.drag_state = None;
        self.region_draft.clear();
    }
}
