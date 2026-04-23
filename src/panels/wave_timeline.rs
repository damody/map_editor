use eui::quick::ui::UI;
use eui::{Rect, TextAlign};

use crate::app::{AppState, Selection, SpawnDrag, WaveZoom};
use crate::style::{
    FS_CAPTION, FS_LABEL, FS_SUBHEAD, WAVE_DOT_R, WAVE_HEADER_H, WAVE_LANE_H, WAVE_RULER_H,
};

/// 由 creep_name hash 決定顏色（固定 8 色 palette）
fn creep_color(name: &str) -> eui::Color {
    const PALETTE: [(f32, f32, f32); 8] = [
        (0.30, 0.78, 0.45),
        (0.85, 0.30, 0.30),
        (0.30, 0.55, 0.85),
        (0.95, 0.75, 0.20),
        (0.75, 0.40, 0.85),
        (0.40, 0.80, 0.80),
        (0.95, 0.55, 0.25),
        (0.65, 0.65, 0.70),
    ];
    let mut h: u32 = 5381;
    for b in name.bytes() {
        h = h.wrapping_mul(33).wrapping_add(b as u32);
    }
    let (r, g, b) = PALETTE[(h as usize) % PALETTE.len()];
    eui::rgba(r, g, b, 1.0)
}

fn creep_letter(name: &str) -> String {
    name.chars()
        .skip_while(|c| !c.is_ascii_alphabetic())
        .take(1)
        .collect::<String>()
        .to_uppercase()
}

pub fn draw(ui: &mut UI, rect: Rect, app: &mut AppState) {
    ui.scope(rect, |ctx| {
        let mut ui = UI::new(ctx);
        let bg = eui::rgba(0.10, 0.11, 0.12, 1.0);
        let r = ui.content_rect();
        ui.paint_filled_rect(r, bg, 0.0);

        let input = ui.ctx().input().clone();
        let (mx, my) = (input.mouse_x, input.mouse_y);

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

        // 計算 px_per_sec 先用尚未更新的 map（drag 前），避免畫面閃爍
        let total_sec = app.map.CreepWave[w_idx]
            .Detail
            .iter()
            .flat_map(|d| d.Creeps.iter().map(|c| c.Time))
            .fold(0.0_f32, f32::max)
            + 0.5;
        let total_sec = total_sec.max(1.0);
        let px_per_sec = match app.wave_edit.zoom_mode {
            WaveZoom::Fit => ((r.w - 16.0 - 110.0) / total_sec).max(1.0),
            WaveZoom::Fixed(s) => s,
        };

        // ── 處理 drag 更新（在畫之前修改 map，本幀即可看到新位置）──
        if let Some(drag) = app.wave_edit.drag.clone() {
            let new_time = drag.orig_time + (mx - drag.start_mouse_x) / px_per_sec;
            let delta = new_time - drag.orig_time;
            let (dw, dd, ds) = drag.sel;
            if drag.batch_after {
                for (offset, ot) in drag.orig_times.iter().enumerate() {
                    let target_idx = ds + offset;
                    if let Some(sp) =
                        app.map.CreepWave[dw].Detail[dd].Creeps.get_mut(target_idx)
                    {
                        sp.Time = (ot + delta).max(0.0);
                    }
                }
            } else {
                crate::wave_ops::drag_spawn_time(
                    &mut app.map.CreepWave[dw],
                    dd,
                    ds,
                    new_time,
                );
            }
            app.dirty = true;
            if !input.mouse_down {
                app.wave_edit.drag = None;
            }
        }

        let wave = app.map.CreepWave[w_idx].clone();

        let header = Rect::new(r.x, r.y, r.w, WAVE_HEADER_H);
        let title = format!("{}  StartTime={:.1}s", wave.Name, wave.StartTime);
        let muted = ui.theme().muted_text;
        ui.ctx()
            .paint_text(header, &title, FS_SUBHEAD, muted, TextAlign::Left);

        let ruler_y = r.y + WAVE_HEADER_H;
        let ruler_rect = Rect::new(r.x + 8.0 + 110.0, ruler_y, r.w - 16.0 - 110.0, WAVE_RULER_H);
        let ruler_color = eui::rgba(0.25, 0.27, 0.30, 1.0);
        ui.paint_filled_rect(ruler_rect, ruler_color, 0.0);
        let scroll_x = app.wave_edit.scroll_x;
        let max_visible_sec = (ruler_rect.w / px_per_sec).ceil() as i32 + 1;
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

        let mut hit_spawn: Option<(usize, usize, usize, f32)> = None;
        let mut hit_lane: Option<(usize, usize)> = None;

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
                100.0,
                WAVE_LANE_H - 8.0,
            );
            ui.ctx().paint_text(
                header_rect,
                &detail.Path,
                FS_LABEL,
                eui::rgba(0.85, 0.85, 0.85, 1.0),
                TextAlign::Left,
            );

            let lane_origin_x = lane_rect.x + 110.0;
            let cy = lane_rect.y + lane_rect.h * 0.5;
            for (si, spawn) in detail.Creeps.iter().enumerate() {
                let cx = lane_origin_x + spawn.Time * px_per_sec - scroll_x;
                if cx < lane_origin_x - WAVE_DOT_R
                    || cx > lane_rect.x + lane_rect.w + WAVE_DOT_R
                {
                    continue;
                }
                let dot_rect = Rect::new(
                    cx - WAVE_DOT_R,
                    cy - WAVE_DOT_R,
                    WAVE_DOT_R * 2.0,
                    WAVE_DOT_R * 2.0,
                );
                let color = creep_color(&spawn.Creep);
                ui.paint_filled_rect(dot_rect, color, WAVE_DOT_R);

                let letter = creep_letter(&spawn.Creep);
                ui.ctx().paint_text(
                    dot_rect,
                    &letter,
                    FS_LABEL,
                    eui::rgba(1.0, 1.0, 1.0, 1.0),
                    TextAlign::Center,
                );

                if let Selection::WaveSpawn(ws, ds, ss) = app.selection {
                    if ws == w_idx && ds == di && ss == si {
                        let outline_r = WAVE_DOT_R + 2.0;
                        let outline = Rect::new(
                            cx - outline_r,
                            cy - outline_r,
                            outline_r * 2.0,
                            outline_r * 2.0,
                        );
                        ui.paint_filled_rect(
                            outline,
                            eui::rgba(1.0, 0.9, 0.2, 0.4),
                            outline_r,
                        );
                    }
                }

                let dx = mx - cx;
                let dy = my - cy;
                if dx * dx + dy * dy <= WAVE_DOT_R * WAVE_DOT_R {
                    hit_spawn = Some((w_idx, di, si, spawn.Time));
                }
            }

            if hit_spawn.is_none() && lane_rect.contains(mx, my) {
                hit_lane = Some((w_idx, di));
            }
        }

        // 點擊 → 選中 + 可能啟動 drag
        if input.mouse_pressed && app.wave_edit.drag.is_none() {
            if let Some((w, d, s, orig_time)) = hit_spawn {
                app.selection = Selection::WaveSpawn(w, d, s);
                let orig_times: Vec<f32> = app.map.CreepWave[w].Detail[d].Creeps[s..]
                    .iter()
                    .map(|c| c.Time)
                    .collect();
                app.begin_edit(Some("wave_drag_time"));
                app.wave_edit.drag = Some(SpawnDrag {
                    sel: (w, d, s),
                    start_mouse_x: mx,
                    orig_time,
                    batch_after: input.key_shift,
                    orig_times,
                });
            } else if let Some((w, d)) = hit_lane {
                app.selection = Selection::WaveDetail(w, d);
            }
        }
    });
}
