use eui::quick::ui::UI;
use eui::Rect;

use crate::app::{AppState, Selection};

pub fn draw(ui: &mut UI, rect: Rect, app: &mut AppState) {
    ui.scope(rect, |ctx| {
        let mut ui = UI::new(ctx);
        let panel_color = ui.theme().panel;
        let r = ui.content_rect();
        ui.paint_filled_rect(r, panel_color, 0.0);

        let inner = eui::quick::ui::inset(&r, 10.0, 10.0);
        ui.scope(inner, |ctx| {
            let mut ui = UI::new(ctx);
            ui.label("Inspector").font_size(18.0).height(28.0).draw();
            ui.spacer(6.0);

            match app.selection {
                Selection::None => {
                    ui.label("(未選中物件)").font_size(15.0).draw();
                }
                Selection::Structure(i) => {
                    if let Some(s) = app.map.Structures.get_mut(i) {
                        ui.label(&format!("Structure #{}", i)).font_size(16.0).draw();
                        ui.spacer(4.0);
                        ui.label(&format!("Tower: {}", s.Tower)).draw();
                        ui.label(&format!("Faction: {}", s.Faction)).draw();
                        ui.label(&format!("IsBase: {}", s.IsBase)).draw();
                        ui.spacer(6.0);
                        let mut x = s.X;
                        if ui.slider("X", &mut x).range(-3000.0, 3000.0).draw() {
                            s.X = x.round();
                            app.dirty = true;
                        }
                        let mut y = s.Y;
                        if ui.slider("Y", &mut y).range(-3000.0, 3000.0).draw() {
                            s.Y = y.round();
                            app.dirty = true;
                        }
                        ui.spacer(6.0);
                        if ui.button("切換 Faction").draw() {
                            s.Faction = if s.Faction == "Player" {
                                "Enemy".to_string()
                            } else {
                                "Player".to_string()
                            };
                            app.dirty = true;
                        }
                        if ui.button("切換 IsBase").secondary().draw() {
                            s.IsBase = !s.IsBase;
                            app.dirty = true;
                        }
                        ui.spacer(12.0);
                        if ui.button("Delete").draw() {
                            app.map.Structures.remove(i);
                            app.selection = Selection::None;
                            app.dirty = true;
                        }
                    }
                }
                Selection::CheckPoint(i) => {
                    if let Some(c) = app.map.CheckPoint.get_mut(i) {
                        ui.label(&format!("CheckPoint #{}", i)).font_size(16.0).draw();
                        ui.spacer(4.0);
                        ui.input("Name", &mut c.Name).draw();
                        ui.input("Class", &mut c.Class).draw();
                        let mut x = c.X;
                        if ui.slider("X", &mut x).range(-3000.0, 3000.0).draw() {
                            c.X = x.round();
                            app.dirty = true;
                        }
                        let mut y = c.Y;
                        if ui.slider("Y", &mut y).range(-3000.0, 3000.0).draw() {
                            c.Y = y.round();
                            app.dirty = true;
                        }
                        ui.spacer(12.0);
                        if ui.button("Delete").draw() {
                            app.map.CheckPoint.remove(i);
                            app.selection = Selection::None;
                            app.dirty = true;
                        }
                    }
                }
            }
        });
    });
}
