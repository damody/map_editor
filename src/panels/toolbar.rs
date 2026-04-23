use eui::quick::ui::UI;
use eui::{ButtonStyle, Rect};

use crate::app::{AppState, Tool, ViewMode};
use crate::io;
use crate::style::{FS_BODY, TOOLBAR_CELL_GAP, TOOLBAR_CELL_W, TOOLBAR_GROUP_GAP};

pub fn draw(ui: &mut UI, rect: Rect, app: &mut AppState) {
    ui.scope(rect, |ctx| {
        let mut ui = UI::new(ctx);
        let panel_color = ui.theme().panel;
        let r = ui.content_rect();
        ui.paint_filled_rect(r, panel_color, 0.0);

        let inner = eui::quick::ui::inset(&r, 8.0, 8.0);
        ui.scope(inner, |ctx| {
            let mut ui = UI::new(ctx);

            // 水平 6 欄：Open / Save / | / Select / AddTower / AddCheckPoint
            let row = ui.content_rect();
            let cell_w = TOOLBAR_CELL_W;
            let mut x = row.x;

            macro_rules! btn {
                ($label:expr, $style:expr) => {{
                    let br = Rect::new(x, row.y, cell_w, row.h);
                    x += cell_w + TOOLBAR_CELL_GAP;
                    let clicked = ui
                        .button($label)
                        .rect(br)
                        .style($style)
                        .draw();
                    clicked
                }};
            }

            if btn!("Open…", ButtonStyle::Primary) {
                match io::pick_and_load() {
                    Ok((path, data)) => {
                        app.map = data;
                        app.current_path = Some(path);
                        app.dirty = false;
                        app.selection = crate::app::Selection::None;
                        app.undo.clear();
                        if let Some(t) = app.map.Tower.first() {
                            app.new_tower_template = t.Name.clone();
                        }
                    }
                    Err(e) => eprintln!("Open failed: {}", e),
                }
            }
            if btn!("Save", ButtonStyle::Secondary) {
                if let Some(ref path) = app.current_path {
                    match io::save_to(path, &app.map) {
                        Ok(()) => {
                            app.dirty = false;
                            println!("Saved: {}", path.display());
                        }
                        Err(e) => eprintln!("Save failed: {}", e),
                    }
                }
            }
            if btn!("Save As…", ButtonStyle::Ghost) {
                if let Some(path) = io::pick_save_path() {
                    match io::save_to(&path, &app.map) {
                        Ok(()) => {
                            app.current_path = Some(path);
                            app.dirty = false;
                        }
                        Err(e) => eprintln!("Save failed: {}", e),
                    }
                }
            }
            if btn!("Save All", ButtonStyle::Primary) {
                if app.dirty {
                    if let Some(ref p) = app.current_path {
                        if let Err(e) = io::save_to(p, &app.map) { eprintln!("Save map failed: {}", e); }
                        else { app.dirty = false; println!("Saved: {}", p.display()); }
                    }
                }
                if app.entity_dirty {
                    if let Some(ref p) = app.entity_path {
                        if let Err(e) = io::save_entity_to(p, &app.entity) { eprintln!("Save entity failed: {}", e); }
                        else { app.entity_dirty = false; println!("Saved: {}", p.display()); }
                    }
                }
                if app.ability_dirty {
                    if let Some(ref p) = app.ability_path {
                        if let Err(e) = io::save_ability_to(p, &app.ability) { eprintln!("Save ability failed: {}", e); }
                        else { app.ability_dirty = false; println!("Saved: {}", p.display()); }
                    }
                }
                if app.mission_dirty {
                    if let Some(ref p) = app.mission_path {
                        if let Err(e) = io::save_mission_to(p, &app.mission) { eprintln!("Save mission failed: {}", e); }
                        else { app.mission_dirty = false; println!("Saved: {}", p.display()); }
                    }
                }
            }

            x += TOOLBAR_GROUP_GAP; // 分隔

            let tool_btn =
                |ui: &mut UI, label: &str, tool: Tool, app_tool: Tool, x: &mut f32| -> bool {
                    let br = Rect::new(*x, row.y, cell_w, row.h);
                    *x += cell_w + TOOLBAR_CELL_GAP;
                    let style = if app_tool == tool {
                        ButtonStyle::Primary
                    } else {
                        ButtonStyle::Ghost
                    };
                    ui.button(label).rect(br).style(style).draw()
                };

            if tool_btn(&mut ui, "Select", Tool::Select, app.tool, &mut x) {
                app.tool = Tool::Select;
            }
            if tool_btn(&mut ui, "+Tower", Tool::AddTower, app.tool, &mut x) {
                app.tool = Tool::AddTower;
            }
            if tool_btn(&mut ui, "+CheckPoint", Tool::AddCheckPoint, app.tool, &mut x) {
                app.tool = Tool::AddCheckPoint;
            }
            if tool_btn(&mut ui, "+Region", Tool::AddBlockedRegion, app.tool, &mut x) {
                app.tool = Tool::AddBlockedRegion;
                app.region_draft.clear();
            }
            if tool_btn(&mut ui, "Edit Region", Tool::EditBlockedRegion, app.tool, &mut x) {
                app.tool = Tool::EditBlockedRegion;
            }

            x += TOOLBAR_GROUP_GAP;
            // ViewMode 切換：Map / Entities
            {
                let br = Rect::new(x, row.y, cell_w, row.h);
                x += cell_w + TOOLBAR_CELL_GAP;
                let style = if app.view_mode == ViewMode::Map {
                    ButtonStyle::Primary
                } else {
                    ButtonStyle::Ghost
                };
                if ui.button("Map").rect(br).style(style).draw() {
                    app.view_mode = ViewMode::Map;
                }
            }
            {
                let br = Rect::new(x, row.y, cell_w, row.h);
                x += cell_w + TOOLBAR_CELL_GAP;
                let style = if app.view_mode == ViewMode::Entities {
                    ButtonStyle::Primary
                } else {
                    ButtonStyle::Ghost
                };
                if ui.button("Entities").rect(br).style(style).draw() {
                    app.view_mode = ViewMode::Entities;
                }
            }
            {
                let br = Rect::new(x, row.y, cell_w, row.h);
                x += cell_w + TOOLBAR_CELL_GAP;
                let style = if app.view_mode == ViewMode::Waves {
                    ButtonStyle::Primary
                } else {
                    ButtonStyle::Ghost
                };
                if ui.button("Waves").rect(br).style(style).draw() {
                    app.view_mode = ViewMode::Waves;
                }
            }

            x += TOOLBAR_GROUP_GAP;
            // 狀態文字：map + 其他已載入檔案的 dirty 標記
            let status = {
                let dir = app
                    .current_path
                    .as_ref()
                    .and_then(|p| p.parent())
                    .map(|d| d.display().to_string())
                    .unwrap_or_else(|| "(unsaved)".to_string());
                let flags = format!(
                    "{}{}{}{}",
                    if app.dirty { "M" } else { "·" },
                    if app.entity_dirty { "E" } else { "·" },
                    if app.ability_dirty { "A" } else { "·" },
                    if app.mission_dirty { "m" } else { "·" },
                );
                format!("[{}] {}", flags, dir)
            };
            let tw = 700.0_f32;
            ui.text(&status)
                .rect(Rect::new(x, row.y, tw, row.h))
                .font_size(FS_BODY)
                .draw();
        });
    });
}
