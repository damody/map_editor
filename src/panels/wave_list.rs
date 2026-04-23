use eui::quick::ui::UI;
use eui::{ButtonStyle, Rect};

use crate::app::{AppState, Selection};
use crate::style::{FS_LABEL, FS_SUBHEAD, LH_LABEL};

pub fn draw(ui: &mut UI, rect: Rect, app: &mut AppState) {
    ui.scope(rect, |ctx| {
        let mut ui = UI::new(ctx);
        let panel_color = ui.theme().panel;
        let r = ui.content_rect();
        ui.paint_filled_rect(r, panel_color, 0.0);
        let inner = eui::quick::ui::inset(&r, 8.0, 8.0);
        ui.scope(inner, |ctx| {
            let mut ui = UI::new(ctx);
            ui.label("Waves").font_size(FS_SUBHEAD).height(LH_LABEL).draw();
            ui.spacer(4.0);

            let waves = app.map.CreepWave.clone();
            for (i, w) in waves.iter().enumerate() {
                let total: usize = w.Detail.iter().map(|d| d.Creeps.len()).sum();
                let is_selected = app.wave_edit.selected_wave == Some(i);
                let style = if is_selected {
                    ButtonStyle::Primary
                } else {
                    ButtonStyle::Ghost
                };
                let label = format!("{}  t={:.1}s  x{}", w.Name, w.StartTime, total);
                if ui.button(&label).style(style).draw() {
                    app.wave_edit.selected_wave = Some(i);
                    app.selection = Selection::Wave(i);
                }
                ui.spacer(2.0);
            }

            if waves.is_empty() {
                ui.label("(無 wave)").font_size(FS_LABEL).draw();
            }
        });
    });
}
