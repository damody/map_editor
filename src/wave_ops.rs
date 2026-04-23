//! Pure data mutation helpers for wave editing (testable without UI).
use crate::schema::CreepWaveJD;

/// 單一 spawn drag：改某 spawn 的 Time（clamp to 0）
pub fn drag_spawn_time(wave: &mut CreepWaveJD, d: usize, s: usize, new_time: f32) {
    if let Some(detail) = wave.Detail.get_mut(d) {
        if let Some(spawn) = detail.Creeps.get_mut(s) {
            spawn.Time = new_time.max(0.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{CreepsJD, DetailJD};

    fn build_wave(times: &[f32]) -> CreepWaveJD {
        CreepWaveJD {
            Name: "W".into(),
            StartTime: 0.0,
            Detail: vec![DetailJD {
                Path: "p".into(),
                Creeps: times
                    .iter()
                    .map(|t| CreepsJD {
                        Time: *t,
                        Creep: "c".into(),
                    })
                    .collect(),
            }],
        }
    }

    #[test]
    fn drag_single_changes_only_target() {
        let mut w = build_wave(&[0.0, 1.0, 2.0]);
        drag_spawn_time(&mut w, 0, 1, 1.5);
        assert_eq!(w.Detail[0].Creeps[0].Time, 0.0);
        assert_eq!(w.Detail[0].Creeps[1].Time, 1.5);
        assert_eq!(w.Detail[0].Creeps[2].Time, 2.0);
    }

    #[test]
    fn drag_clamps_negative_to_zero() {
        let mut w = build_wave(&[1.0]);
        drag_spawn_time(&mut w, 0, 0, -5.0);
        assert_eq!(w.Detail[0].Creeps[0].Time, 0.0);
    }
}
