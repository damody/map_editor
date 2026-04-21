use crate::schema::CreepWaveData;

/// 選中物件
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Selection {
    None,
    Structure(usize),  // index into CreepWaveData.Structures
    CheckPoint(usize), // index into CreepWaveData.CheckPoint
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
}

impl Default for Tool {
    fn default() -> Self {
        Tool::Select
    }
}

/// 全域狀態
pub struct AppState {
    pub map: CreepWaveData,
    pub current_path: Option<std::path::PathBuf>, // 當前開啟的 map.json 路徑
    pub dirty: bool,                  // 未存檔標記
    pub selection: Selection,
    pub tool: Tool,
    /// Canvas 世界座標（中心點）→ 螢幕像素偏移
    pub pan: (f32, f32),
    /// 世界單位 → 像素（例如 0.1 = 1000 world 畫成 100px）
    pub zoom: f32,
    /// 拖拉中的物件初始世界座標（供 restore）
    pub drag_state: Option<DragState>,
    /// 新增塔用的模板名稱（預設為 Tower 陣列第 0 筆）
    pub new_tower_template: String,
    pub new_tower_faction: String, // "Player" or "Enemy"
    pub new_tower_is_base: bool,
    /// Pan 用：記上一 frame 滑鼠座標（中鍵按住時計算 delta）
    pub prev_mouse_screen: Option<(f32, f32)>,
    /// 雙擊偵測用：上次左鍵按下時間、位置
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
            selection: Selection::None,
            tool: Tool::Select,
            pan: (0.0, 0.0),
            zoom: 0.25, // 預設 1 world = 0.25 px（2400 world = 600px）
            drag_state: None,
            new_tower_template: String::new(),
            new_tower_faction: "Player".to_string(),
            new_tower_is_base: false,
            prev_mouse_screen: None,
            last_click_time: None,
            last_click_pos: None,
        }
    }
}
