//! Wire, unspecified: X marks along the line with clear space between them,
//! and no line.

use super::set_lines;
use crate::engine::api::{Input, Output};

/// Half the width of an X, in pixels, as upstream draws it.
const HALF: f64 = 7.0;
/// Distance between the centres of neighbouring X marks: the template leaves
/// about seven tenths of a mark's width between them.
pub(super) const PITCH: f64 = 24.0;
/// Most marks drawn; a longer line spaces its marks further apart so they
/// still run its whole length within the vertex budget.
pub(super) const MAX_MARKS: usize = 2_000;

/// Replaces upstream's touching marks with spaced ones along the control
/// points, each turned with its segment.
pub(super) fn spread(input: &Input<'_>, out: &mut Output) {
    let Some(shape) = out.shapes.first_mut() else {
        return;
    };
    let total: f64 = input
        .pixels
        .windows(2)
        .filter_map(|w| match w {
            [a, b] => Some((b.x - a.x).hypot(b.y - a.y)),
            _ => None,
        })
        .sum();
    let pitch = PITCH.max(total / MAX_MARKS as f64);
    let mut marks = Vec::new();
    // Distance along the line to the next mark's centre.
    let mut next = pitch / 2.0;
    let mut walked = 0.0;
    for pair in input.pixels.windows(2) {
        let [a, b] = pair else { continue };
        let (dx, dy) = (b.x - a.x, b.y - a.y);
        let len = dx.hypot(dy);
        let Some(u) = super::unit((dx, dy)) else {
            continue;
        };
        let n = (-u.1, u.0);
        while next <= walked + len && marks.len() < 2 * MAX_MARKS {
            let t = next - walked;
            let c = (a.x + u.0 * t, a.y + u.1 * t);
            for (p, q) in [(u, n), (u, (-n.0, -n.1))] {
                let d = ((p.0 + q.0) * HALF, (p.1 + q.1) * HALF);
                marks.push(vec![(c.0 - d.0, c.1 - d.1), (c.0 + d.0, c.1 + d.1)]);
            }
            next += pitch;
        }
        walked += len;
    }
    if !marks.is_empty() {
        set_lines(shape, &marks);
    }
}
