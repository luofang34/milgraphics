//! The screen tier's depth, for hosts that test it against their scene.
//!
//! A host that draws the screen tier into its own frame can hide what terrain or buildings
//! cover by testing each vertex against the frame's depth, as the frame tests its own
//! geometry, at no cost per vertex. Every point the library projects comes with the depth
//! the host gives for it ([`Projection::project_depth`](super::Projection::project_depth)),
//! and output points are copied from those projections bit for bit, so a plan's depths are
//! found by exact lookup once it is complete. Points the library places on screen itself
//! (cut ends, arrowheads, glyphs) take depth from the projected points beside them along
//! their line, or from the nearest projected point.

use std::collections::HashMap;

use super::{RenderPlan, ScreenPoint};

#[cfg(test)]
mod tests;

/// Side of the cells nearest projected points are searched in, in pixels.
const CELL_PX: f64 = 32.0;
/// Rings of cells searched around a point before it is left without depth.
const MAX_RINGS: i32 = 8;

/// Depths of the points projected for one plan.
#[derive(Default)]
pub(crate) struct DepthTable {
    exact: HashMap<(u64, u64), f32>,
    cells: HashMap<(i32, i32), Vec<(ScreenPoint, f32)>>,
}

impl DepthTable {
    pub(crate) fn record(&mut self, s: ScreenPoint, depth: f32) {
        if !(s.x.is_finite() && s.y.is_finite() && depth.is_finite()) {
            return;
        }
        self.exact.insert(key(s), depth);
        self.cells.entry(cell(s)).or_default().push((s, depth));
    }

    fn is_empty(&self) -> bool {
        self.exact.is_empty()
    }

    fn exact(&self, s: ScreenPoint) -> Option<f32> {
        self.exact.get(&key(s)).copied()
    }

    /// The depth of the projected point nearest `s`, searching outward ring by ring.
    fn nearest(&self, s: ScreenPoint) -> Option<f32> {
        let (cx, cy) = cell(s);
        for ring in 0..=MAX_RINGS {
            let mut best: Option<(f64, f32)> = None;
            for dx in -ring..=ring {
                for dy in -ring..=ring {
                    if dx.abs() != ring && dy.abs() != ring {
                        continue;
                    }
                    let Some(points) = self.cells.get(&(cx + dx, cy + dy)) else {
                        continue;
                    };
                    for &(p, d) in points {
                        let dist = (p.x - s.x).hypot(p.y - s.y);
                        if best.is_none_or(|(b, _)| dist < b) {
                            best = Some((dist, d));
                        }
                    }
                }
            }
            if let Some((_, d)) = best {
                return Some(d);
            }
        }
        None
    }

    /// Depths along a line of points: exact where projected, interpolated by length between
    /// projected neighbours, and the nearest projected point's beyond them.
    fn along(&self, points: &[ScreenPoint]) -> Vec<f32> {
        let known: Vec<Option<f32>> = points.iter().map(|&p| self.exact(p)).collect();
        let mut length = Vec::with_capacity(points.len());
        let mut walked = 0.0;
        for (i, p) in points.iter().enumerate() {
            if let Some(prev) = i.checked_sub(1).and_then(|j| points.get(j)) {
                walked += (p.x - prev.x).hypot(p.y - prev.y);
            }
            length.push(walked);
        }
        // The nearest projected point before and after each point, found in one pass each.
        let mut before = Vec::with_capacity(points.len());
        let mut last = None;
        for (i, k) in known.iter().enumerate() {
            if let Some(d) = k {
                last = Some((i, *d));
            }
            before.push(last);
        }
        let mut after = vec![None; points.len()];
        let mut next = None;
        for (i, k) in known.iter().enumerate().rev() {
            if let Some(d) = k {
                next = Some((i, *d));
            }
            if let Some(slot) = after.get_mut(i) {
                *slot = next;
            }
        }
        let at = |j: usize| length.get(j).copied().unwrap_or(0.0);
        let mut out = Vec::with_capacity(points.len());
        for (i, &p) in points.iter().enumerate() {
            let around = (
                before.get(i).copied().flatten(),
                after.get(i).copied().flatten(),
            );
            let depth = match around {
                (Some((a, da)), _) if a == i => Some(da),
                (Some((a, da)), Some((b, db))) => {
                    let span = at(b) - at(a);
                    let t = if span > 0.0 {
                        (at(i) - at(a)) / span
                    } else {
                        0.0
                    };
                    Some(da + (db - da) * t as f32)
                }
                _ => self.nearest(p),
            };
            out.push(depth.unwrap_or(f32::NAN));
        }
        out
    }
}

fn key(s: ScreenPoint) -> (u64, u64) {
    (s.x.to_bits(), s.y.to_bits())
}

fn cell(s: ScreenPoint) -> (i32, i32) {
    // Saturating: finite points far off screen share the outermost cells.
    (
        (s.x / CELL_PX).floor() as i32,
        (s.y / CELL_PX).floor() as i32,
    )
}

/// Gives every point of `plan` the depth `table` holds or implies; with an empty table, as
/// for a host that gives no depth, everything is left without.
pub(crate) fn fill(plan: &mut RenderPlan, table: &DepthTable) {
    if table.is_empty() {
        return;
    }
    for item in &mut plan.screen {
        item.depth = table.along(item.shape.points());
    }
    let at = |s: Option<ScreenPoint>| s.and_then(|s| table.exact(s).or_else(|| table.nearest(s)));
    for label in &mut plan.labels {
        label.depth = at(label.screen);
    }
    for handle in &mut plan.handles {
        handle.depth = at(handle.screen);
    }
    for symbol in &mut plan.symbols {
        // Its depth is its anchor's, which projects to its centre less its offset.
        let anchor = symbol.screen.map(|s| ScreenPoint {
            x: s.x - symbol.offset_px[0],
            y: s.y - symbol.offset_px[1],
        });
        symbol.depth = at(anchor);
    }
}
