//! The version 16 Avenue of Approach, drawn as a supporting attack's axis:
//! "AA T" inside the axis where the axis puts its name, N ("ENY" unless
//! set) beside each boundary near the rear when the graphic is hostile,
//! and H beyond the upper boundary there. The axis carries no W or W1.

use super::{centred, font_px, text, unit};
use crate::engine::api::{Input, Output};

/// How far N sits forward of the rear end of a boundary and outward from
/// it, and how far beyond N H sits, in lines.
const FORWARD_LINES: f64 = 1.5;
const OUTWARD_LINES: f64 = 1.0;
const H_BEYOND_N_LINES: f64 = 1.2;

pub(super) fn labels(input: &Input<'_>, out: &mut Output) {
    out.labels.clear();
    let p: Vec<(f64, f64)> = input.pixels.iter().map(|p| (p.x, p.y)).collect();
    let n = p.len();
    // The axis's name spot: the middle of its first leg, or of its second
    // when it has more than one.
    let leg = if n >= 4 { (1, 2) } else { (0, 1) };
    if let (Some(a), Some(b)) = (p.get(leg.0), p.get(leg.1)) {
        let name = match text(&input.modifiers.designation) {
            Some(t) => format!("AA {t}"),
            None => "AA".to_owned(),
        };
        out.labels
            .push(centred(&name, ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0)));
    }
    let (Some(&rear), Some(&ahead)) = (
        n.checked_sub(2).and_then(|i| p.get(i)),
        n.checked_sub(3).and_then(|i| p.get(i)),
    ) else {
        return;
    };
    let Some(forward) = unit((ahead.0 - rear.0, ahead.1 - rear.1)) else {
        return;
    };
    let ends = rail_ends(out, rear, forward);
    let size = font_px();
    let spot = |end: (f64, f64), lines: f64| -> Option<(f64, f64)> {
        let outward = unit((end.0 - rear.0, end.1 - rear.1))?;
        Some((
            end.0 + size * (FORWARD_LINES * forward.0 + lines * outward.0),
            end.1 + size * (FORWARD_LINES * forward.1 + lines * outward.1),
        ))
    };
    let hostile = matches!(input.symbol.as_str().chars().nth(3), Some('5' | '6'));
    if hostile {
        let eny = text(&input.modifiers.hostile).unwrap_or("ENY");
        for end in ends.iter().flatten() {
            if let Some(at) = spot(*end, OUTWARD_LINES) {
                out.labels.push(centred(eny, at));
            }
        }
    }
    let upper = ends
        .iter()
        .flatten()
        .copied()
        .min_by(|a, b| a.1.total_cmp(&b.1));
    if let (Some(h), Some(end)) = (text(&input.modifiers.additional_info), upper) {
        if let Some(at) = spot(end, OUTWARD_LINES + H_BEYOND_N_LINES) {
            out.labels.push(centred(h, at));
        }
    }
}

/// The rear ends of the two boundaries: of the ends of the drawn lines, the
/// nearest to the rear point on each side of the centre line.
fn rail_ends(out: &Output, rear: (f64, f64), forward: (f64, f64)) -> [Option<(f64, f64)>; 2] {
    let mut best: [Option<((f64, f64), f64)>; 2] = [None, None];
    let ends = out
        .shapes
        .iter()
        .flat_map(|s| s.polylines())
        .flat_map(|l| [l.first().copied(), l.last().copied()])
        .flatten();
    for e in ends {
        let (dx, dy) = (e.0 - rear.0, e.1 - rear.1);
        let side = usize::from(forward.0 * dy - forward.1 * dx > 0.0);
        let d = dx.hypot(dy);
        if let Some(slot) = best.get_mut(side) {
            if slot.is_none_or(|(_, bd)| d < bd) {
                *slot = Some((e, d));
            }
        }
    }
    best.map(|b| b.map(|(e, _)| e))
}
