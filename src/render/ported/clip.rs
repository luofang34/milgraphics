//! Bounding the pixel length the ported renderer is given: its work and
//! output grow with the length of the line in pixels, and a close or tilted
//! view can stretch a short line across millions of them.

use crate::engine::render_utility::interpolate::repeats_along_open_line;
use crate::render::{ScreenPoint, ScreenRect};

/// Longest line, in pixels, drawn whole.
pub(super) const MAX_LINE_PX: f64 = 100_000.0;

/// The stretches of the projected control points to draw: the whole line
/// when it is short enough; otherwise, for an open line repeating its glyph,
/// the parts inside the viewport grown by its own size on every side, so cut
/// ends stay off screen; otherwise nothing.
pub(super) fn runs(
    line_type: i32,
    points: Vec<ScreenPoint>,
    viewport: Option<ScreenRect>,
) -> Vec<Vec<ScreenPoint>> {
    if length(&points) <= MAX_LINE_PX {
        return vec![points];
    }
    let Some(v) = viewport.filter(|_| repeats_along_open_line(line_type)) else {
        return Vec::new();
    };
    let grow = (v.max.x - v.min.x).max(v.max.y - v.min.y);
    let bounds = ScreenRect {
        min: ScreenPoint {
            x: v.min.x - grow,
            y: v.min.y - grow,
        },
        max: ScreenPoint {
            x: v.max.x + grow,
            y: v.max.y + grow,
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
