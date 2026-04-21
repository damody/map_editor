use crate::app::Selection;
use crate::entity_schema::EntityConfig;
use crate::schema::CreepWaveData;

#[derive(Clone)]
pub struct Snapshot {
    pub map: CreepWaveData,
    pub entity: EntityConfig,
    pub selection: Selection,
}

pub struct UndoStack {
    past: Vec<Snapshot>,
    future: Vec<Snapshot>,
    limit: usize,
    current_tag: Option<String>,
}

impl UndoStack {
    pub fn new() -> Self {
        Self {
            past: Vec::new(),
            future: Vec::new(),
            limit: 100,
            current_tag: None,
        }
    }

    pub fn clear(&mut self) {
        self.past.clear();
        self.future.clear();
        self.current_tag = None;
    }

    pub fn end_group(&mut self) {
        self.current_tag = None;
    }

    pub fn push(&mut self, snap: Snapshot, tag: Option<&str>) {
        if let Some(t) = tag {
            if self.current_tag.as_deref() == Some(t) {
                return;
            }
            self.current_tag = Some(t.to_string());
        } else {
            self.current_tag = None;
        }
        self.past.push(snap);
        if self.past.len() > self.limit {
            self.past.remove(0);
        }
        self.future.clear();
    }

    pub fn undo(&mut self, current: Snapshot) -> Option<Snapshot> {
        self.current_tag = None;
        let snap = self.past.pop()?;
        self.future.push(current);
        Some(snap)
    }

    pub fn redo(&mut self, current: Snapshot) -> Option<Snapshot> {
        self.current_tag = None;
        let snap = self.future.pop()?;
        self.past.push(current);
        Some(snap)
    }
}

impl Default for UndoStack {
    fn default() -> Self {
        Self::new()
    }
}
