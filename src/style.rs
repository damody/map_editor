//! 編輯器 UI 尺寸常數。所有字級與列高都集中在此，方便全域縮放。
//!
//! 設計原則：
//! - 原始基準字級 (BASE_*) 代表「理論上最自然的點數」
//! - `UI_SCALE` 控制全域放大倍率，這裡設 1.5 讓中文看起來不擁擠
//! - 對外匯出 `FS_*` / `LH_*` 是 scale 後的實際像素
//! - 新增字級時請在這裡集中管理，禁止在 panels / canvas 內出現裸 float

/// 全域字級縮放倍率（原 1.0 視為 base；目前 1.5×）
pub const UI_SCALE: f32 = 1.5;

// ---- Base font sizes (設計尺寸，未縮放) ----
const BASE_CAPTION: f32 = 12.0; // 註記/提示小字
const BASE_LABEL: f32 = 13.0;   // 一般 label
const BASE_BODY_SM: f32 = 14.0; // 次要內文
const BASE_BODY: f32 = 15.0;    // 主要內文（toolbar 狀態列）
const BASE_SUBHEAD: f32 = 16.0; // inspector 小標題
const BASE_HEAD: f32 = 18.0;    // 面板主標題

// ---- Scaled font sizes (實際使用) ----
pub const FS_CAPTION: f32 = BASE_CAPTION * UI_SCALE;
pub const FS_LABEL: f32 = BASE_LABEL * UI_SCALE;
pub const FS_BODY_SM: f32 = BASE_BODY_SM * UI_SCALE;
pub const FS_BODY: f32 = BASE_BODY * UI_SCALE;
pub const FS_SUBHEAD: f32 = BASE_SUBHEAD * UI_SCALE;
pub const FS_HEAD: f32 = BASE_HEAD * UI_SCALE;

// ---- Label row heights (with matching padding) ----
const BASE_LH_LABEL: f32 = 20.0; // 一般 label 行高（給 wave 數字列用）
const BASE_LH_HEAD: f32 = 28.0;  // 面板主標題行高

pub const LH_LABEL: f32 = BASE_LH_LABEL * UI_SCALE;
pub const LH_HEAD: f32 = BASE_LH_HEAD * UI_SCALE;

// ---- Top-level panel dimensions (main.rs layout) ----
const BASE_TOOLBAR_H: f32 = 44.0;
const BASE_WAVES_H: f32 = 60.0;
const BASE_LEFT_W: f32 = 240.0;
const BASE_RIGHT_W: f32 = 280.0;

pub const TOOLBAR_H: f32 = BASE_TOOLBAR_H * UI_SCALE;
pub const WAVES_H: f32 = BASE_WAVES_H * UI_SCALE;
pub const LEFT_W: f32 = BASE_LEFT_W * UI_SCALE;
pub const RIGHT_W: f32 = BASE_RIGHT_W * UI_SCALE;

// ---- Inspector input/slider sizes ----
/// 第二次使用者回饋放大倍率（套用在 inspector 欄位）
const INSPECTOR_BUMP: f32 = 1.2;

/// eui InputBuilder 內部的 label 字級（預設 11.0 × scale × bump）
pub const FS_FIELD_LABEL: f32 = 11.0 * UI_SCALE * INSPECTOR_BUMP;
/// eui InputBuilder 內部的 label 行高（預設 16.0 × scale × bump）
pub const LH_FIELD_LABEL: f32 = 16.0 * UI_SCALE * INSPECTOR_BUMP;
/// input / slider 欄位的整體行高（原 36.0 × scale × bump）
pub const H_FIELD: f32 = 36.0 * UI_SCALE * INSPECTOR_BUMP;
/// input 欄位內值的字級。text_input_impl 自動尺寸約為 12.5px；
/// 先 ×1.3 再 × bump ≈ 19.5px
pub const FS_FIELD_VALUE: f32 = 12.5 * 1.3 * INSPECTOR_BUMP;

/// slider 值數字的字級。先 ×1.5 再 × bump ≈ 23.4px
pub const FS_SLIDER_VALUE: f32 = 13.0 * 1.5 * INSPECTOR_BUMP;
/// slider bar 的最大高度；加高以配合大字
pub const H_SLIDER_BAR_MAX: f32 = 56.0 * INSPECTOR_BUMP;

// ---- Inspector 寬度可調 ----
/// Inspector 寬度最小值（避免被拉到完全看不見）
pub const INSPECTOR_MIN_W: f32 = 200.0;
/// 分隔條寬度（px）— 滑鼠命中用
pub const SPLITTER_W: f32 = 6.0;

// ---- Toolbar cell width ----
/// Toolbar 內每顆按鈕分配的寬度；字級放大後需同步放大以免截斷
const BASE_TOOLBAR_CELL_W: f32 = 100.0;
pub const TOOLBAR_CELL_W: f32 = BASE_TOOLBAR_CELL_W * UI_SCALE;
/// 按鈕之間的水平間距
pub const TOOLBAR_CELL_GAP: f32 = 4.0;
/// 按鈕群組之間的分隔距離
pub const TOOLBAR_GROUP_GAP: f32 = 16.0;
