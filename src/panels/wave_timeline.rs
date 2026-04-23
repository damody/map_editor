use eui::quick::ui::UI;
use eui::{Rect, TextAlign};

use crate::app::{AppState, WaveZoom};
use crate::style::{
    FS_CAPTION, FS_LABEL, FS_SUBHEAD, WAVE_HEADER_H, WAVE_LANE_H, WAVE_RULER_H,
};

pub fn draw(ui: &mut UI, rect: Rect, app: &mut AppState) {
    ui.scope(rect, |ctx| {
        let mut ui = UI::new(ctx);
        let bg = eui::rgba(0.10, 0.11, 0.12, 1.0);
        let r = ui.content_rect();
        ui.paint_filled_rect(r, bg, 0.0);

        let Some(w_idx) = app.wave_edit.selected_wave else {
            let text_color = eui::rgba(0.6, 0.6, 0.6, 1.0);
            ui.ctx().paint_text(
                r,
                "(請從左側選擇一個 Wave)",
                FS_LABEL,
                text_color,
                TextAlign::Center,
            );
            return;
        };
        if w_idx >= app.map.CreepWave.len() {
            return;
        }
        let wave = app.map.CreepWave[w_idx].clone();

        let total_sec = wave
            .Detail
            .iter()
            .flat_map(|d| d.Creeps.iter().map(|c| c.Time))
            .fold(0.0_f32, f32::max)
            + 0.5;
        let total_sec = total_sec.max(1.0);
        let px_per_sec = match app.wave_edit.zoom_mode {
            WaveZoom::Fit => ((r.w - 16.0) / total_sec).max(1.0),
            WaveZoom::Fixed(s) => s,
        };

        let header = Rect::new(r.x, r.y, r.w, WAVE_HEADER_H);
        let title = format!("{}  StartTime={:.1}s", wave.Name, wave.StartTime);
        let muted = ui.theme().muted_text;
        ui.ctx()
            .paint_text(header, &title, FS_SUBHEAD, muted, TextAlign::Left);

        let ruler_y = r.y + WAVE_HEADER_H;
        let ruler_rect = Rect::new(r.x + 8.0, ruler_y, r.w - 16.0, WAVE_RULER_H);
        let ruler_color = eui::rgba(0.25, 0.27, 0.30, 1.0);
        ui.paint_filled_rect(ruler_rect, ruler_color, 0.0);
        let scroll_x = app.wave_edit.scroll_x;
        let max_visible_sec = ((r.w - 16.0) / px_per_sec).ceil() as i32 + 1;
        for s in 0..max_visible_sec {
            let cx = ruler_rect.x + s as f32 * px_per_sec - scroll_x;
            if cx < ruler_rect.x || cx > ruler_rect.x + ruler_rect.w {
                continue;
            }
            let line = Rect::new(cx - 0.5, ruler_y, 1.0, WAVE_RULER_H);
            ui.paint_filled_rect(line, eui::rgba(0.5, 0.5, 0.5, 1.0), 0.0);
            let lbl = Rect::new(cx + 2.0, ruler_y, 30.0, WAVE_RULER_H);
            ui.ctx().paint_text(
                lbl,
                &format!("{}s", s),
                FS_CAPTION,
                eui::rgba(0.7, 0.7, 0.7, 1.0),
                TextAlign::Left,
            );
        }

        let lanes_y = ruler_y + WAVE_RULER_H + 4.0;
        for (di, detail) in wave.Detail.iter().enumerate() {
            let ly = lanes_y + di as f32 * (WAVE_LANE_H + 2.0);
            let lane_rect = Rect::new(r.x + 8.0, ly, r.w - 16.0, WAVE_LANE_H);
            let zebra = if di % 2 == 0 {
                eui::rgba(0.16, 0.17, 0.19, 1.0)
            } else {
                eui::rgba(0.13, 0.14, 0.16, 1.0)
            };
            ui.paint_filled_rect(lane_rect, zebra, 4.0);

            let header_rect = Rect::new(
                lane_rect.x + 6.0,
                lane_rect.y + 4.0,
                120.0,
                WAVE_LANE_H - 8.0,
            );
            ui.ctx().paint_text(
                header_rect,
                &detail.Path,
                FS_LABEL,
                eui::rgba(0.85, 0.85, 0.85, 1.0),
                TextAlign::Left,
            );
        }

        let _ = app;
    });
}
