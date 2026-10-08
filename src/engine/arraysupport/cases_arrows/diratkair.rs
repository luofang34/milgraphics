//! DIRATKAIR: the air attack line with arrowheads and a bow tie on the
//! middle segment.

use super::{arrow, clamp_mbr};
use crate::engine::arraysupport::work::{Work, get, set, set_style};
use crate::engine::base::{EngineError, Pt};
use crate::engine::lineutility::basics::{
    calc_distance_double, mid_point_double, reverse_points_double2,
};
use crate::engine::lineutility::extend::extend_along_line_double_style;

/// The segment (and its endpoints) that carries the bow tie: the first one
/// from the end that makes the line longer than 60 pixels, else a fixed one.
fn middle_segment(w: &Work<'_>) -> Result<(i32, Pt, Option<Pt>), EngineError> {
    let mut d = 0.0;
    let mut k = w.save - 1;
    while k > 0 {
        d += calc_distance_double(get(&w.p, k)?, get(&w.p, k - 1)?);
        if d > 60.0 {
            break;
        }
        k -= 1;
    }
    let middle = if d > 60.0 {
        k
    } else if w.save <= 3 {
        1
    } else {
        2
    };
    let pt2 = get(&w.p, middle)?;
    let pt3 = if middle >= 1 {
        Some(get(&w.p, middle - 1)?)
    } else {
        None
    };
    Ok((middle, pt2, pt3))
}

/// Writes the three arrow points at `at`.
fn put3(w: &mut Work<'_>, at: i32, a: [Pt; 3]) -> Result<(), EngineError> {
    for (j, pt) in (0..).zip(a) {
        set(&mut w.p, at + j, pt)?;
    }
    Ok(())
}

/// DIRATKAIR.
pub(crate) fn dir_atk_air(w: &mut Work<'_>) -> Result<(), EngineError> {
    reverse_points_double2(&mut w.p, w.save)?;
    let (middle, pt2, pt3) = middle_segment(w)?;
    let d_mbr = clamp_mbr(w.d_mbr, w.dpi, 20.0, Some(150.0), Some(250.0));
    let n = (d_mbr as i32) / 20;
    let v = w.vbl;
    let tail = get(&w.p, v - 11)?;
    let head = get(&w.p, v - 10)?;
    put3(w, v - 9, arrow(tail, head, n, n, 0)?)?;

    let mid = Pt {
        x: (tail.x + head.x) / 2.0,
        y: (tail.y + head.y) / 2.0,
        ..get(&w.p, v - 6)?
    };
    set(&mut w.p, v - 6, mid)?;
    let mut pt0 = mid;
    let mut a = arrow(tail, pt0, n, n, 9)?;
    if let (true, Some(pt3)) = (middle >= 1, pt3) {
        pt0 = mid_point_double(pt2, pt3, 0);
        a = arrow(pt3, pt0, n, n, 9)?;
    }
    put3(w, v - 6, a)?;

    let head = get(&w.p, v - 10)?;
    let mut a = arrow(head, pt0, n, n, 9)?;
    if let (true, Some(pt3)) = (middle >= 1, pt3) {
        pt0 = mid_point_double(pt2, pt3, 0);
        a = arrow(pt2, pt0, n, n, 9)?;
    }
    put3(w, v - 3, a)?;

    // The bow tie must not be filled, so its outline is separate points.
    set_style(&mut w.p, middle - 1, 5)?;
    if w.save == 2 {
        set_style(&mut w.p, 1, 5)?;
    }
    let mut air: Vec<Pt> = (0..v).map(|j| get(&w.p, j)).collect::<Result<_, _>>()?;
    let cur = get(&w.p, middle)?;
    let prev = get(&w.p, middle - 1)?;
    let midpt = mid_point_double(prev, cur, 0);
    air.push(extend_along_line_double_style(midpt, cur, d_mbr / 20.0, 0));
    air.push(Pt { style: 5, ..cur });
    air.push(extend_along_line_double_style(midpt, prev, d_mbr / 20.0, 0));
    air.push(Pt { style: 5, ..prev });
    w.vbl = i32::try_from(air.len()).map_err(|_| EngineError::Degenerate("too many points"))?;
    w.p = air;
    w.ac = w.vbl;
    Ok(())
}
