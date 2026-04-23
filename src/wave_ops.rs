//! Pure data mutation helpers for wave editing (testable without UI).
use crate::schema::{CreepWaveData, CreepWaveJD, DetailJD};

/// 單一 spawn drag：改某 spawn 的 Time（clamp to 0）
pub fn drag_spawn_time(wave: &mut CreepWaveJD, d: usize, s: usize, new_time: f32) {
    if let Some(detail) = wave.Detail.get_mut(d) {
        if let Some(spawn) = detail.Creeps.get_mut(s) {
            spawn.Time = new_time.max(0.0);
        }
    }
}

/// 新增一個 wave，預設名稱 W{N+1:02}，自動含一個用 first_path 的 Detail
pub fn add_wave(map: &mut CreepWaveData) -> usize {
    let n = map.CreepWave.len();
    let name = format!("W{:02}", n + 1);
    let path = map.Path.first().map(|p| p.Name.clone()).unwrap_or_default();
    map.CreepWave.push(CreepWaveJD {
        Name: name,
        StartTime: 0.0,
        Detail: vec![DetailJD {
            Path: path,
            Creeps: vec![],
        }],
    });
    n
}

/// 深拷貝指定 wave，名稱加 `_copy` 尾碼（碰撞遞增）
pub fn duplicate_wave(map: &mut CreepWaveData, idx: usize) -> Option<usize> {
    let src = map.CreepWave.get(idx)?.clone();
    let mut name = format!("{}_copy", src.Name);
    let mut k = 2;
    while map.CreepWave.iter().any(|w| w.Name == name) {
        name = format!("{}_copy{}", src.Name, k);
        k += 1;
    }
    let mut new = src;
    new.Name = name;
    let new_idx = map.CreepWave.len();
    map.CreepWave.push(new);
    Some(new_idx)
}

pub fn delete_wave(map: &mut CreepWaveData, idx: usize) -> bool {
    if idx < map.CreepWave.len() {
        map.CreepWave.remove(idx);
        true
    } else {
        false
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

#[cfg(test)]
mod tests_wave_crud {
    use super::*;
    use crate::schema::PathJD;

    fn empty_map_with_path() -> CreepWaveData {
        let mut m = CreepWaveData::default();
        m.Path.push(PathJD {
            Name: "p0".into(),
            Points: vec![],
        });
        m
    }

    #[test]
    fn add_wave_uses_first_path_and_increments_name() {
        let mut m = empty_map_with_path();
        let i = add_wave(&mut m);
        assert_eq!(i, 0);
        assert_eq!(m.CreepWave[0].Name, "W01");
        assert_eq!(m.CreepWave[0].Detail[0].Path, "p0");

        add_wave(&mut m);
        assert_eq!(m.CreepWave[1].Name, "W02");
    }

    #[test]
    fn duplicate_appends_copy_with_collision_handling() {
        let mut m = empty_map_with_path();
        add_wave(&mut m);
        let i1 = duplicate_wave(&mut m, 0).unwrap();
        assert_eq!(m.CreepWave[i1].Name, "W01_copy");
        let i2 = duplicate_wave(&mut m, 0).unwrap();
        assert_eq!(m.CreepWave[i2].Name, "W01_copy2");
    }

    #[test]
    fn delete_removes_and_returns_true() {
        let mut m = empty_map_with_path();
        add_wave(&mut m);
        assert!(delete_wave(&mut m, 0));
        assert!(m.CreepWave.is_empty());
        assert!(!delete_wave(&mut m, 0));
    }
}
