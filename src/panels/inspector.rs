use std::hash::{Hash, Hasher};

use eui::quick::ui::UI;
use eui::{Rect, TextAlign};

use crate::app::{AppState, Selection};
use crate::style::{
    FS_BODY, FS_BODY_SM, FS_FIELD_LABEL, FS_FIELD_VALUE, FS_HEAD, FS_LABEL, FS_SLIDER_VALUE,
    FS_SUBHEAD, H_FIELD, H_SLIDER_BAR_MAX, LH_FIELD_LABEL, LH_HEAD,
};

/// CheckPoint.Class 可選項目（第一項為預設）
const CHECKPOINT_CLASSES: &[&str] = &["Path", "Base", "Spawn", "Tower"];

/// 包裝 ui.input：套用本專案的放大 label / 行高 / 值字級
fn input_str(ui: &mut UI, label: &str, v: &mut String) -> bool {
    ui.input(label, v)
        .label_font_size(FS_FIELD_LABEL)
        .label_height(LH_FIELD_LABEL)
        .height(H_FIELD)
        .value_font_size(FS_FIELD_VALUE)
        .draw()
}

/// 下拉式選單字串欄位；不在清單內的原始值保留在 idx=0 的 fallback，使用者按下即切換。
fn combo_str(ui: &mut UI, label: &str, v: &mut String, items: &[&str]) -> bool {
    let lr = ui.content_rect();
    let y = ui.cursor_y();
    let r = Rect::new(lr.x, y, lr.w, H_FIELD);
    let muted = ui.theme().muted_text;
    let label_r = Rect::new(r.x, r.y, r.w, LH_FIELD_LABEL);
    ui.ctx()
        .paint_text(label_r, label, FS_FIELD_LABEL, muted, TextAlign::Left);

    let field_r = Rect::new(
        r.x,
        r.y + LH_FIELD_LABEL,
        r.w,
        H_FIELD - LH_FIELD_LABEL,
    );
    let mut idx = items.iter().position(|s| *s == v.as_str()).unwrap_or(0);
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    ("combo", label).hash(&mut hasher);
    let id = hasher.finish();
    let changed = ui.ctx().dropdown(id, field_r, items, &mut idx);
    if changed {
        *v = items[idx].to_string();
    }
    ui.ctx().advance_cursor(H_FIELD, 4.0);
    changed
}

pub fn draw(ui: &mut UI, rect: Rect, app: &mut AppState) {
    ui.scope(rect, |ctx| {
        let mut ui = UI::new(ctx);
        let panel_color = ui.theme().panel;
        let r = ui.content_rect();
        ui.paint_filled_rect(r, panel_color, 0.0);

        let inner = eui::quick::ui::inset(&r, 10.0, 10.0);
        ui.scope(inner, |ctx| {
            let mut ui = UI::new(ctx);
            ui.label("Inspector").font_size(FS_HEAD).height(LH_HEAD).draw();
            ui.spacer(6.0);

            match app.selection {
                Selection::None => {
                    ui.label("(未選中物件)").font_size(FS_BODY).draw();
                }
                Selection::Structure(i) => draw_structure(&mut ui, app, i),
                Selection::CheckPoint(i) => draw_checkpoint(&mut ui, app, i),
                Selection::BlockedRegion(i) => draw_blocked_region(&mut ui, app, i),
                Selection::BlockedRegionPoint(ri, pi) => {
                    draw_blocked_region_point(&mut ui, app, ri, pi)
                }
                Selection::TowerTemplate(i) => draw_tower_template(&mut ui, app, i),
                Selection::CreepTemplate(i) => draw_creep_template(&mut ui, app, i),
                Selection::Hero(i) => draw_hero(&mut ui, app, i),
                Selection::Enemy(i) => draw_enemy(&mut ui, app, i),
            }
        });
    });
}

/// f32 欄位 slider；回傳是否有變動
fn slider_f32(ui: &mut UI, label: &str, v: &mut f32, min: f32, max: f32) -> bool {
    let changed = ui
        .slider(label, v)
        .range(min, max)
        .label_font_size(FS_FIELD_LABEL)
        .label_height(LH_FIELD_LABEL)
        .height(H_FIELD)
        .value_font_size(FS_SLIDER_VALUE)
        .max_bar_height(H_SLIDER_BAR_MAX)
        .draw();
    if changed { *v = v.round(); }
    changed
}

/// Option<f32> 欄位：用 slider 調整值，0 → None
fn slider_opt_f32(ui: &mut UI, label: &str, v: &mut Option<f32>, min: f32, max: f32) -> bool {
    let mut val = v.unwrap_or(0.0);
    let changed = ui
        .slider(label, &mut val)
        .range(min, max)
        .label_font_size(FS_FIELD_LABEL)
        .label_height(LH_FIELD_LABEL)
        .height(H_FIELD)
        .value_font_size(FS_SLIDER_VALUE)
        .max_bar_height(H_SLIDER_BAR_MAX)
        .draw();
    if changed {
        val = val.round();
        *v = if val <= 0.0 { None } else { Some(val) };
    }
    changed
}

/// i32 欄位：透過 f32 slider 折射
fn slider_i32(ui: &mut UI, label: &str, v: &mut i32, min: i32, max: i32) -> bool {
    let mut f = *v as f32;
    let changed = ui
        .slider(label, &mut f)
        .range(min as f32, max as f32)
        .label_font_size(FS_FIELD_LABEL)
        .label_height(LH_FIELD_LABEL)
        .height(H_FIELD)
        .value_font_size(FS_SLIDER_VALUE)
        .max_bar_height(H_SLIDER_BAR_MAX)
        .draw();
    if changed { *v = f.round() as i32; }
    changed
}

fn draw_structure(ui: &mut UI, app: &mut AppState, i: usize) {
    if i >= app.map.Structures.len() { return; }
    let mut s = app.map.Structures[i].clone();
    ui.label(&format!("Structure #{}", i)).font_size(FS_SUBHEAD).draw();
    ui.spacer(4.0);

    let mut changed = false;
    if input_str(ui, "Tower (template)", &mut s.Tower) { changed = true; }
    if input_str(ui, "Faction", &mut s.Faction) { changed = true; }
    if slider_f32(ui, "X", &mut s.X, -3000.0, 3000.0) { changed = true; }
    if slider_f32(ui, "Y", &mut s.Y, -3000.0, 3000.0) { changed = true; }
    if slider_opt_f32(ui, "CollisionRadius (0=template)", &mut s.CollisionRadius, 0.0, 200.0) { changed = true; }

    ui.spacer(4.0);
    if ui.button(if s.IsBase { "IsBase: YES" } else { "IsBase: no" }).secondary().draw() {
        s.IsBase = !s.IsBase; changed = true;
    }
    if ui.button("切換 Player/Enemy").draw() {
        s.Faction = if s.Faction == "Player" { "Enemy".into() } else { "Player".into() };
        changed = true;
    }

    ui.spacer(12.0);
    let delete = ui.button("Delete").draw();
    if delete {
        app.begin_edit(None);
        app.map.Structures.remove(i);
        app.selection = Selection::None;
        app.dirty = true;
    } else if changed {
        app.begin_edit(Some(&format!("edit_struct_{}", i)));
        app.map.Structures[i] = s;
        app.dirty = true;
    }
}

fn draw_checkpoint(ui: &mut UI, app: &mut AppState, i: usize) {
    if i >= app.map.CheckPoint.len() { return; }
    let old_name = app.map.CheckPoint[i].Name.clone();
    let mut name = old_name.clone();
    let mut class = app.map.CheckPoint[i].Class.clone();
    let mut x = app.map.CheckPoint[i].X;
    let mut y = app.map.CheckPoint[i].Y;

    ui.label(&format!("CheckPoint #{}", i)).font_size(FS_SUBHEAD).draw();
    ui.spacer(4.0);

    let mut name_changed = false;
    let mut other_changed = false;
    if input_str(ui, "Name", &mut name) { name_changed = true; }
    if combo_str(ui, "Class", &mut class, CHECKPOINT_CLASSES) { other_changed = true; }
    if slider_f32(ui, "X", &mut x, -3000.0, 3000.0) { other_changed = true; }
    if slider_f32(ui, "Y", &mut y, -3000.0, 3000.0) { other_changed = true; }

    ui.spacer(12.0);
    let delete = ui.button("Delete").draw();
    if delete {
        app.begin_edit(None);
        let removed_name = app.map.CheckPoint[i].Name.clone();
        app.map.CheckPoint.remove(i);
        for path in app.map.Path.iter_mut() {
            path.Points.retain(|p| p != &removed_name);
        }
        app.selection = Selection::None;
        app.dirty = true;
    } else {
        if name_changed && name != old_name && !name.is_empty() {
            app.begin_edit(Some(&format!("edit_cp_{}", i)));
            app.map.CheckPoint[i].Name = name.clone();
            for path in app.map.Path.iter_mut() {
                for p in path.Points.iter_mut() {
                    if *p == old_name { *p = name.clone(); }
                }
            }
            app.dirty = true;
        }
        if other_changed {
            app.begin_edit(Some(&format!("edit_cp_{}", i)));
            let c = &mut app.map.CheckPoint[i];
            c.Class = class;
            c.X = x;
            c.Y = y;
            app.dirty = true;
        }
    }
}

fn draw_blocked_region(ui: &mut UI, app: &mut AppState, i: usize) {
    if i >= app.map.BlockedRegions.len() { return; }
    let mut name = app.map.BlockedRegions[i].Name.clone();
    let n_points = app.map.BlockedRegions[i].Points.len();
    ui.label(&format!("BlockedRegion #{}", i)).font_size(FS_SUBHEAD).draw();
    ui.spacer(4.0);
    if input_str(ui, "Name", &mut name) {
        app.begin_edit(Some(&format!("edit_br_{}_name", i)));
        app.map.BlockedRegions[i].Name = name;
        app.dirty = true;
    }
    ui.label(&format!("Points: {}", n_points)).font_size(FS_BODY_SM).draw();
    ui.spacer(12.0);
    if ui.button("Delete Region").draw() {
        app.begin_edit(None);
        app.map.BlockedRegions.remove(i);
        app.selection = Selection::None;
        app.dirty = true;
    }
}

fn draw_blocked_region_point(ui: &mut UI, app: &mut AppState, ri: usize, pi: usize) {
    let (mut x, mut y, points_len) = match app.map.BlockedRegions.get(ri) {
        Some(region) if pi < region.Points.len() => {
            (region.Points[pi].X, region.Points[pi].Y, region.Points.len())
        }
        _ => return,
    };
    ui.label(&format!("Region#{}.Point#{}", ri, pi)).font_size(FS_SUBHEAD).draw();
    ui.spacer(4.0);
    let mut changed = false;
    if slider_f32(ui, "X", &mut x, -3000.0, 3000.0) { changed = true; }
    if slider_f32(ui, "Y", &mut y, -3000.0, 3000.0) { changed = true; }
    if changed {
        app.begin_edit(Some(&format!("edit_br_pt_{}_{}", ri, pi)));
        let region = &mut app.map.BlockedRegions[ri];
        region.Points[pi].X = x;
        region.Points[pi].Y = y;
        app.dirty = true;
    }
    ui.spacer(12.0);
    let min_can_delete = points_len > 3;
    if min_can_delete && ui.button("Delete Point").draw() {
        app.begin_edit(None);
        app.map.BlockedRegions[ri].Points.remove(pi);
        app.selection = Selection::BlockedRegion(ri);
        app.dirty = true;
    } else if !min_can_delete {
        ui.label("(最少 3 點，無法刪除)").font_size(FS_LABEL).draw();
    }
}

fn draw_tower_template(ui: &mut UI, app: &mut AppState, i: usize) {
    if i >= app.map.Tower.len() { return; }
    let mut t = app.map.Tower[i].clone();
    ui.label(&format!("Tower Template #{}", i)).font_size(FS_SUBHEAD).draw();
    ui.spacer(4.0);
    let mut changed = false;
    if input_str(ui, "Name", &mut t.Name) { changed = true; }
    if slider_i32(ui, "Hp", &mut t.Property.Hp, 0, 20000) { changed = true; }
    if slider_i32(ui, "Block", &mut t.Property.Block, 0, 100) { changed = true; }
    if slider_f32(ui, "Attack.Range", &mut t.Attack.Range, 0.0, 3000.0) { changed = true; }
    if slider_f32(ui, "Attack.AttackSpeed", &mut t.Attack.AttackSpeed, 0.0, 10.0) { changed = true; }
    if slider_f32(ui, "Attack.Physic", &mut t.Attack.Physic, 0.0, 1000.0) { changed = true; }
    if slider_f32(ui, "Attack.Magic", &mut t.Attack.Magic, 0.0, 1000.0) { changed = true; }
    let mut ts = t.TurnSpeed.unwrap_or(0.0);
    if slider_f32(ui, "TurnSpeed (0=default 45)", &mut ts, 0.0, 360.0) {
        t.TurnSpeed = if ts <= 0.0 { None } else { Some(ts) };
        changed = true;
    }
    if slider_opt_f32(ui, "CollisionRadius (0=default 50)", &mut t.CollisionRadius, 0.0, 300.0) {
        changed = true;
    }
    if changed {
        app.begin_edit(Some(&format!("edit_tower_tmpl_{}", i)));
        app.map.Tower[i] = t;
        app.dirty = true;
    }
}

fn draw_creep_template(ui: &mut UI, app: &mut AppState, i: usize) {
    if i >= app.map.Creep.len() { return; }
    let mut c = app.map.Creep[i].clone();
    ui.label(&format!("Creep Template #{}", i)).font_size(FS_SUBHEAD).draw();
    ui.spacer(4.0);
    let mut changed = false;
    if input_str(ui, "Name", &mut c.Name) { changed = true; }
    let mut label = c.Label.clone().unwrap_or_default();
    if input_str(ui, "Label (可選)", &mut label) {
        c.Label = if label.is_empty() { None } else { Some(label) };
        changed = true;
    }
    if slider_f32(ui, "HP", &mut c.HP, 0.0, 20000.0) { changed = true; }
    if slider_f32(ui, "DefendPhysic", &mut c.DefendPhysic, 0.0, 500.0) { changed = true; }
    if slider_f32(ui, "DefendMagic", &mut c.DefendMagic, 0.0, 500.0) { changed = true; }
    if slider_f32(ui, "MoveSpeed", &mut c.MoveSpeed, 0.0, 1000.0) { changed = true; }
    let mut faction = c.Faction.clone().unwrap_or_default();
    if input_str(ui, "Faction (Player/Enemy)", &mut faction) {
        c.Faction = if faction.is_empty() { None } else { Some(faction) };
        changed = true;
    }
    let mut ts = c.TurnSpeed.unwrap_or(0.0);
    if slider_f32(ui, "TurnSpeed (0=default 90)", &mut ts, 0.0, 360.0) {
        c.TurnSpeed = if ts <= 0.0 { None } else { Some(ts) };
        changed = true;
    }
    if slider_opt_f32(ui, "CollisionRadius (0=default 20)", &mut c.CollisionRadius, 0.0, 200.0) {
        changed = true;
    }
    if changed {
        app.begin_edit(Some(&format!("edit_creep_tmpl_{}", i)));
        app.map.Creep[i] = c;
        app.dirty = true;
    }
}

fn draw_hero(ui: &mut UI, app: &mut AppState, i: usize) {
    if i >= app.entity.heroes.len() { return; }
    let mut h = app.entity.heroes[i].clone();
    ui.label(&format!("Hero #{}  ({})", i, h.id)).font_size(FS_SUBHEAD).draw();
    ui.spacer(4.0);
    let mut changed = false;
    if input_str(ui, "id", &mut h.id) { changed = true; }
    if input_str(ui, "name", &mut h.name) { changed = true; }
    if input_str(ui, "primary_attribute", &mut h.primary_attribute) { changed = true; }
    if slider_i32(ui, "strength", &mut h.strength, 0, 200) { changed = true; }
    if slider_i32(ui, "agility", &mut h.agility, 0, 200) { changed = true; }
    if slider_i32(ui, "intelligence", &mut h.intelligence, 0, 200) { changed = true; }
    if slider_f32(ui, "attack_range", &mut h.attack_range, 0.0, 3000.0) { changed = true; }
    if slider_i32(ui, "base_damage", &mut h.base_damage, 0, 1000) { changed = true; }
    if slider_f32(ui, "base_armor", &mut h.base_armor, 0.0, 500.0) { changed = true; }
    if slider_i32(ui, "base_hp", &mut h.base_hp, 0, 20000) { changed = true; }
    if slider_i32(ui, "base_mana", &mut h.base_mana, 0, 20000) { changed = true; }
    if slider_f32(ui, "move_speed", &mut h.move_speed, 0.0, 1000.0) { changed = true; }
    let mut ts = h.turn_speed.unwrap_or(0.0);
    if slider_f32(ui, "turn_speed (0=default 180)", &mut ts, 0.0, 720.0) {
        h.turn_speed = if ts <= 0.0 { None } else { Some(ts) };
        changed = true;
    }
    if slider_opt_f32(ui, "collision_radius (0=default 30)", &mut h.collision_radius, 0.0, 200.0) {
        changed = true;
    }
    if changed {
        app.begin_edit(Some(&format!("edit_hero_{}", i)));
        app.entity.heroes[i] = h;
        app.entity_dirty = true;
    }
}

fn draw_enemy(ui: &mut UI, app: &mut AppState, i: usize) {
    if i >= app.entity.enemies.len() { return; }
    let mut e = app.entity.enemies[i].clone();
    ui.label(&format!("Enemy #{}  ({})", i, e.id)).font_size(FS_SUBHEAD).draw();
    ui.spacer(4.0);
    let mut changed = false;
    if input_str(ui, "id", &mut e.id) { changed = true; }
    if input_str(ui, "name", &mut e.name) { changed = true; }
    if input_str(ui, "enemy_type", &mut e.enemy_type) { changed = true; }
    if slider_i32(ui, "hp", &mut e.hp, 0, 20000) { changed = true; }
    if slider_f32(ui, "armor", &mut e.armor, 0.0, 500.0) { changed = true; }
    if slider_f32(ui, "magic_resistance", &mut e.magic_resistance, 0.0, 500.0) { changed = true; }
    if slider_i32(ui, "damage", &mut e.damage, 0, 1000) { changed = true; }
    if slider_f32(ui, "attack_range", &mut e.attack_range, 0.0, 3000.0) { changed = true; }
    if slider_f32(ui, "move_speed", &mut e.move_speed, 0.0, 1000.0) { changed = true; }
    if input_str(ui, "ai_type", &mut e.ai_type) { changed = true; }
    if slider_i32(ui, "exp_reward", &mut e.exp_reward, 0, 10000) { changed = true; }
    if slider_i32(ui, "gold_reward", &mut e.gold_reward, 0, 10000) { changed = true; }
    if slider_opt_f32(ui, "collision_radius (0=default 25)", &mut e.collision_radius, 0.0, 200.0) {
        changed = true;
    }
    if changed {
        app.begin_edit(Some(&format!("edit_enemy_{}", i)));
        app.entity.enemies[i] = e;
        app.entity_dirty = true;
    }
}
