//! Port of the squall-line helpers of lineutility.java.

use super::basics::calc_distance_double;
use super::extend::extend_line_double;
use super::transform::rotate_geometry_double_origin;
use crate::engine::base::{At, EngineError, Pt, idx};
use std::f64::consts::PI;

/// Upstream `GetSquallCurve`: one half sine wave of `quantity` points along
/// the segment, bulging by `amplitude` on the side given by `sign`.
fn get_squall_curve(
    start: Pt,
    end: Pt,
    out: &mut [Pt],
    sign: i32,
    amplitude: f64,
    quantity: i32,
) -> Result<(), EngineError> {
    let dist = calc_distance_double(start, end);
    for j in 0..quantity {
        let t = extend_line_double(end, start, -dist * f64::from(j) / f64::from(quantity));
        let y = t.y
            + amplitude
                * f64::from(sign)
                * (f64::from(j) * 180.0 / f64::from(quantity) * PI / 180.0).sin();
        *out.at_mut(idx(j, out.len())?)? = Pt::new(t.x, y);
    }
    Ok(())
}

/// Upstream `GetSquallSegment`: writes the squall wave between the two
/// points into `squall_pts` and returns the number of points written. The
/// wave side alternates, so `sign` is updated for the caller.
pub(crate) fn get_squall_segment(
    start_pt: Pt,
    end_pt: Pt,
    squall_pts: &mut [Pt],
    sign: &mut i32,
    amplitude: f64,
    quantity: i32,
    length: f64,
) -> Result<i32, EngineError> {
    let mut counter: i32 = 0;
    let mut curve = vec![Pt::default(); idx(quantity, 0)?];
    let dist = calc_distance_double(start_pt, end_pt);
    let num_curves = (dist / length) as i32;
    let angle = ((start_pt.y - end_pt.y) / (start_pt.x - end_pt.x)).atan();
    let l_angle = ((180.0 / PI) * angle) as i32;
    let end_pt2 = Pt::new(
        if start_pt.x < end_pt.x {
            start_pt.x + dist
        } else {
            start_pt.x - dist
        },
        start_pt.y,
    );
    for j in 0..num_curves {
        let start_curve = extend_line_double(end_pt2, start_pt, -(f64::from(j) * length));
        let end_curve = extend_line_double(end_pt2, start_pt, -(f64::from(j + 1) * length));
        get_squall_curve(
            start_curve,
            end_curve,
            &mut curve,
            *sign,
            amplitude,
            quantity,
        )?;
        for p in &curve {
            *squall_pts.at_mut(idx(counter, squall_pts.len())?)? = *p;
            counter += 1;
        }
        *sign = -*sign;
    }
    if num_curves == 0 {
        *squall_pts.at_mut(idx(counter, squall_pts.len())?)? = start_pt;
        counter += 1;
        *squall_pts.at_mut(idx(counter, squall_pts.len())?)? = end_pt;
        counter += 1;
    }
    rotate_geometry_double_origin(squall_pts, counter, l_angle)?;
    Ok(counter)
}
