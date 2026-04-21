use eui::quick::ui::UI;
use eui::{ButtonStyle, Rect};

use crate::app::{AppState, Tool};
use crate::io;

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
            let cell_w = 100.0_f32;
            let mut x = row.x;

            macro_rules! btn {
                ($label:expr, $style:expr) => {{
                    let br = Rect::new(x, row.y, cell_w, row.h);
                    x += cell_w + 4.0;
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

            x += 16.0; // 分隔

            let tool_btn =
                |ui: &mut UI, label: &str, tool: Tool, app_tool: Tool, x: &mut f32| -> bool {
                    let br = Rect::new(*x, row.y, cell_w, row.h);
                    *x += cell_w + 4.0;
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

            x += 16.0;
            // 狀態文字
            let status = if let Some(ref p) = app.current_path {
                format!(
                    "{}{}",
                    if app.dirty { "● " } else { "" },
                    p.display()
                )
            } else {
                "(unsaved)".to_string()
            };
            let tw = 600.0_f32;
            ui.text(&status)
                .rect(Rect::new(x, row.y, tw, row.h))
                .font_size(15.0)
                .draw();
        });
    });
}
