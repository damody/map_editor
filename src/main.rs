mod app;
mod canvas;
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

    // 若啟動時帶 CLI 參數 → 自動開啟該檔案
    if let Some(path) = std::env::args().nth(1) {
        let p = std::path::PathBuf::from(&path);
        let bytes = std::fs::read_to_string(&p).expect("cannot read map file");
        let cleaned = io::strip_json_comments_public(&bytes);
        match serde_json::from_str::<schema::CreepWaveData>(&cleaned) {
            Ok(data) => {
                state.map = data;
                state.current_path = Some(p);
                if let Some(t) = state.map.Tower.first() {
                    state.new_tower_template = t.Name.clone();
                }
            }
            Err(e) => eprintln!("Failed to parse {}: {}", path, e),
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
