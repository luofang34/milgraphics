//! Bounding the pixel length the ported renderer is given: its work and
//! output grow with the length of the line in pixels, and a close or tilted
//! view can stretch a short line across millions of them.

use crate::engine::render_utility::interpolate::repeats_along_open_line;
use crate::engine::tactical_lines as lt;
use crate::engine::visible::PixelBox;
pub(super) use crate::render::near::{clip, near};
use crate::render::{ScreenPoint, ScreenRect};
use crate::sidc::SymbolId;

/// Longest line, in pixels, drawn whole.
pub(super) const MAX_LINE_PX: f64 = 100_000.0;

/// Length of the fixed stretches a longer line is drawn in. Each starts a
/// whole number of stretches from the line's first point, so its ends stay
/// put on the line while the view pans and its pattern does not slide.
pub(super) const STRETCH_PX: f64 = 20_000.0;

/// Most stretches drawn for one view; past it the line is cut at `near`.
const MAX_STRETCHES: usize = 16;

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

/// The stretches of the control points, in the frame's pixels, to draw: the whole line
/// when it is short enough or, `whole`, its generator keeps to `near`;
/// otherwise, for an open line repeating its glyph, the fixed stretches
/// ([`STRETCH_PX`]) that reach `near`, or for a line passing `near` too
/// often, the parts inside `near`, so cut ends stay off screen; otherwise
/// nothing.
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
    if let Some(fixed) = stretches(&points, &b) {
        return fixed;
    }
    clip(&points, bounds)
        .into_iter()
        .filter(|run| run.len() >= 2 && length(run) <= MAX_LINE_PX)
        .collect()
}

/// The fixed stretches of `points` that reach `near`, each whole; `None`
/// when more than [`MAX_STRETCHES`] do.
pub(super) fn stretches(points: &[ScreenPoint], near: &PixelBox) -> Option<Vec<Vec<ScreenPoint>>> {
    let pt = |p: &ScreenPoint| crate::engine::base::Pt::new(p.x, p.y);
    let mut wanted = std::collections::BTreeSet::new();
    let mut walked = 0.0;
    for w in points.windows(2) {
        let [a, b] = w else { continue };
        if let Some((lo, hi)) = near.span(pt(a), pt(b)) {
            let first = ((walked + lo) / STRETCH_PX).floor() as u64;
            let last = ((walked + hi) / STRETCH_PX).floor() as u64;
            for j in first..=last {
                wanted.insert(j);
                if wanted.len() > MAX_STRETCHES {
                    return None;
                }
            }
        }
        walked += (b.x - a.x).hypot(b.y - a.y);
    }
    Some(
        wanted
            .into_iter()
            .map(|j| between(points, j as f64 * STRETCH_PX, (j + 1) as f64 * STRETCH_PX))
            .filter(|run| run.len() >= 2)
            .collect(),
    )
}

/// The part of the polyline from `from` to `to` pixels along it.
fn between(points: &[ScreenPoint], from: f64, to: f64) -> Vec<ScreenPoint> {
    let mut out = Vec::new();
    let mut walked = 0.0;
    for w in points.windows(2) {
        let [a, b] = w else { continue };
        let len = (b.x - a.x).hypot(b.y - a.y);
        let at = |d: f64| {
            let t = if len > 0.0 {
                ((d - walked) / len).clamp(0.0, 1.0)
            } else {
                0.0
            };
            ScreenPoint {
                x: a.x + (b.x - a.x) * t,
                y: a.y + (b.y - a.y) * t,
            }
        };
        if walked + len >= from && walked <= to {
            if out.is_empty() {
                out.push(at(from.max(walked)));
            }
            out.push(at(to.min(walked + len)));
        }
        walked += len;
        if walked > to {
            break;
        }
    }
    out
}

#[cfg(test)]
pub(super) fn length_of(points: &[ScreenPoint]) -> f64 {
    length(points)
}

pub(super) fn length(points: &[ScreenPoint]) -> f64 {
    points
        .windows(2)
        .map(|w| match w {
            [a, b] => (b.x - a.x).hypot(b.y - a.y),
            _ => 0.0,
        })
        .sum()
}
