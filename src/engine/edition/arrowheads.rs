//! Arrowheads drawn open where upstream fills them, and the reverse.

use super::lines_like;
use crate::engine::api::{Input, Output};
use crate::engine::base::shape_type;

/// Fix: upstream fills the arrowhead as a triangle of its own; the template
/// draws two strokes meeting at the tip, styled like the zigzag line.
pub(super) fn open_fix(out: &mut Output) {
    let Some(line) = out
        .shapes
        .iter()
        .find(|s| s.shape_type == shape_type::POLYLINE)
        .cloned()
    else {
        return;
    };
    for shape in &mut out.shapes {
        if shape.shape_type != shape_type::FILL {
            continue;
        }
        let rings = shape.polylines();
        // The triangle: wing, tip, wing, back to the first wing.
        let [ring] = rings.as_slice() else {
            continue;
        };
        if let [w1, tip, w2, ..] = ring.as_slice() {
            *shape = lines_like(&line, &[vec![*w1, *tip, *w2]]);
        }
    }
}

/// Control: upstream strokes the two arrowheads at the opening; the version
/// 16 template fills them. They are the shape's three-point lines after the
/// circle.
pub(super) fn fill_control(out: &mut Output) {
    let Some(shape) = out.shapes.first_mut() else {
        return;
    };
    let lines = shape.polylines();
    let (arc, heads): (Vec<_>, Vec<_>) = lines.into_iter().partition(|l| l.len() != 3);
    if heads.len() != 2 || arc.len() != 1 {
        return;
    }
    let like = shape.clone();
    *shape = lines_like(&like, &arc);
    for head in heads {
        let mut ring = head;
        if let Some(&first) = ring.first() {
            ring.push(first);
        }
        let mut filled = lines_like(&like, &[ring]);
        filled.fill_color = like.line_color;
        out.shapes.push(filled);
    }
}

/// Distance from the circle to the centre of the "C", in pixels: half the
/// label font's height and a gap, so the letter sits outside the line.
const C_CLEARANCE: f64 = 8.0;

/// Control: the template sets the "C" outside the circle; upstream centres
/// it on the line. It keeps its bearing from the centre, point 1.
pub(super) fn control_label_outside(input: &Input<'_>, out: &mut Output) {
    let (Some(centre), Some(rim)) = (input.pixels.first(), input.pixels.get(1)) else {
        return;
    };
    let radius = (rim.x - centre.x).hypot(rim.y - centre.y);
    for label in out.labels.iter_mut().filter(|l| l.text == "C") {
        let Some(u) = super::unit((label.x - centre.x, label.y - centre.y)) else {
            continue;
        };
        label.x = centre.x + u.0 * (radius + C_CLEARANCE);
        label.y = centre.y + u.1 * (radius + C_CLEARANCE);
    }
}
