//! Bounding the pixel length the ported renderer is given: its work and
//! output grow with the length of the line in pixels, and a close or tilted
//! view can stretch a short line across millions of them.

use crate::engine::render_utility::interpolate::repeats_along_open_line;
use crate::engine::tactical_lines as lt;
use crate::engine::visible::PixelBox;
use crate::render::{ScreenPoint, ScreenRect};
use crate::sidc::SymbolId;

/// Longest line, in pixels, drawn whole.
pub(super) const MAX_LINE_PX: f64 = 100_000.0;

/// The viewport grown by its own size on every side: repeats, cut ends and
/// labels kept beyond the viewport are drawn only this far out.
pub(super) fn near(viewport: Option<ScreenRect>) -> Option<PixelBox> {
    let v = viewport?;
    let grow = (v.max.x - v.min.x).max(v.max.y - v.min.y);
    Some(PixelBox {
        min_x: v.min.x - grow,
        min_y: v.min.y - grow,
        max_x: v.max.x + grow,
        max_y: v.max.y + grow,
    })
}

/// Whether the generator of `line_type` for `symbol` emits only the repeats
/// near the view, so the graphic is drawn whole at any length when the view
/// is known.
pub(super) fn limits_itself(line_type: i32, symbol: &SymbolId) -> bool {
    matches!(
        line_type,
        lt::OBSAREA
            | lt::OBSFAREA
            | lt::ZONE
            | lt::ENCIRCLE
            | lt::STRONG
            | lt::FORT_REVD
            | lt::FORT
    ) || crate::engine::edition::spreads_wire(symbol)
}

/// The stretches of the projected control points to draw: the whole line
/// when it is short enough or, `whole`, its generator keeps to `near`; otherwise, for
/// an open line repeating its glyph, the parts inside `near`, so cut ends
/// stay off screen; otherwise nothing.
pub(super) fn runs(
    (line_type, whole): (i32, bool),
    points: Vec<ScreenPoint>,
    near: Option<PixelBox>,
) -> Vec<Vec<ScreenPoint>> {
    if length(&points) <= MAX_LINE_PX || (near.is_some() && whole) {
        return vec![points];
    }
    let Some(b) = near.filter(|_| repeats_along_open_line(line_type)) else {
        return Vec::new();
    };
    let bounds = ScreenRect {
        min: ScreenPoint {
            x: b.min_x,
            y: b.min_y,
        },
        max: ScreenPoint {
            x: b.max_x,
            y: b.max_y,
        },
    };
    clip(&points, bounds)
        .into_iter()
        .filter(|run| run.len() >= 2 && length(run) <= MAX_LINE_PX)
        .collect()
}

fn length(points: &[ScreenPoint]) -> f64 {
    points
        .windows(2)
        .map(|w| match w {
            [a, b] => (b.x - a.x).hypot(b.y - a.y),
            _ => 0.0,
        })
        .sum()
}

/// The parts of the polyline inside `r`, as separate runs.
pub(super) fn clip(points: &[ScreenPoint], r: ScreenRect) -> Vec<Vec<ScreenPoint>> {
    let mut runs: Vec<Vec<ScreenPoint>> = Vec::new();
    let mut open = false;
    for w in points.windows(2) {
        let [a, b] = w else { continue };
        let Some((p, q, exits)) = clip_segment(*a, *b, r) else {
            open = false;
            continue;
        };
        match runs.last_mut() {
            Some(run) if open => run.push(q),
            _ => runs.push(vec![p, q]),
        }
        open = !exits;
    }
    runs
}

/// Liang–Barsky: the part of `a`–`b` inside `r`, and whether it leaves `r`
/// before reaching `b`.
fn clip_segment(
    a: ScreenPoint,
    b: ScreenPoint,
    r: ScreenRect,
) -> Option<(ScreenPoint, ScreenPoint, bool)> {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let (mut t0, mut t1) = (0.0_f64, 1.0_f64);
    for (p, q) in [
        (-dx, a.x - r.min.x),
        (dx, r.max.x - a.x),
        (-dy, a.y - r.min.y),
        (dy, r.max.y - a.y),
    ] {
        if p == 0.0 {
            if q < 0.0 {
                return None;
            }
        } else {
            let t = q / p;
            if p < 0.0 {
                t0 = t0.max(t);
            } else {
                t1 = t1.min(t);
            }
        }
    }
    if t0 > t1 {
        return None;
    }
    let at = |t: f64| ScreenPoint {
        x: a.x + dx * t,
        y: a.y + dy * t,
    };
    Some((at(t0), at(t1), t1 < 1.0))
}
