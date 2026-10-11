//! The near box: the viewport grown by its own size on every side, the
//! farthest the screen tier reaches beyond what the host draws.
//!
//! A perspective camera projects ground just in front of it arbitrarily far
//! off screen, and where a line runs under the camera the horizon search
//! converges on such points. Cutting the screen tier at the near box keeps
//! every coordinate and every line's length bounded by the viewport's size,
//! so host work that grows with pixel length, such as dashing, stays bounded
//! too, while everything within a viewport's width of the screen is drawn
//! as projected.

use crate::engine::visible::PixelBox;
use crate::render::{ScreenPoint, ScreenRect};

/// The near box of `viewport`, if the host gives one.
pub(crate) fn near(viewport: Option<ScreenRect>) -> Option<PixelBox> {
    near_rect(viewport).map(|r| PixelBox {
        min_x: r.min.x,
        min_y: r.min.y,
        max_x: r.max.x,
        max_y: r.max.y,
    })
}

/// [`near`] as a screen rectangle.
pub(crate) fn near_rect(viewport: Option<ScreenRect>) -> Option<ScreenRect> {
    let v = viewport?;
    let grow = (v.max.x - v.min.x).max(v.max.y - v.min.y);
    Some(ScreenRect {
        min: ScreenPoint {
            x: v.min.x - grow,
            y: v.min.y - grow,
        },
        max: ScreenPoint {
            x: v.max.x + grow,
            y: v.max.y + grow,
        },
    })
}

/// Whether `p` is inside `r`; a non-finite point never is.
pub(crate) fn contains(r: ScreenRect, p: ScreenPoint) -> bool {
    (r.min.x..=r.max.x).contains(&p.x) && (r.min.y..=r.max.y).contains(&p.y)
}

/// The parts of the polyline inside `r`, as separate runs. Segments with a
/// non-finite end are dropped.
pub(crate) fn clip(points: &[ScreenPoint], r: ScreenRect) -> Vec<Vec<ScreenPoint>> {
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
    if ![a.x, a.y, b.x, b.y].iter().all(|v| v.is_finite()) {
        return None;
    }
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
