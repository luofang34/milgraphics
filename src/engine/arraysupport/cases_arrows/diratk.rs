//! DIRATKGND, DIRATKSPT, MFLANE/RAFT and the infiltration arrows.

use super::{arrow, clamp_mbr};
use crate::engine::arraysupport::work::{Work, get, set, set_style};
use crate::engine::base::EngineError;
use crate::engine::dism::delay::get_infiltration_double;
use crate::engine::lineutility::basics::{calc_distance_double, reverse_points_double2};
use crate::engine::lineutility::extend::{extend_along_line_double, extend_line_double};

/// DIRATKGND: the line, a filled arrow at the end and a thin arrow.
pub(crate) fn dir_atk_gnd(w: &mut Work<'_>) -> Result<(), EngineError> {
    let d_mbr = clamp_mbr(w.d_mbr, w.dpi, 30.0, Some(150.0), Some(500.0));
    let (p0, p1) = (get(&w.p, 0)?, get(&w.p, 1)?);
    let d = calc_distance_double(p0, p1);
    if d < d_mbr / 40.0 {
        set(&mut w.p, 1, extend_line_double(p0, p1, d_mbr / 40.0 + 1.0))?;
    }
    let (p0, p1) = (get(&w.p, 0)?, get(&w.p, 1)?);
    set(&mut w.p, 0, extend_along_line_double(p0, p1, d_mbr / 40.0))?;
    reverse_points_double2(&mut w.p, w.save)?;

    let v = w.vbl;
    let pt0 = get(&w.p, v - 12)?;
    let pt1 = get(&w.p, v - 11)?;
    let pt2 = extend_line_double(pt0, pt1, d_mbr / 40.0);
    let n = (d_mbr as i32) / 20;
    let a = arrow(pt0, pt1, n, n, 0)?;
    for (j, pt) in (0..).zip(a) {
        set(&mut w.p, v - 10 + j, pt)?;
    }
    let n = (d_mbr / 13.33) as i32;
    let a = arrow(pt0, pt2, n, n, 0)?;
    for (j, pt) in (0..).zip(a) {
        set(&mut w.p, v - 7 + j, pt)?;
    }
    let copies = [
        (v - 4, v - 10, 0),
        (v - 3, v - 7, 5),
        (v - 2, v - 8, 0),
        (v - 1, v - 5, 5),
    ];
    for (to, from, style) in copies {
        let mut pt = get(&w.p, from)?;
        pt.style = style;
        set(&mut w.p, to, pt)?;
    }
    w.ac = v;
    Ok(())
}

/// MFLANE and RAFT: the lane with an arrowhead at each end.
pub(crate) fn mf_lane(w: &mut Work<'_>) -> Result<(), EngineError> {
    let v = w.vbl;
    let d_mbr = w.d_mbr;
    let pt2 = extend_line_double(get(&w.p, v - 8)?, get(&w.p, v - 7)?, d_mbr / 2.0);
    let pt3 = get(&w.p, v - 7)?;
    let pt1 = extend_line_double(get(&w.p, 1)?, get(&w.p, 0)?, d_mbr / 2.0);
    let d_mbr = clamp_mbr(d_mbr, w.dpi, 10.0, None, Some(250.0));
    let (n, base) = ((d_mbr as i32) / 10, (d_mbr as i32) / 5);
    let a = arrow(pt2, pt3, n, base, 0)?;
    for (k, pt) in (0..).zip(a) {
        set(&mut w.p, v - 6 + k, pt)?;
    }
    let a = arrow(pt1, w.pt0, n, base, 0)?;
    for (k, pt) in (0..).zip(a) {
        set(&mut w.p, v - 3 + k, pt)?;
    }
    set_style(&mut w.p, w.save - 1, 5)?;
    w.ac = v;
    Ok(())
}

/// DIRATKSPT: the line with a closing arrowhead in the last three slots.
pub(crate) fn dir_atk_spt(w: &mut Work<'_>) -> Result<(), EngineError> {
    reverse_points_double2(&mut w.p, w.save)?;
    let d_mbr = clamp_mbr(w.d_mbr, w.dpi, 20.0, Some(150.0), Some(500.0));
    let v = w.vbl;
    let n = (d_mbr as i32) / 20;
    let a = arrow(get(&w.p, v - 5)?, get(&w.p, v - 4)?, n, n, 0)?;
    for (k, pt) in (0..).zip(a) {
        set(&mut w.p, v - k - 1, pt)?;
    }
    w.ac = v;
    Ok(())
}

/// EXFILTRATION and INFILTRATION: the S-shaped path, then the arrowhead
/// replaces its last three slots. The new array also is the returned list.
pub(crate) fn infiltration(w: &mut Work<'_>) -> Result<(), EngineError> {
    let d_mbr = clamp_mbr(w.d_mbr, w.dpi, 20.0, Some(150.0), Some(500.0));
    w.p = get_infiltration_double(&w.p, w.settings)?;
    w.vbl = i32::try_from(w.p.len()).map_err(|_| EngineError::Degenerate("too many points"))?;
    let v = w.vbl;
    let n = (d_mbr as i32) / 20;
    let a = arrow(get(&w.p, v - 5)?, get(&w.p, v - 4)?, n, n, 0)?;
    for (k, pt) in (0..).zip(a) {
        set(&mut w.p, v - k - 1, pt)?;
    }
    w.points = w.p.clone();
    w.ac = v;
    Ok(())
}
