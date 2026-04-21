//! 2D 幾何工具。與 `omb/src/util/geometry.rs` 演算法完全一致，以保證
//! 編輯器預覽的阻擋判定與遊戲執行時結果相同。

/// Ray-casting：點是否在多邊形（凹/凸皆可）內部。
pub fn point_in_polygon(px: f32, py: f32, poly: &[(f32, f32)]) -> bool {
    if poly.len() < 3 {
        return false;
    }
    let mut inside = false;
    let n = poly.len();
    let mut j = n - 1;
    for i in 0..n {
        let (ix, iy) = poly[i];
        let (jx, jy) = poly[j];
        let cond = (iy > py) != (jy > py)
            && px < (jx - ix) * (py - iy) / (jy - iy + f32::EPSILON) + ix;
        if cond {
            inside = !inside;
        }
        j = i;
    }
    inside
}

/// 點 (px,py) 到線段 (ax,ay)-(bx,by) 的最短距離。
pub fn point_segment_dist(px: f32, py: f32, ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let dx = bx - ax;
    let dy = by - ay;
    let len2 = dx * dx + dy * dy;
    if len2 < 1e-8 {
        return ((px - ax).powi(2) + (py - ay).powi(2)).sqrt();
    }
    let t = (((px - ax) * dx + (py - ay) * dy) / len2).clamp(0.0, 1.0);
    let cx = ax + t * dx;
    let cy = ay + t * dy;
    ((px - cx).powi(2) + (py - cy).powi(2)).sqrt()
}
