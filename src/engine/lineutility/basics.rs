//! Port of the small point-array helpers of lineutility.java: distance,
//! centre, bounds, reversal, midpoint and quadrant.

use crate::engine::base::{At, EngineError, Pt, idx};

/// Upstream `ResizeArray`: the first `length` points, or the input unchanged
/// when it is not longer than `length`.
pub(crate) fn resize_array(pts: &[Pt], length: i32) -> Result<Vec<Pt>, EngineError> {
    let n = idx(length, pts.len())?;
    if pts.len() <= n {
        return Ok(pts.to_vec());
    }
    Ok(pts.iter().take(n).copied().collect())
}

/// Upstream `CalcSegmentAngleDouble`: the segment angle in radians.
pub(crate) fn calc_segment_angle_double(pt0: Pt, pt1: Pt) -> f64 {
    let (n_temp, m) = super::slope::calc_true_slope_double(pt0, pt1);
    if n_temp == 0 {
        std::f64::consts::FRAC_PI_2
    } else {
        m.atan()
    }
}

/// Min/max of the first `n` points' coordinates as (min x, min y, max x, max y).
fn bounds(pts: &[Pt], n: i32) -> Result<(f64, f64, f64, f64), EngineError> {
    let mut b = (
        f64::INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
    );
    for j in 0..n {
        let p = pts.at(idx(j, pts.len())?)?;
        b.0 = b.0.min(p.x);
        b.1 = b.1.min(p.y);
        b.2 = b.2.max(p.x);
        b.3 = b.3.max(p.y);
    }
    Ok(b)
}

/// Upstream `CalcCenterPointDouble`: centre of the bounding box of the first
/// `vbl_counter` points; keeps the first point's style and segment.
pub(crate) fn calc_center_point_double(pts: &[Pt], vbl_counter: i32) -> Result<Pt, EngineError> {
    let first = pts.at(0)?;
    let (min_x, min_y, max_x, max_y) = bounds(pts, vbl_counter)?;
    let (min_x, min_y) = (min_x.min(first.x), min_y.min(first.y));
    let (max_x, max_y) = (max_x.max(first.x), max_y.max(first.y));
    Ok(Pt {
        x: (min_x + max_x) / 2.0,
        y: (min_y + max_y) / 2.0,
        ..first
    })
}

/// Upstream `CalcCenterPointDouble2`: as [`calc_center_point_double`] but the
/// result is a fresh point with default style.
pub(crate) fn calc_center_point_double2(pts: &[Pt], vbl_counter: i32) -> Result<Pt, EngineError> {
    let c = calc_center_point_double(pts, vbl_counter)?;
    Ok(Pt::new(c.x, c.y))
}

/// Upstream `CalcDistanceDouble`: Euclidean distance; a zero or infinite
/// result falls back to the larger axis distance.
pub(crate) fn calc_distance_double(p1: Pt, p2: Pt) -> f64 {
    let mut r = ((p1.x - p2.x) * (p1.x - p2.x) + (p1.y - p2.y) * (p1.y - p2.y)).sqrt();
    let xdist = (p1.x - p2.x).abs();
    let ydist = (p1.y - p2.y).abs();
    let max = if ydist > xdist { ydist } else { xdist };
    if (r == 0.0 || r.is_infinite()) && max > 0.0 {
        r = max;
    }
    r
}

/// Upstream `ReversePointsDouble2`: reverses the first `vbl_counter` points.
pub(crate) fn reverse_points_double2(pts: &mut [Pt], vbl_counter: i32) -> Result<(), EngineError> {
    let n = idx(vbl_counter, pts.len())?;
    let len = pts.len();
    let head = pts.get_mut(..n).ok_or(EngineError::Index {
        index: i64::from(vbl_counter),
        len,
    })?;
    head.reverse();
    Ok(())
}

/// Upstream `GetPixelsMin`: the smallest x and y of the first `vbl_counter`
/// points.
pub(crate) fn get_pixels_min(pts: &[Pt], vbl_counter: i32) -> Result<(f64, f64), EngineError> {
    let b = bounds(pts, vbl_counter)?;
    Ok((b.0, b.1))
}

/// Upstream `MidPointDouble`: the midpoint carrying `styl`.
pub(crate) fn mid_point_double(pt0: Pt, pt1: Pt, styl: i32) -> Pt {
    Pt {
        x: (pt0.x + pt1.x) / 2.0,
        y: (pt0.y + pt1.y) / 2.0,
        style: styl,
        ..pt0
    }
}

/// Upstream `GetQuadrantDouble(POINT2, POINT2)`: 1 up-right, 2 down-right,
/// 3 down-left, 4 up-left (screen y grows downward); later tests win on ties.
pub(crate) fn get_quadrant_double(pt1: Pt, pt2: Pt) -> i32 {
    get_quadrant_double_xy(pt1.x, pt1.y, pt2.x, pt2.y)
}

/// Upstream `GetQuadrantDouble(double, double, double, double)`.
pub(crate) fn get_quadrant_double_xy(x1: f64, y1: f64, x2: f64, y2: f64) -> i32 {
    let mut q = 1;
    if x2 >= x1 && y2 <= y1 {
        q = 1;
    }
    if x2 >= x1 && y2 >= y1 {
        q = 2;
    }
    if x2 <= x1 && y2 >= y1 {
        q = 3;
    }
    if x2 <= x1 && y2 <= y1 {
        q = 4;
    }
    q
}
