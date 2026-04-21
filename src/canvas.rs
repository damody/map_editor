//! 中央 2D 地圖 viewport：繪製 + 滑鼠互動
use eui::quick::ui::UI;
use eui::{rgba, Color, Rect};

use crate::app::{AppState, DragState, Selection, Tool};
use crate::geometry::{point_in_polygon, point_segment_dist};
use crate::schema::{BlockedRegionJD, CheckPointJD, PointJD, StructureJD};

pub fn world_to_screen(app: &AppState, rect: &Rect, wx: f32, wy: f32) -> (f32, f32) {
    let cx = rect.x + rect.w * 0.5;
    let cy = rect.y + rect.h * 0.5;
    let sx = cx + (wx - app.pan.0) * app.zoom;
    let sy = cy + (-wy - app.pan.1) * app.zoom;
    (sx, sy)
}

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
    ui.paint_filled_rect(rect, rgba(0.15, 0.18, 0.20, 1.0), 0.0);

    draw_grid(ui, &rect, app);
    draw_paths(ui, &rect, app);
    draw_blocked_regions(ui, &rect, app);

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
        ui.text(&cp.Name)
            .rect(Rect::new(sx + 8.0, sy - 8.0, 100.0, 16.0))
            .color(Color::WHITE)
            .font_size(13.0)
            .draw();
    }

    // Structures
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
        if let Some(cr) = st.CollisionRadius {
            draw_circle(ui, sx, sy, cr * app.zoom, rgba(1.0, 0.8, 0.3, 0.7));
        }
        if selected {
            let s2 = size + 6.0;
            let outer = Rect::new(sx - s2 * 0.5, sy - s2 * 0.5, s2, s2);
            ui.paint_outline_rect(outer, rgba(1.0, 0.9, 0.0, 1.0), 2.0, 4.0);
        }
        ui.text(&st.Tower)
            .rect(Rect::new(sx - 40.0, sy + size * 0.5 + 2.0, 80.0, 14.0))
            .color(Color::WHITE)
            .font_size(12.0)
            .center()
            .draw();
    }

    if !app.region_draft.is_empty() {
        draw_region_draft(ui, &rect, app);
    }

    // === 滑鼠互動 ===
    let mx = ui.ctx().input().mouse_x;
    let my = ui.ctx().input().mouse_y;
    let in_canvas = point_in_rect(&rect, mx, my);

    if ui.ctx().input().mouse_middle_down {
        if let Some((px, py)) = app.prev_mouse_screen {
            let dx_screen = mx - px;
            let dy_screen = my - py;
            app.pan.0 -= dx_screen / app.zoom;
            app.pan.1 -= dy_screen / app.zoom;
        }
    }
    app.prev_mouse_screen = Some((mx, my));

    if in_canvas {
        let wheel = ui.ctx().input().mouse_wheel_y;
        if wheel.abs() > 0.01 {
            let old_zoom = app.zoom;
            app.zoom = (app.zoom * (1.0 + wheel * 0.1)).clamp(0.02, 5.0);
            let (wx_before, wy_before) = screen_to_world_raw(app, &rect, mx, my, old_zoom);
            let (wx_after, wy_after) = screen_to_world(app, &rect, mx, my);
            app.pan.0 += wx_before - wx_after;
            app.pan.1 += -(wy_before - wy_after);
        }
    }

    // 右鍵：AddBlockedRegion 時 commit 多邊形
    if in_canvas
        && ui.ctx().input().mouse_right_pressed
        && app.tool == Tool::AddBlockedRegion
    {
        commit_region_draft(app);
        return;
    }

    if in_canvas && ui.ctx().input().mouse_pressed {
        let now = std::time::Instant::now();
        let is_double = match (app.last_click_time, app.last_click_pos) {
            (Some(t), Some((px, py))) => {
                now.duration_since(t).as_millis() < 400
                    && (mx - px).abs() < 8.0
                    && (my - py).abs() < 8.0
            }
            _ => false,
        };

        if is_double {
            if app.tool == Tool::AddBlockedRegion {
                commit_region_draft(app);
                app.last_click_time = None;
                app.last_click_pos = None;
                return;
            }
            if try_insert_on_path(app, &rect, mx, my) {
                app.last_click_time = None;
                app.last_click_pos = None;
                return;
            }
        }
        app.last_click_time = Some(now);
        app.last_click_pos = Some((mx, my));

        match app.tool {
            Tool::Select => {
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
                for (ri, region) in app.map.BlockedRegions.iter().enumerate() {
                    for (pi, p) in region.Points.iter().enumerate() {
                        let (sx, sy) = world_to_screen(app, &rect, p.X, p.Y);
                        let d2 = (sx - mx).powi(2) + (sy - my).powi(2);
                        if d2 < 8.0 * 8.0 {
                            if best.map(|(_, d)| d2 < d).unwrap_or(true) {
                                best = Some((Selection::BlockedRegionPoint(ri, pi), d2));
                            }
                        }
                    }
                }
                if best.is_none() {
                    for (ri, region) in app.map.BlockedRegions.iter().enumerate() {
                        let poly: Vec<(f32, f32)> = region
                            .Points
                            .iter()
                            .map(|p| world_to_screen(app, &rect, p.X, p.Y))
                            .collect();
                        if point_in_polygon(mx, my, &poly) {
                            best = Some((Selection::BlockedRegion(ri), 0.0));
                            break;
                        }
                    }
                }
                app.selection = best.map(|(s, _)| s).unwrap_or(Selection::None);
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
                        Selection::BlockedRegionPoint(ri, pi) => {
                            let p = &app.map.BlockedRegions[ri].Points[pi];
                            (p.X, p.Y)
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
                app.map.CheckPoint.push(CheckPointJD {
                    Name: format!("cp_{}", idx),
                    Class: "Path".to_string(),
                    X: wx.round(),
                    Y: wy.round(),
                });
                app.dirty = true;
                app.selection = Selection::CheckPoint(idx);
            }
            Tool::AddBlockedRegion => {
                let (wx, wy) = screen_to_world(app, &rect, mx, my);
                app.region_draft.push(PointJD {
                    X: wx.round(),
                    Y: wy.round(),
                });
            }
            Tool::EditBlockedRegion => {
                let mut best: Option<(usize, usize, f32)> = None;
                for (ri, region) in app.map.BlockedRegions.iter().enumerate() {
                    for (pi, p) in region.Points.iter().enumerate() {
                        let (sx, sy) = world_to_screen(app, &rect, p.X, p.Y);
                        let d2 = (sx - mx).powi(2) + (sy - my).powi(2);
                        if d2 < 12.0 * 12.0 {
                            if best.map(|(_, _, d)| d2 < d).unwrap_or(true) {
                                best = Some((ri, pi, d2));
                            }
                        }
                    }
                }
                if let Some((ri, pi, _)) = best {
                    app.selection = Selection::BlockedRegionPoint(ri, pi);
                    let p = &app.map.BlockedRegions[ri].Points[pi];
                    app.drag_state = Some(DragState {
                        sel: app.selection,
                        orig_world_x: p.X,
                        orig_world_y: p.Y,
                        start_mouse_x: mx,
                        start_mouse_y: my,
                    });
                }
            }
        }
    }

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
                Selection::BlockedRegionPoint(ri, pi) => {
                    if let Some(p) = app
                        .map
                        .BlockedRegions
                        .get_mut(ri)
                        .and_then(|r| r.Points.get_mut(pi))
                    {
                        p.X = new_x;
                        p.Y = new_y;
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

fn commit_region_draft(app: &mut AppState) {
    if app.region_draft.len() >= 3 {
        let idx = app.map.BlockedRegions.len();
        app.map.BlockedRegions.push(BlockedRegionJD {
            Name: format!("region_{}", idx),
            Points: std::mem::take(&mut app.region_draft),
        });
        app.dirty = true;
        app.selection = Selection::BlockedRegion(idx);
    } else {
        app.region_draft.clear();
    }
}

fn draw_blocked_regions(ui: &mut UI, rect: &Rect, app: &AppState) {
    let fill = rgba(1.0, 0.2, 0.2, 0.25);
    let fill_sel = rgba(1.0, 0.8, 0.2, 0.35);
    let edge = rgba(1.0, 0.3, 0.3, 0.95);
    let vertex = rgba(1.0, 0.9, 0.4, 1.0);
    let vertex_sel = rgba(1.0, 1.0, 1.0, 1.0);

    for (ri, region) in app.map.BlockedRegions.iter().enumerate() {
        let selected_region = matches!(app.selection, Selection::BlockedRegion(i) if i == ri)
            || matches!(app.selection, Selection::BlockedRegionPoint(r, _) if r == ri);
        let n = region.Points.len();
        if n < 2 {
            continue;
        }
        let pts: Vec<(f32, f32)> = region
            .Points
            .iter()
            .map(|p| world_to_screen(app, rect, p.X, p.Y))
            .collect();
        let _f = if selected_region { fill_sel } else { fill };
        // TODO: eui Context 尚無 paint_triangle；暫僅畫邊線，待 eui 補上 fan-fill 後
        // 再恢復填色。
        for i in 0..n {
            let a = pts[i];
            let b = pts[(i + 1) % n];
            ui.ctx()
                .paint_line(a.0, a.1, b.0, b.1, edge, if selected_region { 2.5 } else { 1.5 });
        }
        for (pi, (sx, sy)) in pts.iter().enumerate() {
            let sel_pt = matches!(
                app.selection,
                Selection::BlockedRegionPoint(r, p) if r == ri && p == pi
            );
            let r = if sel_pt { 6.0 } else { 4.0 };
            let c = if sel_pt { vertex_sel } else { vertex };
            let rc = Rect::new(sx - r, sy - r, r * 2.0, r * 2.0);
            ui.paint_filled_rect(rc, c, r);
        }
        if n > 0 {
            ui.text(&region.Name)
                .rect(Rect::new(pts[0].0 + 6.0, pts[0].1 - 14.0, 100.0, 14.0))
                .color(Color::WHITE)
                .font_size(12.0)
                .draw();
        }
    }
}

fn draw_region_draft(ui: &mut UI, rect: &Rect, app: &AppState) {
    let color = rgba(1.0, 1.0, 0.3, 0.9);
    let mut prev: Option<(f32, f32)> = None;
    for p in &app.region_draft {
        let (sx, sy) = world_to_screen(app, rect, p.X, p.Y);
        let rc = Rect::new(sx - 4.0, sy - 4.0, 8.0, 8.0);
        ui.paint_filled_rect(rc, color, 4.0);
        if let Some((px, py)) = prev {
            ui.ctx().paint_line(px, py, sx, sy, color, 2.0);
        }
        prev = Some((sx, sy));
    }
    if let Some((px, py)) = prev {
        let mx = ui.ctx().input().mouse_x;
        let my = ui.ctx().input().mouse_y;
        ui.ctx()
            .paint_line(px, py, mx, my, rgba(1.0, 1.0, 0.3, 0.4), 1.5);
    }
}

fn draw_circle(ui: &mut UI, cx: f32, cy: f32, r_px: f32, color: Color) {
    const SEG: usize = 24;
    let mut prev: Option<(f32, f32)> = None;
    for i in 0..=SEG {
        let t = (i as f32) / (SEG as f32) * std::f32::consts::TAU;
        let x = cx + r_px * t.cos();
        let y = cy + r_px * t.sin();
        if let Some((px, py)) = prev {
            ui.ctx().paint_line(px, py, x, y, color, 1.0);
        }
        prev = Some((x, y));
    }
}

fn screen_to_world_raw(app: &AppState, rect: &Rect, sx: f32, sy: f32, zoom: f32) -> (f32, f32) {
    let cx = rect.x + rect.w * 0.5;
    let cy = rect.y + rect.h * 0.5;
    let wx = (sx - cx) / zoom + app.pan.0;
    let wy = -((sy - cy) / zoom + app.pan.1);
    (wx, wy)
}

fn try_insert_on_path(app: &mut AppState, rect: &Rect, mx: f32, my: f32) -> bool {
    const THRESHOLD_PX: f32 = 15.0;
    let cp_index: std::collections::HashMap<String, usize> = app
        .map
        .CheckPoint
        .iter()
        .enumerate()
        .map(|(i, c)| (c.Name.clone(), i))
        .collect();

    let mut best: Option<(String, String, f32)> = None;
    for path in app.map.Path.iter() {
        if path.Points.len() < 2 {
            continue;
        }
        for i in 0..path.Points.len() - 1 {
            let a_name = &path.Points[i];
            let b_name = &path.Points[i + 1];
            let a = match cp_index
                .get(a_name)
                .and_then(|&idx| app.map.CheckPoint.get(idx))
            {
                Some(c) => c,
                None => continue,
            };
            let b = match cp_index
                .get(b_name)
                .and_then(|&idx| app.map.CheckPoint.get(idx))
            {
                Some(c) => c,
                None => continue,
            };
            let (ax, ay) = world_to_screen(app, rect, a.X, a.Y);
            let (bx, by) = world_to_screen(app, rect, b.X, b.Y);
            let d = point_segment_dist(mx, my, ax, ay, bx, by);
            if d < THRESHOLD_PX {
                if best.as_ref().map(|(_, _, bd)| d < *bd).unwrap_or(true) {
                    best = Some((a_name.clone(), b_name.clone(), d));
                }
            }
        }
    }

    let (a_name, b_name, _) = match best {
        Some(x) => x,
        None => return false,
    };

    let (wx, wy) = screen_to_world(app, rect, mx, my);
    let mut n = app.map.CheckPoint.len();
    let new_name = loop {
        let cand = format!("cp_{}", n);
        if !app.map.CheckPoint.iter().any(|c| c.Name == cand) {
            break cand;
        }
        n += 1;
    };
    app.map.CheckPoint.push(CheckPointJD {
        Name: new_name.clone(),
        Class: "Path".to_string(),
        X: wx.round(),
        Y: wy.round(),
    });

    let mut inserted_any = false;
    for path in app.map.Path.iter_mut() {
        if path.Points.len() < 2 {
            continue;
        }
        let mut i = path.Points.len().saturating_sub(2);
        loop {
            let p0 = &path.Points[i];
            let p1 = &path.Points[i + 1];
            let hit = (p0 == &a_name && p1 == &b_name) || (p0 == &b_name && p1 == &a_name);
            if hit {
                path.Points.insert(i + 1, new_name.clone());
                inserted_any = true;
            }
            if i == 0 {
                break;
            }
            i -= 1;
        }
    }

    if inserted_any {
        app.dirty = true;
        app.selection = Selection::CheckPoint(app.map.CheckPoint.len() - 1);
        true
    } else {
        app.map.CheckPoint.pop();
        false
    }
}

fn draw_grid(ui: &mut UI, rect: &Rect, app: &AppState) {
    let color = rgba(0.25, 0.28, 0.30, 1.0);
    let step = 100.0_f32;
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
    let (ox, oy) = world_to_screen(app, rect, 0.0, 0.0);
    let axis_c = rgba(0.6, 0.6, 0.3, 1.0);
    ui.ctx().paint_line(ox - 20.0, oy, ox + 20.0, oy, axis_c, 1.5);
    ui.ctx().paint_line(ox, oy - 20.0, ox, oy + 20.0, axis_c, 1.5);
}

fn draw_paths(ui: &mut UI, rect: &Rect, app: &AppState) {
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
