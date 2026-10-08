//! EXPLOIT and ABATIS.

use super::arrow;
use crate::engine::arraysupport::work::{Work, get, set, set_style};
use crate::engine::base::{EngineError, Pt};
use crate::engine::lineutility::basics::{calc_distance_double, get_pixels_min};
use crate::engine::lineutility::extend::extend_line_double;
use crate::engine::lineutility::slope::calc_true_slope_double;

/// EXPLOIT: the line plus an arrowhead at the second point and a dashed
/// tail at the third.
pub(crate) fn exploit(w: &mut Work<'_>) -> Result<(), EngineError> {
    let (pt0, pt1, pt2) = (w.pt0, w.pt1, w.pt2);
    let tri = (calc_distance_double(pt1, pt2) / 2.0_f64.sqrt()) as i32;
    let a = arrow(pt1, pt0, tri, tri * 2, 0)?;
    for (k, pt) in (0..).zip(a) {
        set(&mut w.p, k + 2, pt)?;
    }
    let a = arrow(extend_line_double(pt0, pt1, 10.0), pt1, tri, tri * 2, 1)?;
    for (k, pt) in (0..).zip(a) {
        set(&mut w.p, k + 5, pt)?;
    }
    w.ac = w.vbl;
    Ok(())
}

/// ABATIS: the line closed back to its start, with a tick at the middle.
pub(crate) fn abatis(w: &mut Work<'_>) -> Result<(), EngineError> {
    let (pt0, pt1) = (w.pt0, w.pt1);
    let (min_x, _) = get_pixels_min(&[pt0, pt1], 2)?;
    // The offset keeps the y-intercept construction away from the points.
    let offset_x = if min_x <= 0.0 { min_x - 100.0 } else { 0.0 };
    let mut d_mbr = w.d_mbr;
    if d_mbr > 300.0 * w.dpi {
        d_mbr = 300.0 * w.dpi;
    }
    let v = w.vbl;
    let moved = extend_line_double(get(&w.p, 1)?, get(&w.p, 0)?, -d_mbr / 10.0);
    set(&mut w.p, 0, moved)?;
    let (vertical, m) = calc_true_slope_double(pt0, pt1);
    let first = get(&w.p, 0)?;
    let midpt = Pt {
        x: (pt0.x + first.x) / 2.0,
        y: (pt0.y + first.y) / 2.0,
        ..first
    };
    set(&mut w.p, v - 3, pt0)?;
    set_style(&mut w.p, v - 4, 5)?;
    set_style(&mut w.p, v - 3, 0)?;
    let tick = d_mbr / 20.0;
    let mut end = get(&w.p, v - 2)?;
    if vertical != 0 && m != 0.0 {
        let b = midpt.y + (1.0 / m) * midpt.x;
        let b1 = (-1.0 / m) * offset_x + b;
        let intercept = Pt {
            x: offset_x,
            y: b1,
            ..pt0
        };
        end = extend_line_double(intercept, midpt, tick);
        if end.y >= midpt.y {
            end = extend_line_double(intercept, midpt, -tick);
        }
    }
    if vertical != 0 && m == 0.0 {
        end = midpt;
        end.y = midpt.y - tick;
    }
    if vertical == 0 {
        end = midpt;
        end.x = midpt.x - tick;
    }
    end.style = 0;
    set(&mut w.p, v - 2, end)?;
    let closing = get(&w.p, 0)?;
    set(&mut w.p, v - 1, closing)?;
    w.ac = v;
    Ok(())
}
