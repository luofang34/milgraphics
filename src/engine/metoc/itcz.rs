//! The Inter-Tropical Convergence Zone as MIL-STD-2525E change 1 draws it
//! (45 110407, TABLE M-II): two rails along the line joined by rungs in
//! alternating groups of two and three, a ladder. Upstream draws the line
//! alone.

use crate::engine::base::{Pt, Shape, shape_type};

/// Distance of each rail from the line, in pixels at the default width.
const HALF_WIDTH: f64 = 8.0;
/// Distance between the rungs of a group.
const RUNG_SPACING: f64 = 6.0;
/// Distance between groups.
const GROUP_GAP: f64 = 30.0;
/// Most rungs one line draws, so a line zoomed far in stays cheap.
const MAX_RUNGS: usize = 5000;
/// Longest mitre, in rail offsets, before a sharp corner is cut short.
const MITRE_LIMIT: f64 = 3.0;

/// The rails and rungs along `pixels`; sizes scale with `scale`.
pub(crate) fn shapes(pixels: &[Pt], scale: f64) -> Vec<Shape> {
    let half = HALF_WIDTH * scale;
    let mut out = Vec::new();
    for side in [1.0, -1.0] {
        let rail = offset(pixels, side * half);
        let mut shape = Shape::new(shape_type::POLYLINE);
        let mut points = rail.into_iter();
        if let Some(first) = points.next() {
            shape.move_to(first);
            points.for_each(|p| shape.line_to(p));
            out.push(shape);
        }
    }
    let mut rungs = Shape::new(shape_type::POLYLINE);
    for (at, normal) in rung_positions(pixels, scale) {
        rungs.move_to(Pt::new(at.x + normal.0 * half, at.y + normal.1 * half));
        rungs.line_to(Pt::new(at.x - normal.0 * half, at.y - normal.1 * half));
    }
    if !rungs.path.is_empty() {
        out.push(rungs);
    }
    out
}

/// Unit normal of `a`→`b` (to its left on a y-down screen), if it has a
/// length.
fn normal(a: Pt, b: Pt) -> Option<(f64, f64)> {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let len = dx.hypot(dy);
    (len > 1e-9).then(|| (dy / len, -dx / len))
}

/// `pixels` moved `distance` along their normals, mitred at the corners.
fn offset(pixels: &[Pt], distance: f64) -> Vec<Pt> {
    let normals: Vec<Option<(f64, f64)>> = pixels
        .iter()
        .zip(pixels.iter().skip(1))
        .map(|(&a, &b)| normal(a, b))
        .collect();
    pixels
        .iter()
        .enumerate()
        .filter_map(|(i, p)| {
            let before = i
                .checked_sub(1)
                .and_then(|j| normals.get(j).copied().flatten());
            let after = normals.get(i).copied().flatten();
            let n = match (before, after) {
                (Some(a), Some(b)) => {
                    let (sx, sy) = (a.0 + b.0, a.1 + b.1);
                    let dot = a.0 * b.0 + a.1 * b.1;
                    // The mitre is 1 / cos(half the turn) long.
                    let k = (2.0 / (1.0 + dot)).sqrt().min(MITRE_LIMIT);
                    let len = sx.hypot(sy);
                    if len > 1e-9 {
                        (sx / len * k, sy / len * k)
                    } else {
                        a
                    }
                }
                (Some(n), None) | (None, Some(n)) => n,
                (None, None) => return None,
            };
            Some(Pt::new(p.x + n.0 * distance, p.y + n.1 * distance))
        })
        .collect()
}

/// Where the rungs cross the line, with the line's normal there: groups of
/// two and three in turn, a group gap apart, starting half a gap in.
fn rung_positions(pixels: &[Pt], scale: f64) -> Vec<(Pt, (f64, f64))> {
    let (spacing, gap) = (RUNG_SPACING * scale, GROUP_GAP * scale);
    if !(spacing.is_finite() && spacing > 0.5) {
        return Vec::new();
    }
    let mut out = Vec::new();
    // Distance along the line of the next rung, and of the walk so far.
    let mut next = gap / 2.0;
    let mut walked = 0.0;
    let (mut group, mut in_group) = (2usize, 0usize);
    for (&a, &b) in pixels.iter().zip(pixels.iter().skip(1)) {
        let Some(n) = normal(a, b) else {
            continue;
        };
        let len = (b.x - a.x).hypot(b.y - a.y);
        while next <= walked + len && out.len() < MAX_RUNGS {
            let t = (next - walked) / len;
            out.push((Pt::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t), n));
            in_group += 1;
            if in_group == group {
                (group, in_group) = (5 - group, 0);
                next += gap;
            } else {
                next += spacing;
            }
        }
        walked += len;
    }
    out
}

#[cfg(test)]
mod tests;
