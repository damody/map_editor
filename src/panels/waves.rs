use eui::quick::ui::UI;
use eui::Rect;

use crate::app::AppState;

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
                .font_size(16.0)
                .height(20.0)
                .draw();
            ui.spacer(4.0);

            // 單行水平列出所有 wave
            let mut buf = String::new();
            for w in app.map.CreepWave.iter() {
                let total: usize = w.Detail.iter().map(|d| d.Creeps.len()).sum();
                buf.push_str(&format!("[{} t={:.0}s x{}]  ", w.Name, w.StartTime, total));
            }
            if buf.is_empty() {
                buf = "(無 wave 資料)".to_string();
            }
            ui.label(&buf).font_size(13.0).draw();
        });
    });
}
