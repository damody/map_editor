use eui::quick::ui::UI;
use eui::Rect;

use crate::app::AppState;
use crate::panels::{wave_inspector, wave_list, wave_timeline};
use crate::style::{FS_LABEL, FS_SUBHEAD, LH_LABEL, WAVE_LIST_W};

/// Map/Entities 模式底部的唯讀 wave 預覽條（單行）
pub fn draw(ui: &mut UI, rect: Rect, app: &mut AppState) {
    ui.scope(rect, |ctx| {
        let mut ui = UI::new(ctx);
        let panel_color = ui.theme().panel;
        let r = ui.content_rect();
        ui.paint_filled_rect(r, panel_color, 0.0);

        let inner = eui::quick::ui::inset(&r, 10.0, 8.0);
        ui.scope(inner, |ctx| {
            let mut ui = UI::new(ctx);
            ui.label("Creep Waves (read-only)")
                .font_size(FS_SUBHEAD)
                .height(LH_LABEL)
                .draw();
            ui.spacer(4.0);

            let mut buf = String::new();
            for w in app.map.CreepWave.iter() {
                let total: usize = w.Detail.iter().map(|d| d.Creeps.len()).sum();
                buf.push_str(&format!("[{} t={:.0}s x{}]  ", w.Name, w.StartTime, total));
            }
            if buf.is_empty() {
                buf = "(無 wave 資料)".to_string();
            }
            ui.label(&buf).font_size(FS_LABEL).draw();
        });
    });
}

/// Waves 模式三欄分派：左 wave 列表｜中 timeline｜右 inspector
pub fn draw_wave_mode(ui: &mut UI, rect: Rect, app: &mut AppState) {
    let list_w = WAVE_LIST_W;
    let inspector_w = app.inspector_w.max(crate::style::INSPECTOR_MIN_W);
    let timeline_w = (rect.w - list_w - inspector_w).max(100.0);

    let list_rect = Rect::new(rect.x, rect.y, list_w, rect.h);
    let timeline_rect = Rect::new(rect.x + list_w, rect.y, timeline_w, rect.h);
    let inspector_rect = Rect::new(rect.x + list_w + timeline_w, rect.y, inspector_w, rect.h);

    wave_list::draw(ui, list_rect, app);
    wave_timeline::draw(ui, timeline_rect, app);
    wave_inspector::draw(ui, inspector_rect, app);
}
