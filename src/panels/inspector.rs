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
                    if i >= app.map.Structures.len() {
                        return;
                    }
                    // 取出當前值複製，讓 UI 輸入後再寫回（避免 borrow conflict）
                    let mut tower = app.map.Structures[i].Tower.clone();
                    let mut faction = app.map.Structures[i].Faction.clone();
                    let mut x = app.map.Structures[i].X;
                    let mut y = app.map.Structures[i].Y;
                    let is_base = app.map.Structures[i].IsBase;

                    ui.label(&format!("Structure #{}", i)).font_size(16.0).draw();
                    ui.spacer(4.0);

                    let mut changed = false;
                    if ui.input("Tower (template)", &mut tower).draw() {
                        changed = true;
                    }
                    if ui.input("Faction", &mut faction).draw() {
                        changed = true;
                    }
                    if ui.slider("X", &mut x).range(-3000.0, 3000.0).draw() {
                        changed = true;
                        x = x.round();
                    }
                    if ui.slider("Y", &mut y).range(-3000.0, 3000.0).draw() {
                        changed = true;
                        y = y.round();
                    }

                    // IsBase toggle
                    let mut is_base_mut = is_base;
                    ui.spacer(4.0);
                    if ui
                        .button(if is_base_mut { "IsBase: YES" } else { "IsBase: no" })
                        .secondary()
                        .draw()
                    {
                        is_base_mut = !is_base_mut;
                        changed = true;
                    }

                    // 快捷切換 Faction 按鈕
                    if ui.button("切換 Player/Enemy").draw() {
                        faction = if faction == "Player" {
                            "Enemy".to_string()
                        } else {
                            "Player".to_string()
                        };
                        changed = true;
                    }

                    ui.spacer(12.0);
                    let mut delete = false;
                    if ui.button("Delete").draw() {
                        delete = true;
                    }

                    // 寫回 / delete
                    if delete {
                        app.map.Structures.remove(i);
                        app.selection = Selection::None;
                        app.dirty = true;
                    } else if changed {
                        let s = &mut app.map.Structures[i];
                        s.Tower = tower;
                        s.Faction = faction;
                        s.X = x;
                        s.Y = y;
                        s.IsBase = is_base_mut;
                        app.dirty = true;
                    }
                }
                Selection::CheckPoint(i) => {
                    if i >= app.map.CheckPoint.len() {
                        return;
                    }
                    let old_name = app.map.CheckPoint[i].Name.clone();
                    let mut name = old_name.clone();
                    let mut class = app.map.CheckPoint[i].Class.clone();
                    let mut x = app.map.CheckPoint[i].X;
                    let mut y = app.map.CheckPoint[i].Y;

                    ui.label(&format!("CheckPoint #{}", i)).font_size(16.0).draw();
                    ui.spacer(4.0);

                    let mut name_changed = false;
                    let mut other_changed = false;

                    if ui.input("Name", &mut name).draw() {
                        name_changed = true;
                    }
                    if ui.input("Class", &mut class).draw() {
                        other_changed = true;
                    }
                    if ui.slider("X", &mut x).range(-3000.0, 3000.0).draw() {
                        other_changed = true;
                        x = x.round();
                    }
                    if ui.slider("Y", &mut y).range(-3000.0, 3000.0).draw() {
                        other_changed = true;
                        y = y.round();
                    }

                    ui.spacer(12.0);
                    let mut delete = false;
                    if ui.button("Delete").draw() {
                        delete = true;
                    }

                    if delete {
                        // 刪除 CheckPoint 時同步從所有 Path 移除其引用
                        let removed_name = app.map.CheckPoint[i].Name.clone();
                        app.map.CheckPoint.remove(i);
                        for path in app.map.Path.iter_mut() {
                            path.Points.retain(|p| p != &removed_name);
                        }
                        app.selection = Selection::None;
                        app.dirty = true;
                    } else {
                        if name_changed && name != old_name && !name.is_empty() {
                            // 重命名：同步更新所有 Path.Points 裡舊名字
                            app.map.CheckPoint[i].Name = name.clone();
                            for path in app.map.Path.iter_mut() {
                                for p in path.Points.iter_mut() {
                                    if *p == old_name {
                                        *p = name.clone();
                                    }
                                }
                            }
                            app.dirty = true;
                        }
                        if other_changed {
                            let c = &mut app.map.CheckPoint[i];
                            c.Class = class;
                            c.X = x;
                            c.Y = y;
                            app.dirty = true;
                        }
                    }
                }
            }
        });
    });
}
