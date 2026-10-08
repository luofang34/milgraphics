//! The version 16 PsyOps Zones: a solid loudspeaker inside the zone with H
//! over T to its right, in place of the kill box label the line type
//! carries. W - W1 stays where the line type puts it, outside the upper
//! left of the zone.

use super::{bounds_centre, font_px, text};
use crate::engine::api::{Input, Justify, Label, Output};
use crate::engine::base::{Pt, Shape, shape_type};

/// The label upstream's kill box puts at the centre, above T.
const KILL_BOX_LABEL: &str = "PKB";
/// The loudspeaker, in pixels with its middle left at the origin, y down:
/// a block, a horn opening to the right, and four sound bars beyond it.
const BLOCK: [(f64, f64); 4] = [(0.0, -5.0), (6.0, -5.0), (6.0, 5.0), (0.0, 5.0)];
const HORN: [(f64, f64); 4] = [(6.0, -5.0), (14.0, -12.0), (14.0, 12.0), (6.0, 5.0)];
const BARS_Y: [f64; 4] = [-9.0, -3.0, 3.0, 9.0];
const BAR_X: (f64, f64) = (15.5, 21.0);
const BAR_HALF_HEIGHT: f64 = 1.0;
/// Width of the loudspeaker and the gap between it and the text.
const SPEAKER_WIDTH: f64 = 21.0;
const GAP: f64 = 6.0;

pub(super) fn speaker(input: &Input<'_>, out: &mut Output) {
    let Some(centre) = centre(&input.pixels) else {
        return;
    };
    let t = text(&input.modifiers.designation);
    if let Some(i) = out.labels.iter().position(|l| l.text == KILL_BOX_LABEL) {
        out.labels.remove(i);
        // The group text puts T on the line after the label.
        if out
            .labels
            .get(i)
            .is_some_and(|l| Some(l.text.as_str()) == t)
        {
            out.labels.remove(i);
        }
    }
    let lines: Vec<&str> = [text(&input.modifiers.additional_info), t]
        .into_iter()
        .flatten()
        .collect();
    let widest = lines
        .iter()
        .map(|l| (input.text_width)(l))
        .fold(0.0, f64::max);
    let left = centre.0 - (SPEAKER_WIDTH + GAP + widest) / 2.0;
    let size = font_px();
    let first = centre.1 - size * (lines.len().saturating_sub(1) as f64) / 2.0;
    for (k, line) in lines.iter().enumerate() {
        out.labels.push(Label {
            text: (*line).to_owned(),
            x: left + SPEAKER_WIDTH + GAP,
            y: first + size * k as f64 + 0.3 * size,
            angle_deg: 0.0,
            justify: Justify::Left,
            knockout: false,
        });
    }
    let color = out.shapes.first().and_then(|s| s.line_color);
    out.shapes.push(glyph((left, centre.1), color));
}

/// The middle of the zone: the circle's centre, the midpoint of the
/// rectangle's two points, or the centre of the area's bounds.
fn centre(points: &[Pt]) -> Option<(f64, f64)> {
    match points {
        [c] => Some((c.x, c.y)),
        [a, b] => Some(((a.x + b.x) / 2.0, (a.y + b.y) / 2.0)),
        _ => bounds_centre(points),
    }
}

/// The filled loudspeaker with its middle left at `at`.
fn glyph(at: (f64, f64), color: Option<crate::style::Rgba>) -> Shape {
    let mut shape = Shape::new(shape_type::FILL);
    shape.fill_color = color;
    shape.line_color = color;
    let bars = BARS_Y.map(|y| {
        [
            (BAR_X.0, y - BAR_HALF_HEIGHT),
            (BAR_X.1, y - BAR_HALF_HEIGHT),
            (BAR_X.1, y + BAR_HALF_HEIGHT),
            (BAR_X.0, y + BAR_HALF_HEIGHT),
        ]
    });
    let parts = [BLOCK, HORN].into_iter().chain(bars);
    for part in parts {
        let mut points = part.iter().map(|&(x, y)| Pt::new(at.0 + x, at.1 + y));
        if let Some(p) = points.next() {
            shape.move_to(p);
        }
        for p in points {
            shape.line_to(p);
        }
        if let Some(&(x, y)) = part.first() {
            shape.line_to(Pt::new(at.0 + x, at.1 + y));
        }
    }
    shape
}
