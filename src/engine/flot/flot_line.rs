//! Port of the plain flot builders in flot.java (`GetFlotCountDouble`,
//! `GetFlotDouble`), used by the FLOT and LC line types.

use super::segment::{FlipState, get_flot_segment};
use super::{budgeted_len, count, int_coords, slot};
use crate::engine::base::{At, EngineError, Pt, idx};
use crate::engine::lineutility::basics::calc_distance_double;
use crate::engine::lineutility::intersect::calc_distance2;

/// Upstream `GetFlotCountDouble`: the number of points a flot line over the
/// first `num_pts` points needs (10 per flot, one for a segment too short
/// for a flot, one for the end).
pub(crate) fn get_flot_count_double(
    pts: &[Pt],
    flot_diameter: f64,
    num_pts: i32,
) -> Result<i32, EngineError> {
    let n = count(num_pts)?;
    let mut total = 0_i32;
    for j in 0..n.saturating_sub(1) {
        let d = calc_distance_double(pts.at(j)?, pts.at(j + 1)?);
        let segs = (d / flot_diameter) as i32;
        total = total.wrapping_add(if segs > 0 { segs.wrapping_mul(10) } else { 1 });
    }
    Ok(total.wrapping_add(1))
}

/// Upstream `GetFlotDouble`: replaces the first `num_pts` points of `pts`
/// with the flot line's points (10 per flot, the half circles of diameter
/// `flot_diameter` on one side, alternating sides as the line turns) and
/// returns how many were written. Style 5 marks the last point and
/// everything after it.
pub(crate) fn get_flot_double(
    pts: &mut Vec<Pt>,
    flot_diameter: f64,
    num_pts: i32,
) -> Result<i32, EngineError> {
    let n = count(num_pts)?;
    let flot_count = get_flot_count_double(pts, flot_diameter, num_pts)?;
    let vb = int_coords(pts, n)?;
    let mut state = FlipState::unset();
    let mut written = 0_i32;
    for l in 0..n.saturating_sub(1) {
        let c = super::segment::segment_coords(&vb, l)?;
        let distance = calc_distance2(
            i64::from(c[0]),
            i64::from(c[1]),
            i64::from(c[2]),
            i64::from(c[3]),
        );
        let segs = (distance / flot_diameter) as i32;
        if segs > 0 {
            let mut points = vec![0_i32; budgeted_len(segs, 30)?];
            let seg_pts = get_flot_segment(&vb, l, Some(&mut points), flot_diameter, &mut state)?;
            for j in 0..idx(seg_pts, 0)? {
                let (x, y) = (points.at(j * 3)?, points.at(j * 3 + 1)?);
                if written < flot_count {
                    let p = slot(pts, idx(written, 0)?)?;
                    p.x = f64::from(x);
                    p.y = f64::from(y);
                    written += 1;
                }
            }
        } else {
            // The orientation state must still advance through short segments.
            get_flot_segment(&vb, l, None, flot_diameter, &mut state)?;
            if written < flot_count {
                let p = slot(pts, idx(written, 0)?)?;
                p.x = f64::from(c[0]);
                p.y = f64::from(c[1]);
                written += 1;
            }
        }
    }
    let first_end = idx(written - 1, pts.len())?;
    for p in pts.iter_mut().skip(first_end) {
        p.style = 5;
    }
    Ok(written)
}
