use eui::quick::ui::UI;
use eui::Rect;

use crate::app::AppState;
use crate::style::{FS_LABEL, LH_LABEL};

pub fn draw(ui: &mut UI, rect: Rect, app: &mut AppState) {
    ui.scope(rect, |ctx| {
        let mut ui = UI::new(ctx);
        let panel_color = ui.theme().panel;
        let r = ui.content_rect();
        ui.paint_filled_rect(r, panel_color, 0.0);
        let inner = eui::quick::ui::inset(&r, 8.0, 8.0);
        ui.scope(inner, |ctx| {
            let mut ui = UI::new(ctx);
            ui.label("Wave Inspector").font_size(FS_LABEL).height(LH_LABEL).draw();
            let _ = app;
        });
    });
}
