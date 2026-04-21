//! 中央 2D 地圖 viewport：繪製 + 滑鼠互動
use eui::quick::ui::UI;
use eui::{rgba, Color, Rect};

use crate::app::{AppState, DragState, Selection, Tool};
use crate::schema::StructureJD;

/// 世界座標 → 螢幕像素（在 rect 內）
pub fn world_to_screen(app: &AppState, rect: &Rect, wx: f32, wy: f32) -> (f32, f32) {
    let cx = rect.x + rect.w * 0.5;
    let cy = rect.y + rect.h * 0.5;
    let sx = cx + (wx - app.pan.0) * app.zoom;
    // 螢幕 Y 往下、world Y 往上：-wy
    let sy = cy + (-wy - app.pan.1) * app.zoom;
    (sx, sy)
}

/// 螢幕像素 → 世界座標
pub fn screen_to_world(app: &AppState, rect: &Rect, sx: f32, sy: f32) -> (f32, f32) {
    let cx = rect.x + rect.w * 0.5;
    let cy = rect.y + rect.h * 0.5;
    let wx = (sx - cx) / app.zoom + app.pan.0;
    let wy = -((sy - cy) / app.zoom + app.pan.1);
    (wx, wy)
}

fn point_in_rect(rect: &Rect, x: f32, y: f32) -> bool {
    x >= rect.x && x <= rect.x + rect.w && y >= rect.y && y <= rect.y + rect.h
}

pub fn draw(ui: &mut UI, rect: Rect, app: &mut AppState) {
    // 背景
    ui.paint_filled_rect(rect, rgba(0.15, 0.18, 0.20, 1.0), 0.0);

    // Grid（每 100 world unit 一條線）
    draw_grid(ui, &rect, app);

    // 路徑線
    draw_paths(ui, &rect, app);

    // CheckPoints
    for (i, cp) in app.map.CheckPoint.iter().enumerate() {
        let (sx, sy) = world_to_screen(app, &rect, cp.X, cp.Y);
        let selected = matches!(app.selection, Selection::CheckPoint(idx) if idx == i);
        let size = if selected { 14.0 } else { 10.0 };
        let color = match cp.Class.as_str() {
            "Base" => rgba(1.0, 0.8, 0.2, 1.0),
            "Spawn" => rgba(0.2, 1.0, 0.4, 1.0),
            "Tower" => rgba(0.7, 0.7, 1.0, 1.0),
            _ => rgba(0.8, 0.8, 0.8, 1.0),
        };
        let r = Rect::new(sx - size * 0.5, sy - size * 0.5, size, size);
        ui.paint_filled_rect(r, color, size * 0.5);
        // 名字
        ui.text(&cp.Name)
            .rect(Rect::new(sx + 8.0, sy - 8.0, 100.0, 16.0))
            .color(Color::WHITE)
            .font_size(13.0)
            .draw();
    }

    // Structures（塔/基地）
    for (i, st) in app.map.Structures.iter().enumerate() {
        let (sx, sy) = world_to_screen(app, &rect, st.X, st.Y);
        let selected = matches!(app.selection, Selection::Structure(idx) if idx == i);
        let size = if st.IsBase { 28.0 } else { 20.0 };
        let color = match st.Faction.as_str() {
            "Player" | "player" => rgba(0.2, 0.5, 1.0, 1.0),
            _ => rgba(1.0, 0.3, 0.3, 1.0),
        };
        let r = Rect::new(sx - size * 0.5, sy - size * 0.5, size, size);
        ui.paint_filled_rect(r, color, 4.0);
        if selected {
            // 黃色邊框 by 畫大一點的方塊在後面（偷懶用 4 條邊）
            let s2 = size + 6.0;
            let outer = Rect::new(sx - s2 * 0.5, sy - s2 * 0.5, s2, s2);
            ui.paint_outline_rect(outer, rgba(1.0, 0.9, 0.0, 1.0), 2.0, 4.0);
        }
        // tower 名稱
        ui.text(&st.Tower)
            .rect(Rect::new(sx - 40.0, sy + size * 0.5 + 2.0, 80.0, 14.0))
            .color(Color::WHITE)
            .font_size(12.0)
            .center()
            .draw();
    }

    // === 滑鼠互動 ===
    // 只在滑鼠於 canvas rect 內才處理
    let mx = ui.ctx().input().mouse_x;
    let my = ui.ctx().input().mouse_y;
    let in_canvas = point_in_rect(&rect, mx, my);

    // Pan：中鍵拖拉。用上一 frame 的滑鼠位置算 delta，除以 zoom 換算成世界座標。
    if ui.ctx().input().mouse_middle_down {
        if let Some((px, py)) = app.prev_mouse_screen {
            let dx_screen = mx - px;
            let dy_screen = my - py;
            app.pan.0 -= dx_screen / app.zoom;
            app.pan.1 -= dy_screen / app.zoom;
        }
    }
    // 更新 prev（不論是否在 canvas 內都記，拖出去再拖回來才不會跳）
    app.prev_mouse_screen = Some((mx, my));

    // zoom: wheel
    if in_canvas {
        let wheel = ui.ctx().input().mouse_wheel_y;
        if wheel.abs() > 0.01 {
            let old_zoom = app.zoom;
            app.zoom = (app.zoom * (1.0 + wheel * 0.1)).clamp(0.02, 5.0);
            // 以滑鼠為中心縮放：調整 pan 讓 mouse 指的 world 點保持不動
            let (wx_before, wy_before) = screen_to_world_raw(app, &rect, mx, my, old_zoom);
            let (wx_after, wy_after) = screen_to_world(app, &rect, mx, my);
            app.pan.0 += wx_before - wx_after;
            app.pan.1 += -(wy_before - wy_after);
        }
    }

    // 左鍵：依 tool 決定行為
    if in_canvas && ui.ctx().input().mouse_pressed {
        match app.tool {
            Tool::Select => {
                // 找最近的 Structure 或 CheckPoint
                let mut best: Option<(Selection, f32)> = None;
                for (i, st) in app.map.Structures.iter().enumerate() {
                    let (sx, sy) = world_to_screen(app, &rect, st.X, st.Y);
                    let d2 = (sx - mx).powi(2) + (sy - my).powi(2);
                    let hit = if st.IsBase { 20.0 } else { 14.0 };
                    if d2 < hit * hit {
                        if best.map(|(_, d)| d2 < d).unwrap_or(true) {
                            best = Some((Selection::Structure(i), d2));
                        }
                    }
                }
                for (i, cp) in app.map.CheckPoint.iter().enumerate() {
                    let (sx, sy) = world_to_screen(app, &rect, cp.X, cp.Y);
                    let d2 = (sx - mx).powi(2) + (sy - my).powi(2);
                    if d2 < 10.0 * 10.0 {
                        if best.map(|(_, d)| d2 < d).unwrap_or(true) {
                            best = Some((Selection::CheckPoint(i), d2));
                        }
                    }
                }
                app.selection = best.map(|(s, _)| s).unwrap_or(Selection::None);
                // 若選中某物，開始拖拉
                if app.selection != Selection::None {
                    let (ox, oy) = match app.selection {
                        Selection::Structure(i) => {
                            let s = &app.map.Structures[i];
                            (s.X, s.Y)
                        }
                        Selection::CheckPoint(i) => {
                            let c = &app.map.CheckPoint[i];
                            (c.X, c.Y)
                        }
                        _ => (0.0, 0.0),
                    };
                    app.drag_state = Some(DragState {
                        sel: app.selection,
                        orig_world_x: ox,
                        orig_world_y: oy,
                        start_mouse_x: mx,
                        start_mouse_y: my,
                    });
                }
            }
            Tool::AddTower => {
                if !app.new_tower_template.is_empty() {
                    let (wx, wy) = screen_to_world(app, &rect, mx, my);
                    app.map.Structures.push(StructureJD {
                        Tower: app.new_tower_template.clone(),
                        Faction: app.new_tower_faction.clone(),
                        X: wx.round(),
                        Y: wy.round(),
                        IsBase: app.new_tower_is_base,
                        CollisionRadius: None,
                    });
                    app.dirty = true;
                    app.selection = Selection::Structure(app.map.Structures.len() - 1);
                }
            }
            Tool::AddCheckPoint => {
                let (wx, wy) = screen_to_world(app, &rect, mx, my);
                let idx = app.map.CheckPoint.len();
                app.map.CheckPoint.push(crate::schema::CheckPointJD {
                    Name: format!("cp_{}", idx),
                    Class: "Path".to_string(),
                    X: wx.round(),
                    Y: wy.round(),
                });
                app.dirty = true;
                app.selection = Selection::CheckPoint(idx);
            }
        }
    }

    // 拖拉移動
    if let Some(ds) = app.drag_state {
        if ui.ctx().input().mouse_down {
            let dx = (mx - ds.start_mouse_x) / app.zoom;
            let dy = -(my - ds.start_mouse_y) / app.zoom;
            let new_x = (ds.orig_world_x + dx).round();
            let new_y = (ds.orig_world_y + dy).round();
            match ds.sel {
                Selection::Structure(i) => {
                    if let Some(s) = app.map.Structures.get_mut(i) {
                        s.X = new_x;
                        s.Y = new_y;
                        app.dirty = true;
                    }
                }
                Selection::CheckPoint(i) => {
                    if let Some(c) = app.map.CheckPoint.get_mut(i) {
                        c.X = new_x;
                        c.Y = new_y;
                        app.dirty = true;
                    }
                }
                _ => {}
            }
        } else {
            app.drag_state = None;
        }
    }
}

/// 同 screen_to_world 但用指定 zoom（用於 zoom 中心校正）
fn screen_to_world_raw(app: &AppState, rect: &Rect, sx: f32, sy: f32, zoom: f32) -> (f32, f32) {
    let cx = rect.x + rect.w * 0.5;
    let cy = rect.y + rect.h * 0.5;
    let wx = (sx - cx) / zoom + app.pan.0;
    let wy = -((sy - cy) / zoom + app.pan.1);
    (wx, wy)
}

fn draw_grid(ui: &mut UI, rect: &Rect, app: &AppState) {
    let color = rgba(0.25, 0.28, 0.30, 1.0);
    let step = 100.0_f32;
    // 計算 viewport 的世界座標範圍
    let (wx0, wy0) = screen_to_world(app, rect, rect.x, rect.y + rect.h);
    let (wx1, wy1) = screen_to_world(app, rect, rect.x + rect.w, rect.y);
    let x_start = (wx0 / step).floor() as i32;
    let x_end = (wx1 / step).ceil() as i32;
    for gx in x_start..=x_end {
        let wx = gx as f32 * step;
        let (sx, _) = world_to_screen(app, rect, wx, 0.0);
        if sx >= rect.x && sx <= rect.x + rect.w {
            ui.ctx().paint_line(sx, rect.y, sx, rect.y + rect.h, color, 1.0);
        }
    }
    let y_start = (wy0 / step).floor() as i32;
    let y_end = (wy1 / step).ceil() as i32;
    for gy in y_start..=y_end {
        let wy = gy as f32 * step;
        let (_, sy) = world_to_screen(app, rect, 0.0, wy);
        if sy >= rect.y && sy <= rect.y + rect.h {
            ui.ctx().paint_line(rect.x, sy, rect.x + rect.w, sy, color, 1.0);
        }
    }
    // 原點十字
    let (ox, oy) = world_to_screen(app, rect, 0.0, 0.0);
    let axis_c = rgba(0.6, 0.6, 0.3, 1.0);
    ui.ctx().paint_line(ox - 20.0, oy, ox + 20.0, oy, axis_c, 1.5);
    ui.ctx().paint_line(ox, oy - 20.0, ox, oy + 20.0, axis_c, 1.5);
}

fn draw_paths(ui: &mut UI, rect: &Rect, app: &AppState) {
    // 建 name -> index 查表
    let cp_map: std::collections::HashMap<&str, &crate::schema::CheckPointJD> = app
        .map
        .CheckPoint
        .iter()
        .map(|c| (c.Name.as_str(), c))
        .collect();
    for path in app.map.Path.iter() {
        let color = rgba(1.0, 1.0, 1.0, 0.35);
        let mut prev: Option<(f32, f32)> = None;
        for pname in path.Points.iter() {
            if let Some(cp) = cp_map.get(pname.as_str()) {
                let (sx, sy) = world_to_screen(app, rect, cp.X, cp.Y);
                if let Some((px, py)) = prev {
                    ui.ctx().paint_line(px, py, sx, sy, color, 2.0);
                }
                prev = Some((sx, sy));
            }
        }
    }
}
