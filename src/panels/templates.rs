use eui::quick::ui::UI;
use eui::{ButtonStyle, Rect};

use crate::app::{AppState, Selection, ViewMode};

pub fn draw(ui: &mut UI, rect: Rect, app: &mut AppState) {
    ui.scope(rect, |ctx| {
        let mut ui = UI::new(ctx);
        let panel_color = ui.theme().panel;
        let r = ui.content_rect();
        ui.paint_filled_rect(r, panel_color, 0.0);

        let inner = eui::quick::ui::inset(&r, 10.0, 10.0);
        ui.scope(inner, |ctx| {
            let mut ui = UI::new(ctx);
            match app.view_mode {
                ViewMode::Map => draw_map_mode(&mut ui, app),
                ViewMode::Entities => draw_entities_mode(&mut ui, app),
            }
        });
    });
}

fn draw_map_mode(ui: &mut UI, app: &mut AppState) {
    ui.label("Tower 模板").font_size(18.0).height(28.0).draw();
    ui.spacer(4.0);

    // Tower templates — 雙擊 (shift) 進 inspector；普通點擊 → 設為「新增塔」模板
    let towers = app.map.Tower.clone();
    let mut selected_template = app.new_tower_template.clone();
    for (i, t) in towers.iter().enumerate() {
        let is_set_as_new = selected_template == t.Name;
        let sel_for_inspector = matches!(app.selection, Selection::TowerTemplate(idx) if idx == i);
        let style = if sel_for_inspector {
            ButtonStyle::Secondary
        } else if is_set_as_new {
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
            app.selection = Selection::TowerTemplate(i);
        }
        ui.spacer(2.0);
    }
    app.new_tower_template = selected_template;

    ui.spacer(8.0);
    ui.label(&format!("Faction: {}", app.new_tower_faction))
        .font_size(15.0)
        .draw();
    if ui.button("切換 Player/Enemy").secondary().draw() {
        app.new_tower_faction = if app.new_tower_faction == "Player" {
            "Enemy".into()
        } else {
            "Player".into()
        };
    }
    ui.spacer(4.0);
    if ui
        .button(if app.new_tower_is_base { "IsBase: YES" } else { "IsBase: no" })
        .secondary()
        .draw()
    {
        app.new_tower_is_base = !app.new_tower_is_base;
    }

    ui.spacer(16.0);
    ui.label("Creep 模板").font_size(18.0).height(28.0).draw();
    ui.spacer(4.0);
    let creeps = app.map.Creep.clone();
    for (i, c) in creeps.iter().enumerate() {
        let sel = matches!(app.selection, Selection::CreepTemplate(idx) if idx == i);
        let style = if sel { ButtonStyle::Primary } else { ButtonStyle::Ghost };
        let lbl = c.Label.clone().unwrap_or_else(|| c.Name.clone());
        if ui
            .button(&format!("{}  HP:{}  Msd:{}", lbl, c.HP as i32, c.MoveSpeed as i32))
            .style(style)
            .draw()
        {
            app.selection = Selection::CreepTemplate(i);
        }
        ui.spacer(2.0);
    }

    // BlockedRegions 列表：方便跨多邊形管理
    ui.spacer(16.0);
    ui.label(&format!("BlockedRegions ({})", app.map.BlockedRegions.len()))
        .font_size(18.0)
        .height(28.0)
        .draw();
    let regions = app.map.BlockedRegions.clone();
    for (i, r) in regions.iter().enumerate() {
        let sel = matches!(app.selection, Selection::BlockedRegion(idx) if idx == i)
            || matches!(app.selection, Selection::BlockedRegionPoint(ri, _) if ri == i);
        let style = if sel { ButtonStyle::Primary } else { ButtonStyle::Ghost };
        if ui
            .button(&format!("{}  ({} pts)", r.Name, r.Points.len()))
            .style(style)
            .draw()
        {
            app.selection = Selection::BlockedRegion(i);
        }
        ui.spacer(2.0);
    }
}

fn draw_entities_mode(ui: &mut UI, app: &mut AppState) {
    ui.label("Heroes (entity.json)").font_size(18.0).height(28.0).draw();
    ui.spacer(4.0);
    let heroes = app.entity.heroes.clone();
    for (i, h) in heroes.iter().enumerate() {
        let sel = matches!(app.selection, Selection::Hero(idx) if idx == i);
        let style = if sel { ButtonStyle::Primary } else { ButtonStyle::Ghost };
        if ui
            .button(&format!("{}  ({})", h.name, h.id))
            .style(style)
            .draw()
        {
            app.selection = Selection::Hero(i);
        }
        ui.spacer(2.0);
    }

    ui.spacer(16.0);
    ui.label("Enemies (entity.json)").font_size(18.0).height(28.0).draw();
    ui.spacer(4.0);
    let enemies = app.entity.enemies.clone();
    for (i, e) in enemies.iter().enumerate() {
        let sel = matches!(app.selection, Selection::Enemy(idx) if idx == i);
        let style = if sel { ButtonStyle::Primary } else { ButtonStyle::Ghost };
        if ui
            .button(&format!("{}  HP:{}  Dmg:{}", e.name, e.hp, e.damage))
            .style(style)
            .draw()
        {
            app.selection = Selection::Enemy(i);
        }
        ui.spacer(2.0);
    }

    ui.spacer(16.0);
    if let Some(ref p) = app.entity_path {
        ui.label(&format!("Loaded: {}", p.display()))
            .font_size(12.0)
            .draw();
    } else {
        ui.label("(未載入 entity.json)").font_size(13.0).draw();
    }
}
