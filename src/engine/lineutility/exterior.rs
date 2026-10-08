//! Port of `getExteriorPoints`, `getDeepCopy` and `createStrokedShape` from
//! lineutility.java.

use super::extend::extend_directed_line;
use super::slope::{calc_true_intersect_double2, calc_true_slope_double2, reverse_direction};
use crate::engine::base::{At, EngineError, PathOp, Pt, idx};

/// Upstream `getExteriorPoints`: replaces the first `vbl_counter` points with
/// the corners of the offset polygon (outside, or inside when `interior`).
/// The offset distance is the `style` of the first point, updated by any
/// later point with a positive style.
///
/// `inside_outside` is upstream's `arraysupport.GetInsideOutsideDouble2`,
/// called as `(pt0, pt1, all points, vbl_counter, index, line_type)`.
pub(crate) fn get_exterior_points<F>(
    pts: &mut [Pt],
    vbl_counter: i32,
    line_type: i32,
    interior: bool,
    inside_outside: F,
) -> Result<(), EngineError>
where
    F: Fn(Pt, Pt, &[Pt], i32, i32, i32) -> Result<i32, EngineError>,
{
    let len = pts.len();
    let mut dist = f64::from(pts.at(0)?.style);
    let mut out: Vec<Pt> = Vec::new();
    let direction_of = |a: Pt, b: Pt, index: i32| -> Result<i32, EngineError> {
        let d = inside_outside(a, b, pts, vbl_counter, index, line_type)?;
        Ok(if interior { reverse_direction(d) } else { d })
    };
    for j in 0..vbl_counter {
        let (i0, i1, i2) = if j == 0 || j == vbl_counter - 1 {
            (vbl_counter - 2, 0, 1)
        } else {
            (j - 1, j, j + 1)
        };
        let pt0 = pts.at(idx(i0, len)?)?;
        let pt1 = pts.at(idx(i1, len)?)?;
        let pt2 = pts.at(idx(i2, len)?)?;
        if pt1.style > 0 {
            dist = f64::from(pt1.style);
        }
        let prev_index = if j - 1 < 0 { vbl_counter - 1 } else { j - 1 };
        let dir0 = direction_of(pt0, pt1, prev_index)?;
        let pt00 = extend_directed_line(pt0, pt1, pt0, dir0, dist);
        let pt01 = extend_directed_line(pt0, pt1, pt1, dir0, dist);
        let index = if j == vbl_counter - 1 { 0 } else { j };
        let dir1 = direction_of(pt1, pt2, index)?;
        let pt10 = extend_directed_line(pt1, pt2, pt1, dir1, dist);
        let pt11 = extend_directed_line(pt1, pt2, pt2, dir1, dist);
        if pt0.x == pt1.x && pt1.x == pt2.x {
            out.push(pt01);
            continue;
        }
        let (_, m01) = calc_true_slope_double2(pt00, pt01);
        let (_, m12) = calc_true_slope_double2(pt10, pt11);
        if m01 == m12 {
            out.push(pt01);
            continue;
        }
        let b01 = pt01.y - m01 * pt01.x;
        let b12 = pt11.y - m12 * pt11.x;
        out.push(calc_true_intersect_double2(
            m01,
            b01,
            m12,
            b12,
            1,
            1,
            (0.0, 0.0),
        ));
    }
    for (j, p) in out.into_iter().enumerate() {
        *pts.at_mut(j)? = p;
    }
    Ok(())
}

/// Upstream `getDeepCopy`: copies position and style; the segment number is
/// reset.
pub(crate) fn get_deep_copy(pts: &[Pt]) -> Vec<Pt> {
    pts.iter().map(|p| Pt::styled(p.x, p.y, p.style)).collect()
}

/// Upstream `createStrokedShape`: despite its name, a plain copy of the path.
pub(crate) fn create_stroked_shape(path: &[PathOp]) -> Vec<PathOp> {
    path.to_vec()
}
