mod app;
mod canvas;
mod entity_schema;
mod geometry;
mod io;
mod panels;
mod schema;

use eui::{AppOptions, Rect};

use app::AppState;

fn main() {
    let mut state = AppState::default();

    // 使用支援 CJK 的字型（Windows 系統內建的微軟正黑體），確保中文可顯示。
    // 若不存在就回退到系統預設（可能無法顯示中文）。
    let cjk_candidates = [
        "C:/Windows/Fonts/msjh.ttc",
        "C:/Windows/Fonts/msjh.ttf",
        "C:/Windows/Fonts/msyh.ttc",
        "C:/Windows/Fonts/simhei.ttf",
    ];
    let cjk_font = cjk_candidates
        .iter()
        .find(|p| std::path::Path::new(p).exists())
        .map(|s| s.to_string());

    let mut opts = AppOptions::default();
    opts.title = "Map Editor".to_string();
    opts.width = 1600;
    opts.height = 960;
    opts.text_font_file = cjk_font;

    // 若啟動時帶 CLI 參數 → 判斷是目錄還是單檔
    if let Some(path_arg) = std::env::args().nth(1) {
        let p = std::path::PathBuf::from(&path_arg);
        if p.is_dir() {
            // 目錄模式：一次載入 4 個 JSON
            let (mp, ep, ap, misp) = io::load_campaign_dir(&p);
            if let Some((path, data)) = mp {
                if let Some(t) = data.Tower.first() {
                    state.new_tower_template = t.Name.clone();
                }
                state.map = data;
                state.current_path = Some(path);
            } else {
                eprintln!("Directory has no map.json: {}", p.display());
            }
            if let Some((path, data)) = ep {
                state.entity = data;
                state.entity_path = Some(path);
            }
            if let Some((path, data)) = ap {
                state.ability = data;
                state.ability_path = Some(path);
            }
            if let Some((path, data)) = misp {
                state.mission = data;
                state.mission_path = Some(path);
            }
        } else {
            // 單檔模式（向後相容）：載入 map.json + 同目錄 sibling entity
            let bytes = std::fs::read_to_string(&p).expect("cannot read map file");
            let cleaned = io::strip_json_comments_public(&bytes);
            match serde_json::from_str::<schema::CreepWaveData>(&cleaned) {
                Ok(data) => {
                    state.map = data;
                    state.current_path = Some(p.clone());
                    if let Some(t) = state.map.Tower.first() {
                        state.new_tower_template = t.Name.clone();
                    }
                    if let Some((ep, ed)) = io::try_load_sibling_entity(&p) {
                        state.entity = ed;
                        state.entity_path = Some(ep);
                    }
                    if let Some((ap, ad)) = io::try_load_sibling_ability(&p) {
                        state.ability = ad;
                        state.ability_path = Some(ap);
                    }
                    if let Some((misp, misd)) = io::try_load_sibling_mission(&p) {
                        state.mission = misd;
                        state.mission_path = Some(misp);
                    }
                }
                Err(e) => eprintln!("Failed to parse {}: {}", path_arg, e),
            }
        }
    }

    eui::run_with_options(move |_ctx, ui| {
        let content = ui.content_rect();
        let bg = ui.theme().background;
        ui.paint_filled_rect(content, bg, 0.0);

        // 佈局：toolbar (頂), waves (底), 中間 left templates | canvas | right inspector
        let toolbar_h = 44.0;
        let waves_h = 60.0;
        let left_w = 240.0;
        let right_w = 280.0;

        let toolbar_rect = Rect::new(content.x, content.y, content.w, toolbar_h);
        let waves_rect = Rect::new(
            content.x,
            content.y + content.h - waves_h,
            content.w,
            waves_h,
        );
        let middle_rect = Rect::new(
            content.x,
            content.y + toolbar_h,
            content.w,
            (content.h - toolbar_h - waves_h).max(0.0),
        );
        let templates_rect = Rect::new(middle_rect.x, middle_rect.y, left_w, middle_rect.h);
        let inspector_rect = Rect::new(
            middle_rect.x + middle_rect.w - right_w,
            middle_rect.y,
            right_w,
            middle_rect.h,
        );
        let canvas_rect = Rect::new(
            middle_rect.x + left_w,
            middle_rect.y,
            middle_rect.w - left_w - right_w,
            middle_rect.h,
        );

        panels::toolbar::draw(ui, toolbar_rect, &mut state);
        panels::templates::draw(ui, templates_rect, &mut state);
        panels::inspector::draw(ui, inspector_rect, &mut state);
        panels::waves::draw(ui, waves_rect, &mut state);
        canvas::draw(ui, canvas_rect, &mut state);
    }, opts);
}
