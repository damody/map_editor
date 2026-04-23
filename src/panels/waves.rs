use eui::quick::ui::UI;
use eui::Rect;

use crate::app::AppState;
use crate::style::WAVE_LIST_W;
use crate::panels::{wave_list, wave_timeline, wave_inspector};

/// Waves 模式三欄分派：左 wave 列表｜中 timeline｜右 inspector
pub fn draw(ui: &mut UI, rect: Rect, app: &mut AppState) {
    let list_w = WAVE_LIST_W;
    let inspector_w = app.inspector_w.max(crate::style::INSPECTOR_MIN_W);
    let timeline_w = (rect.w - list_w - inspector_w).max(100.0);

    let list_rect = Rect::new(rect.x, rect.y, list_w, rect.h);
    let timeline_rect = Rect::new(rect.x + list_w, rect.y, timeline_w, rect.h);
    let inspector_rect = Rect::new(rect.x + list_w + timeline_w, rect.y, inspector_w, rect.h);

    wave_list::draw(ui, list_rect, app);
    wave_timeline::draw(ui, timeline_rect, app);
    wave_inspector::draw(ui, inspector_rect, app);
}
