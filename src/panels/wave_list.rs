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

            ui.spacer(12.0);
            if ui.button("+ Add Wave").style(ButtonStyle::Primary).draw() {
                app.begin_edit(None);
                let new_idx = crate::wave_ops::add_wave(&mut app.map);
                app.wave_edit.selected_wave = Some(new_idx);
                app.selection = Selection::Wave(new_idx);
                app.dirty = true;
            }
            ui.spacer(2.0);

            if let Some(sel) = app.wave_edit.selected_wave {
                if ui.button("Duplicate").secondary().draw() {
                    app.begin_edit(None);
                    if let Some(new_idx) = crate::wave_ops::duplicate_wave(&mut app.map, sel) {
                        app.wave_edit.selected_wave = Some(new_idx);
                        app.selection = Selection::Wave(new_idx);
                        app.dirty = true;
                    }
                }
                ui.spacer(2.0);

                let confirming = matches!(
                    app.wave_edit.pending_delete_wave,
                    Some((i, t)) if i == sel && t.elapsed().as_secs() < 5
                );
                let label = if confirming { "再點一次刪除" } else { "Delete" };
                let style = if confirming {
                    ButtonStyle::Primary
                } else {
                    ButtonStyle::Ghost
                };
                if ui.button(label).style(style).draw() {
                    if confirming {
                        app.begin_edit(None);
                        crate::wave_ops::delete_wave(&mut app.map, sel);
                        app.wave_edit.selected_wave = None;
                        app.selection = Selection::None;
                        app.wave_edit.pending_delete_wave = None;
                        app.dirty = true;
                    } else {
                        app.wave_edit.pending_delete_wave = Some((sel, std::time::Instant::now()));
                    }
                }
            }
        });
    });
}
