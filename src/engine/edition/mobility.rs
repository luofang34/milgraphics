//! The version 16 Mobility Corridor: a line with a fork opening outward at
//! each end. Every segment between two points repeats the information of
//! the first: B (the echelon) set into a break in the middle of the
//! segment, and H above it.

use super::{font_px, lines_like, rotated, text, unit, upright_deg};
use crate::engine::api::{Input, Output};

/// Length of each prong of a fork, in pixels, and its angle from the
/// line's direction.
const PRONG: f64 = 30.0;
const PRONG_ANGLE_DEG: f64 = 35.0;
/// Distance from the line to the middle of H, in lines.
const H_ABOVE_LINES: f64 = 1.4;

pub(super) fn corridor(input: &Input<'_>, out: &mut Output) {
    out.labels.clear();
    let p: Vec<(f64, f64)> = input.pixels.iter().map(|p| (p.x, p.y)).collect();
    let size = font_px();
    let echelon = text(&input.modifiers.echelon);
    let h = text(&input.modifiers.additional_info);
    for pair in p.windows(2) {
        let [a, b] = pair else { continue };
        let Some(d) = unit((b.0 - a.0, b.1 - a.1)) else {
            continue;
        };
        let mid = ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
        let angle = upright_deg(d);
        if let Some(echelon) = echelon {
            out.labels.push(rotated(echelon, mid, angle, true));
        }
        if let Some(h) = h {
            // Above B as the text reads.
            let turn = angle.to_radians();
            let up = (turn.sin(), -turn.cos());
            let off = H_ABOVE_LINES * size;
            let at = (mid.0 + off * up.0, mid.1 + off * up.1);
            out.labels.push(rotated(h, at, angle, false));
        }
    }
    let ends = [
        (p.first(), p.get(1)),
        (p.last(), p.len().checked_sub(2).and_then(|i| p.get(i))),
    ];
    let mut prongs = Vec::new();
    for (end, inner) in ends {
        let (Some(&e), Some(&i)) = (end, inner) else {
            continue;
        };
        let Some(o) = unit((e.0 - i.0, e.1 - i.1)) else {
            continue;
        };
        for sign in [-1.0, 1.0] {
            let a = sign * PRONG_ANGLE_DEG.to_radians();
            let dir = (o.0 * a.cos() - o.1 * a.sin(), o.0 * a.sin() + o.1 * a.cos());
            prongs.push(vec![e, (e.0 + PRONG * dir.0, e.1 + PRONG * dir.1)]);
        }
    }
    if let Some(line) = out.shapes.first() {
        if !prongs.is_empty() {
            let shape = lines_like(line, &prongs);
            out.shapes.push(shape);
        }
    }
}
