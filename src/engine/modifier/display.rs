//! `DisplayModifiers2` of Modifier2.java: turns the recorded labels into
//! text with a pixel position, an angle and a justification.

use super::{
    ABOVE_END, ABOVE_END_INSIDE, ABOVE_MIDDLE, ABOVE_MIDDLE_PERPENDICULAR, ABOVE_START_INSIDE,
    AREA, AREA_IMAGE, JUSTIFY_CENTER, JUSTIFY_LEFT, JUSTIFY_RIGHT, PlacedLabel, SCREEN, TO_END,
};
use crate::engine::base::{At, EngineError, Pt, java_round};
use crate::engine::lineutility::basics::get_quadrant_double;
use crate::engine::lineutility::extend::{extend_along_line_double, extend_directed_line};
use crate::engine::lineutility::slope::reverse_direction;
use crate::engine::tactical_lines as tl;
use crate::engine::tg::{ModifierLabel, Tg};
use core::f64::consts::PI;

/// Where one label goes.
struct Placement {
    /// Upstream's modifier position.
    position: (f64, f64),
    /// The point the label belongs to.
    anchor: (f64, f64),
    /// Offset from the anchor to the text.
    anchor_offset: (f64, f64),
    justify: i32,
    /// Radians; the path's angle, adjusted for upright text.
    theta: f64,
}

/// A label's rounded path and the sizes its placement depends on.
struct Frame {
    pt0: Pt,
    pt1: Pt,
    /// `x1`, `y1`, `x2`, `y2` after rounding.
    xy: (f64, f64, f64, f64),
    line_factor: f64,
    string_width: f64,
    string_height: f64,
    theta: f64,
}

/// Upstream `DisplayModifiers2` for the "ge" client: places every label of
/// `tg.modifiers`. `text_width` is the font's `FontMetrics.stringWidth`;
/// like upstream, 1 is added to it here. Labels without text (upstream's
/// images) and of an unknown placement type are skipped.
pub(crate) fn display_modifiers2(
    tg: &Tg,
    text_width: &dyn Fn(&str) -> f64,
    is_text_flipped: bool,
) -> Vec<PlacedLabel> {
    let mut out = Vec::new();
    if tg.modifiers.is_empty() || tg.font.size == 0 {
        return out;
    }
    let string_height = f64::from(tg.font.size);
    for label in &tg.modifiers {
        if label.text.is_empty() {
            continue;
        }
        let frame = frame_of(
            label,
            text_width(&label.text) + 1.0,
            string_height,
            is_text_flipped,
        );
        // Upstream catches a failure inside the loop and keeps the labels
        // placed so far.
        let p = match place(tg, label.kind, &frame) {
            Ok(Some(p)) => p,
            Ok(None) => continue,
            Err(_) => break,
        };
        out.push(PlacedLabel {
            text: label.text.clone(),
            x: p.position.0,
            y: p.position.1,
            angle_deg: p.theta * 180.0 / PI,
            justify: p.justify,
            anchor: p.anchor,
            anchor_offset: p.anchor_offset,
            text_id: label.text_id.clone(),
        });
    }
    out
}

/// The rounded text path of `label`, with its angle flipped upright.
fn frame_of(label: &ModifierLabel, string_width: f64, string_height: f64, flipped: bool) -> Frame {
    let [a, b] = label.text_path;
    let (x1, y1) = (java_round(a.x), java_round(a.y));
    let (x2, y2) = (java_round(b.x), java_round(b.y));
    let mut theta = (y2 - y1).atan2(x2 - x1);
    if x1 > x2 {
        theta -= PI;
    }
    Frame {
        pt0: Pt::new(x1, y1),
        pt1: Pt::new(x2, y2),
        xy: (x1, y1, x2, y2),
        line_factor: if flipped {
            -label.line_factor
        } else {
            label.line_factor
        },
        string_width,
        string_height,
        theta,
    }
}

/// Dispatch on the placement type.
fn place(tg: &Tg, kind: i32, f: &Frame) -> Result<Option<Placement>, EngineError> {
    Ok(Some(match kind {
        TO_END | ABOVE_END => place_at_end(tg, kind, f),
        ABOVE_START_INSIDE | ABOVE_END_INSIDE => place_inside(kind, f),
        ABOVE_MIDDLE | ABOVE_MIDDLE_PERPENDICULAR => place_above_middle(kind, f),
        AREA => place_area(f),
        AREA_IMAGE => {
            let (x1, y1) = (f.xy.0, f.xy.1);
            Placement {
                position: (x1.trunc(), y1.trunc()),
                anchor: (x1, y1),
                anchor_offset: (0.0, 0.0),
                justify: JUSTIFY_CENTER,
                theta: f.theta,
            }
        }
        SCREEN => place_screen(tg, f)?,
        _ => return Ok(None),
    }))
}

/// `toEnd` and `aboveEnd`: next to the first point, beside or on the line.
fn place_at_end(tg: &Tg, kind: i32, f: &Frame) -> Placement {
    let (x1, _, mut x2, _) = f.xy;
    if x1 == x2 {
        x2 += 1.0;
    }
    let mut direction = if f.line_factor >= 0.0 { 2 } else { 3 };
    // The "ge" client always reverses; LC does for every client.
    if tg.line_type == tl::LC || crate::engine::settings::CLIENT == "ge" {
        direction = reverse_direction(direction);
    }
    let justify = if (kind == TO_END && x1 < x2) || (kind == ABOVE_END && x2 < x1) {
        JUSTIFY_RIGHT
    } else {
        JUSTIFY_LEFT
    };
    let pt3 = extend_directed_line(
        f.pt1,
        f.pt0,
        f.pt0,
        direction,
        f.line_factor * f.string_height,
    );
    let mut offset = (pt3.x - f.pt0.x, pt3.y - f.pt0.y);
    // Keep the text off the line itself.
    let gap = f64::from(tg.font.size) / 2.0;
    offset.0 += if justify == JUSTIFY_RIGHT { -gap } else { gap };
    Placement {
        position: (pt3.x, pt3.y),
        anchor: (f.pt0.x, f.pt0.y),
        anchor_offset: offset,
        justify,
        theta: f.theta,
    }
}

/// `aboveStartInside` and `aboveEndInside`: a string's width in from one
/// end of the path.
fn place_inside(kind: i32, f: &Frame) -> Placement {
    let (pt3, anchor) = if kind == ABOVE_START_INSIDE {
        (
            extend_along_line_double(f.pt0, f.pt1, f.string_width),
            f.pt0,
        )
    } else {
        (
            extend_along_line_double(f.pt1, f.pt0, f.string_width),
            f.pt1,
        )
    };
    Placement {
        position: (pt3.x.trunc(), pt3.y),
        anchor: (anchor.x, anchor.y),
        anchor_offset: (pt3.x - anchor.x, pt3.y - anchor.y),
        justify: JUSTIFY_LEFT,
        theta: f.theta,
    }
}

/// `aboveMiddle` and its perpendicular variant: centred on the path's
/// midpoint, offset by the line factor.
fn place_above_middle(kind: i32, f: &Frame) -> Placement {
    let (x1, y1, x2, y2) = f.xy;
    let mid = Pt::new((x1 + x2) / 2.0, (y1 + y2) / 2.0);
    let distance = (f.line_factor * f.string_height).abs();
    let mut direction = if f.line_factor >= 0.0 { 3 } else { 2 };
    if x1 == x2 && y1 > y2 {
        direction = 1;
    }
    if x1 == x2 && y1 < y2 {
        direction = 0;
    }
    let moved = extend_directed_line(f.pt0, mid, mid, direction, distance);
    let mut theta = f.theta;
    if kind == ABOVE_MIDDLE_PERPENDICULAR {
        // Undo the upright flip, then turn a quarter.
        if x1 > x2 {
            theta += PI;
        }
        if y1 > y2 {
            theta += PI;
        }
        theta -= PI / 2.0;
    }
    Placement {
        position: (moved.x, moved.y),
        anchor: (mid.x, mid.y),
        anchor_offset: (moved.x - mid.x, moved.y - mid.y),
        justify: JUSTIFY_CENTER,
        theta,
    }
}

/// `area`: upright text centred on the first point, `lineFactor` lines
/// down. The arithmetic is in whole pixels as upstream's.
fn place_area(f: &Frame) -> Placement {
    let (x1, y1) = (f.xy.0, f.xy.1);
    let y =
        (y1 as i32) + ((f.string_height / 2.0) as i32) + ((f.line_factor * f.string_height) as i32);
    let x = x1 as i32;
    let (x, y) = (f64::from(x), f64::from(y));
    Placement {
        position: (x, y),
        anchor: (x1, y1),
        anchor_offset: (x - x1, y - y1),
        justify: JUSTIFY_CENTER,
        theta: 0.0,
    }
}

/// `screen`: text for screen, cover and guard. Upstream never records a
/// modifier position for it; the glyph position stands in.
fn place_screen(tg: &Tg, f: &Frame) -> Result<Placement, EngineError> {
    let (x1, y1) = (f.xy.0, f.xy.1);
    let width = f.string_width as i32;
    let height = f.string_height as i32;
    let lf = (f.line_factor * f.string_height) as i32;
    let (x, y, anchor, theta);
    if tg.pixels.len() >= 14 {
        let (pt1, pt2) = (tg.pixels.at(3)?, tg.pixels.at(10)?);
        let quadrant = get_quadrant_double(pt1, pt2);
        let mut t = (pt2.y - pt1.y).atan2(pt2.x - pt1.x);
        if t.abs() < PI / 8.0 {
            t += if t < 0.0 { -PI / 2.0 } else { PI / 2.0 };
        }
        match quadrant {
            1 | 4 => t += PI / 2.0,
            2 | 3 => t -= PI / 2.0,
            _ => {}
        }
        theta = t;
        x = (x1 as i32) - width / 2;
        y = (y1 as i32) + height / 2 + lf;
        anchor = (x1, y1);
    } else {
        let first = tg.pixels.at(0)?;
        let (ax, ay) = (first.x as i32, first.y as i32);
        theta = 0.0;
        anchor = (f64::from(ax), f64::from(ay));
        x = ax - width / 2;
        y = ay - height / 2 + lf + height / 2 + lf;
    }
    let (x, y) = (f64::from(x), f64::from(y));
    Ok(Placement {
        position: (x, y),
        anchor,
        anchor_offset: (x - anchor.0, y - anchor.1),
        justify: JUSTIFY_LEFT,
        theta,
    })
}
