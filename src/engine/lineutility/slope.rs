//! Port of the slope, line-equation and intersection helpers of
//! lineutility.java.

use super::basics::calc_distance_double;
use super::extend::extend_along_line_double;
use crate::engine::base::{At, EngineError, Pt};

/// Upstream `CalcTrueSlopeDouble`: `(1, slope)`. A nearly vertical run
/// (|dx| < 1) is widened to dx = +/-1, so the status is always 1.
pub(crate) fn calc_true_slope_double(first: Pt, last: Pt) -> (i32, f64) {
    let mut delta_x = first.x - last.x;
    if delta_x.abs() < 1.0 {
        delta_x = if delta_x >= 0.0 { 1.0 } else { -1.0 };
    }
    let delta_y = first.y - last.y;
    (1, delta_y / delta_x)
}

/// Upstream `CalcTrueSlopeDoubleForRoutes`: `(false, 0.0)` when |dx| < 2.
pub(crate) fn calc_true_slope_double_for_routes(first: Pt, last: Pt) -> (bool, f64) {
    let delta_x = first.x - last.x;
    if delta_x.abs() < 2.0 {
        return (false, 0.0);
    }
    (true, (first.y - last.y) / delta_x)
}

/// Upstream `CalcTrueSlopeDouble2`: the slope, and false when the run was
/// nearly vertical (|dx| < 1) and had to be widened.
pub(crate) fn calc_true_slope_double2(first: Pt, last: Pt) -> (bool, f64) {
    let mut delta_x = first.x - last.x;
    let mut ok = true;
    if delta_x.abs() < 1.0 {
        delta_x = if delta_x >= 0.0 { 1.0 } else { -1.0 };
        ok = false;
    }
    (ok, (first.y - last.y) / delta_x)
}

/// Upstream `CalcTrueLinesDouble`: `(1, [m, b, m, b + delta, m, b - delta])`
/// for the line and its two parallels `n_distance` away.
pub(crate) fn calc_true_lines_double(
    n_distance: i64,
    line_point1: Pt,
    line_point2: Pt,
) -> (i32, [f64; 6]) {
    let (n_temp, m) = calc_true_slope_double(line_point1, line_point2);
    let mut r = [0.0; 6];
    let d = n_distance as f64;
    if n_temp == 0 {
        r[3] = line_point1.x + d;
        r[5] = line_point1.x - d;
        return (0, r);
    }
    let b = line_point2.y - m * line_point2.x;
    let delta = (m * m * (d * d) + d * d).sqrt();
    r[0] = m;
    r[1] = b;
    r[2] = m;
    r[3] = b + delta;
    r[4] = m;
    r[5] = b - delta;
    (1, r)
}

/// Upstream `CalcTrueIntersectDouble2`: intersection of two lines given as
/// slope/intercept, or as x = X when the matching `bol_vertical` flag is 0.
/// `(x1, x2)` are upstream's last two parameters, grouped to keep the argument count down.
/// Parallel lines return `(x1, x2)` as the point, as upstream does.
pub(crate) fn calc_true_intersect_double2(
    m1: f64,
    b1: f64,
    m2: f64,
    b2: f64,
    bol_vertical1: i32,
    bol_vertical2: i32,
    (x1, x2): (f64, f64),
) -> Pt {
    let mut p = Pt::new(x1, x2);
    if bol_vertical1 == 0 && bol_vertical2 == 0 {
        return p;
    }
    if bol_vertical1 == 0 && bol_vertical2 == 1 {
        p.x = x1;
        p.y = m2 * x1 + b2;
        return p;
    }
    if bol_vertical1 == 1 && bol_vertical2 == 0 {
        p.x = x2;
        p.y = m1 * x2 + b1;
        return p;
    }
    if m1 != m2 {
        let x = (b2 - b1) / (m1 - m2);
        p.x = x;
        p.y = m1 * x + b1;
    }
    p
}

/// Upstream `GetOffsetPointDouble`: the point `n_offset` pixels beyond the
/// end point along the start-to-end direction.
pub(crate) fn get_offset_point_double(start: Pt, end: Pt, n_offset: i64) -> Pt {
    let mut t = start;
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let off = n_offset as f64;
    if dx == 0.0 {
        t.x = end.x;
        t.y = if dy > 0.0 { end.y + off } else { end.y - off };
        return t;
    }
    if dy == 0.0 {
        t.x = if dx > 0.0 { end.x + off } else { end.x - off };
        t.y = end.y;
        return t;
    }
    let angle = (dx / dy).atan() + std::f64::consts::FRAC_PI_2;
    t.x = if end.x > start.x {
        end.x + off * angle.cos().abs()
    } else {
        end.x - off * angle.cos().abs()
    };
    t.y = if end.y > start.y {
        end.y + off * angle.sin().abs()
    } else {
        end.y - off * angle.sin().abs()
    };
    t
}

/// Upstream `LineOfXPoints` (used for DMAF). The caller supplies the X size
/// that upstream derives from `getScaledSize(5, thickness, patternScale)`.
pub(crate) fn line_of_x_points(x_size: f64, pts: &[Pt]) -> Result<Vec<Pt>, EngineError> {
    use super::extend::extend_directed_line;
    let mut out = Vec::new();
    let d_increment = x_size * 4.0;
    for j in 0..pts.len().saturating_sub(1) {
        let (a, b) = (pts.at(j)?, pts.at(j + 1)?);
        let dist = calc_distance_double(a, b);
        let mut iterations = ((dist - x_size) / d_increment) as i32;
        if dist - f64::from(iterations) * d_increment > d_increment / 2.0 {
            iterations += 1;
        }
        for k in 0..iterations {
            let kd = f64::from(k) * d_increment;
            let front = extend_along_line_double(a, b, kd - x_size);
            let back = extend_along_line_double(a, b, kd + x_size);
            let front_above = extend_directed_line(a, b, front, 2, x_size);
            let mut front_below = extend_directed_line(a, b, front, 3, x_size);
            let back_above = extend_directed_line(a, b, back, 2, x_size);
            let mut back_below = extend_directed_line(a, b, back, 3, x_size);
            back_below.style = 5;
            front_below.style = 5;
            out.extend([front_above, back_below, back_above, front_below]);
        }
    }
    Ok(out)
}

/// Upstream `CalcDistanceToLineDouble`: distance from `pt3` to the line
/// through `pt1` and `pt2`.
pub(crate) fn calc_distance_to_line_double(pt1: Pt, pt2: Pt, pt3: Pt) -> f64 {
    let mut intersect = pt1;
    let (bol_vertical, m) = calc_true_slope_double(pt1, pt2);
    if bol_vertical != 0 && m != 0.0 {
        let m1 = -1.0 / m;
        let b = pt1.y - m * pt1.x;
        let b1 = pt3.y - m1 * pt3.x;
        intersect = calc_true_intersect_double2(m, b, m1, b1, 1, 1, (intersect.x, intersect.y));
    }
    if bol_vertical != 0 && m == 0.0 {
        intersect.y = pt1.y;
        intersect.x = pt3.x;
    }
    if bol_vertical == 0 {
        intersect.y = pt3.y;
        intersect.x = pt1.x;
    }
    calc_distance_double(pt3, intersect)
}

/// Upstream `CalcDirectionFromLine`: 0 left, 1 right, 2 above, 3 below.
pub(crate) fn calc_direction_from_line(pt0: Pt, pt1: Pt, pt_relative: Pt) -> i32 {
    if pt0.x == pt1.x {
        return i32::from(pt_relative.x >= pt0.x);
    }
    if pt0.y == pt1.y {
        return if pt_relative.y < pt0.y { 2 } else { 3 };
    }
    let (_, m1) = calc_true_slope_double(pt0, pt1);
    let m2 = -1.0 / m1;
    let b1 = pt0.y - m1 * pt0.x;
    let b2 = pt_relative.y - m2 * pt_relative.x;
    let intersect = calc_true_intersect_double2(m1, b1, m2, b2, 1, 1, (0.0, 0.0));
    if m1 > 1.0 {
        i32::from(pt_relative.x >= intersect.x)
    } else if pt_relative.y < intersect.y {
        2
    } else {
        3
    }
}

/// Upstream `reverseDirection`: swaps left/right and above/below; other
/// codes pass through.
pub(crate) fn reverse_direction(direction: i32) -> i32 {
    match direction {
        0 => 1,
        1 => 0,
        2 => 3,
        3 => 2,
        d => d,
    }
}
