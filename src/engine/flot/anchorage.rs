//! Port of the anchorage helpers in flot.java (`GetAnchorageFlotSegment`,
//! `GetAnchorageCountDouble`): every other flot along a segment.

use super::angle::calc_angle_points;
use super::count;
use super::segment::{
    FlipState, flot_center, heading_degrees, oriented_angle, segment_coords, write_arc,
};
use crate::engine::base::{At, EngineError, Pt};
use crate::engine::lineutility::basics::calc_distance_double;
use crate::engine::lineutility::intersect::calc_distance2;

/// Upstream `GetAnchorageFlotSegment`: writes the flots of the segment from
/// (`x1`, `y1`) to (`x2`, `y2`), skipping alternate ones, into `points`
/// (30 values per flot) and returns the number of values written. `vb`
/// holds the polyline's integer coordinates and `segment` the index of this
/// segment in it.
pub(crate) fn get_anchorage_flot_segment(
    vb: &[i32],
    ends: (i32, i32, i32, i32),
    segment: usize,
    float_diameter: f64,
    points: &mut [i32],
    state: &mut FlipState,
) -> Result<usize, EngineError> {
    let (x1, y1, x2, y2) = ends;
    if segment == 0 {
        state.flip = i32::from(vb.at(0)? >= vb.at(2)?);
    }
    let c = segment_coords(vb, segment)?;
    let heading = heading_degrees(c);
    let mut distance = calc_distance2(i64::from(x1), i64::from(y1), i64::from(x2), i64::from(y2));
    let mut num_segs = (distance / float_diameter) as i32;
    if num_segs % 2 == 0 {
        num_segs -= 1;
    }
    distance += f64::from(num_segs) * float_diameter - distance;
    let angle = oriented_angle(c, segment, heading, state);
    let mut written = 0_usize;
    let mut m = 0;
    while m < num_segs {
        let (lx, ly) = flot_center([x1, y1, x2, y2], m, float_diameter, distance);
        let radius = distance / (f64::from(num_segs) * 2.0);
        write_arc(points, written, &calc_angle_points(lx, ly, angle, radius))?;
        written += 30;
        m += 2;
    }
    state.last_direction = state.direction;
    Ok(written)
}

/// Upstream `GetAnchorageCountDouble`: the points an anchorage line over
/// the first `num_pts` points needs (12 per flot, one for a segment too
/// short for one, one for the end).
pub(crate) fn get_anchorage_count_double(
    pts: &[Pt],
    float_diameter: f64,
    num_pts: i32,
) -> Result<i32, EngineError> {
    let n = count(num_pts)?;
    let mut total = 0_i32;
    for j in 0..n.saturating_sub(1) {
        let d = calc_distance_double(pts.at(j)?, pts.at(j + 1)?);
        let segs = (d / float_diameter) as i32;
        total = total.wrapping_add(if segs > 0 { segs.wrapping_mul(12) } else { 1 });
    }
    Ok(total.wrapping_add(1))
}
