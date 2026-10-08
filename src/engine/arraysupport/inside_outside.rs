//! Port of `arraysupport.GetInsideOutsideDouble2`: which side of a segment
//! is the inside of the polyline, found by counting crossings of a ray from
//! the segment midpoint.

use crate::engine::base::EngineError;
use crate::engine::base::Pt;
use crate::engine::lineutility::slope::{calc_true_slope_double, reverse_direction};
use crate::engine::lineutility::{EXTEND_ABOVE, EXTEND_BELOW, EXTEND_LEFT, EXTEND_RIGHT};
use crate::engine::tactical_lines as lt;

use super::work::get;

/// Counts the segments (other than `index`) a horizontal ray from the left
/// crosses at the height `y0`, left of `x0`.
fn crossings_from_left(
    pts: &[Pt],
    vbl_counter: i32,
    index: i32,
    x0: f64,
    y0: f64,
) -> Result<i32, EngineError> {
    let mut n = 0;
    for j in 0..vbl_counter - 1 {
        if index == j {
            continue;
        }
        let (a, b) = (get(pts, j)?, get(pts, j + 1)?);
        let straddles = (a.y < y0 && b.y > y0)
            || (a.y > y0 && b.y < y0)
            || (a.y < y0 && b.y == y0)
            || (a.y == y0 && b.y < y0);
        if !straddles {
            continue;
        }
        let (vertical, m) = calc_true_slope_double(a, b);
        if vertical == 0 && a.x < x0 {
            n += 1;
        }
        if m != 0.0 && vertical == 1 {
            let intercept = a.y - m * a.x;
            let x = (y0 - intercept) / m;
            if x < x0 {
                n += 1;
            }
        }
    }
    Ok(n)
}

/// Counts the segments (other than `index`) a vertical ray from the top
/// crosses at `x0`, above `y0`.
fn crossings_from_top(
    pts: &[Pt],
    vbl_counter: i32,
    index: i32,
    x0: f64,
    y0: f64,
) -> Result<i32, EngineError> {
    let mut n = 0;
    for j in 0..vbl_counter - 1 {
        if index == j {
            continue;
        }
        let (a, b) = (get(pts, j)?, get(pts, j + 1)?);
        let straddles = (a.x < x0 && b.x > x0)
            || (a.x > x0 && b.x < x0)
            || (a.x < x0 && b.x == x0)
            || (a.x == x0 && b.x < x0);
        if !straddles {
            continue;
        }
        let (vertical, m) = calc_true_slope_double(a, b);
        if vertical == 1 && m == 0.0 && a.y < y0 {
            n += 1;
        }
        if m != 0.0 && vertical == 1 {
            let intercept = a.y - m * a.x;
            let y = m * x0 + intercept;
            if y < y0 {
                n += 1;
            }
        }
    }
    Ok(n)
}

/// Upstream `GetInsideOutsideDouble2`: the direction code (one of the
/// `EXTEND_*` constants) of the outside of the segment `pt0`-`pt1` of the
/// polyline `pts`; `index` is the segment's own index, which the ray ignores.
/// ICING reverses the answer.
pub(crate) fn get_inside_outside_double2(
    pt0: Pt,
    pt1: Pt,
    pts: &[Pt],
    vbl_counter: i32,
    index: i32,
    line_type: i32,
) -> Result<i32, EngineError> {
    let (vertical, m0) = calc_true_slope_double(pt0, pt1);
    let x0 = (pt0.x + pt1.x) / 2.0;
    let y0 = (pt0.y + pt1.y) / 2.0;
    let from_left = m0.abs() >= 1.0 || vertical == 0;
    let crossings = if from_left {
        crossings_from_left(pts, vbl_counter, index, x0, y0)?
    } else {
        crossings_from_top(pts, vbl_counter, index, x0, y0)?
    };
    let direction = match (crossings % 2, from_left) {
        (0, true) => EXTEND_LEFT,
        (0, false) => EXTEND_ABOVE,
        (_, true) => EXTEND_RIGHT,
        (_, false) => EXTEND_BELOW,
    };
    if line_type == lt::ICING {
        return Ok(reverse_direction(direction));
    }
    Ok(direction)
}
