use eui::quick::ui::UI;
use eui::{ButtonStyle, Rect};

use crate::app::AppState;

pub fn draw(ui: &mut UI, rect: Rect, app: &mut AppState) {
    ui.scope(rect, |ctx| {
        let mut ui = UI::new(ctx);
        let panel_color = ui.theme().panel;
        let r = ui.content_rect();
        ui.paint_filled_rect(r, panel_color, 0.0);

        let inner = eui::quick::ui::inset(&r, 10.0, 10.0);
        ui.scope(inner, |ctx| {
            let mut ui = UI::new(ctx);

            ui.label("Tower 模板").font_size(18.0).height(28.0).draw();
            ui.spacer(4.0);

            // 列出所有 Tower 模板，點擊 → 設為當前新增模板
            let mut selected_template = app.new_tower_template.clone();
            let towers = app.map.Tower.clone();
            for t in towers.iter() {
                let style = if selected_template == t.Name {
                    ButtonStyle::Primary
                } else {
                    ButtonStyle::Ghost
                };
                if ui
                    .button(&format!("{}  HP:{}  Rng:{:.0}", t.Name, t.Property.Hp, t.Attack.Range))
                    .style(style)
                    .draw()
                {
                    selected_template = t.Name.clone();
                }
                ui.spacer(2.0);
            }
            app.new_tower_template = selected_template;

            ui.spacer(12.0);
            ui.label(&format!("Faction: {}", app.new_tower_faction))
                .font_size(15.0)
                .draw();
            if ui.button("切換 Player/Enemy").secondary().draw() {
                app.new_tower_faction = if app.new_tower_faction == "Player" {
                    "Enemy".to_string()
                } else {
                    "Player".to_string()
                };
            }
            ui.spacer(4.0);
            if ui
                .button(if app.new_tower_is_base {
                    "IsBase: YES"
                } else {
                    "IsBase: no"
                })
                .secondary()
                .draw()
            {
                app.new_tower_is_base = !app.new_tower_is_base;
            }

            ui.spacer(16.0);
            ui.label("Creep 模板").font_size(18.0).height(28.0).draw();
            ui.spacer(4.0);
            for c in app.map.Creep.iter() {
                let lbl = c.Label.clone().unwrap_or_else(|| c.Name.clone());
                ui.label(&format!(
                    "{}  HP:{}  Msd:{}",
                    lbl, c.HP as i32, c.MoveSpeed as i32
                ))
                .font_size(14.0)
                .draw();
            }
        });
    });
}
