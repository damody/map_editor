use eui::quick::ui::UI;
use eui::Rect;

use crate::app::{AppState, Selection};
use crate::style::{
    FS_BODY, FS_FIELD_LABEL, FS_FIELD_VALUE, FS_HEAD, H_FIELD, LH_FIELD_LABEL, LH_HEAD,
};

fn input_str(ui: &mut UI, label: &str, v: &mut String) -> bool {
    ui.input(label, v)
        .label_font_size(FS_FIELD_LABEL)
        .label_height(LH_FIELD_LABEL)
        .height(H_FIELD)
        .value_font_size(FS_FIELD_VALUE)
        .draw()
}

fn input_f32(ui: &mut UI, label: &str, v: &mut f32) -> bool {
    let mut s = format!("{:.2}", v);
    let changed = input_str(ui, label, &mut s);
    if changed {
        if let Ok(parsed) = s.trim().parse::<f32>() {
            *v = parsed;
            return true;
        }
    }
    false
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
            ui.label("Wave Inspector").font_size(FS_HEAD).height(LH_HEAD).draw();
            ui.spacer(6.0);

            match app.selection {
                Selection::Wave(w) => draw_wave(&mut ui, app, w),
                Selection::WaveDetail(w, d) => draw_detail(&mut ui, app, w, d),
                Selection::WaveSpawn(w, d, s) => draw_spawn(&mut ui, app, w, d, s),
                _ => {
                    ui.label("(請選中 Wave / Detail / Spawn)").font_size(FS_BODY).draw();
                }
            }
        });
    });
}

fn draw_wave(ui: &mut UI, app: &mut AppState, w: usize) {
    if w >= app.map.CreepWave.len() {
        return;
    }
    let mut name = app.map.CreepWave[w].Name.clone();
    let mut start_time = app.map.CreepWave[w].StartTime;

    if input_str(ui, "Name", &mut name) {
        app.begin_edit(None);
        app.map.CreepWave[w].Name = name;
        app.dirty = true;
    }
    if input_f32(ui, "StartTime", &mut start_time) {
        app.begin_edit(Some(&format!("wave_starttime_{}", w)));
        app.map.CreepWave[w].StartTime = start_time;
        app.dirty = true;
    }
}

fn draw_detail(ui: &mut UI, app: &mut AppState, w: usize, d: usize) {
    if w >= app.map.CreepWave.len() {
        return;
    }
    if d >= app.map.CreepWave[w].Detail.len() {
        return;
    }
    let mut path = app.map.CreepWave[w].Detail[d].Path.clone();
    if input_str(ui, "Path", &mut path) {
        app.begin_edit(None);
        app.map.CreepWave[w].Detail[d].Path = path;
        app.dirty = true;
    }
    let count = app.map.CreepWave[w].Detail[d].Creeps.len();
    ui.label(&format!("Spawns: {}", count)).font_size(FS_BODY).draw();
}

fn draw_spawn(ui: &mut UI, app: &mut AppState, w: usize, d: usize, s: usize) {
    if w >= app.map.CreepWave.len() {
        return;
    }
    if d >= app.map.CreepWave[w].Detail.len() {
        return;
    }
    if s >= app.map.CreepWave[w].Detail[d].Creeps.len() {
        return;
    }
    let mut time = app.map.CreepWave[w].Detail[d].Creeps[s].Time;
    let mut creep = app.map.CreepWave[w].Detail[d].Creeps[s].Creep.clone();
    if input_f32(ui, "Time (s)", &mut time) {
        app.begin_edit(Some(&format!("wave_spawn_time_{}_{}_{}", w, d, s)));
        app.map.CreepWave[w].Detail[d].Creeps[s].Time = time.max(0.0);
        app.dirty = true;
    }
    if input_str(ui, "Creep", &mut creep) {
        app.begin_edit(None);
        app.map.CreepWave[w].Detail[d].Creeps[s].Creep = creep;
        app.dirty = true;
    }
}
