//! IL, PLANNED, ESR1, ESR2, FORDSITE, ROADBLK, PNO/PLD/CFL, FENCED and
//! FOXHOLE.

use crate::engine::arraysupport::work::{MAX_LENGTH, MIN_LENGTH, Work, get, set, set_style};
use crate::engine::base::{EngineError, Pt};
use crate::engine::lineutility::basics::{calc_distance_double, mid_point_double};
use crate::engine::lineutility::extend::{
    extend_angled_line, extend_directed_line, extend_line_double, extend_true_line_perp_double,
};
use crate::engine::lineutility::relative::line_relative_to_line;
use crate::engine::lineutility::slope::{
    calc_distance_to_line_double, calc_true_slope_double, line_of_x_points,
};
use crate::engine::tactical_lines as lt;

/// IL, PLANNED, ESR1, ESR2: the control line and its parallel through the
/// third point, as two segments.
pub(crate) fn relative_line(w: &mut Work<'_>) -> Result<(), EngineError> {
    let (mut pt0, mut pt1, mut pt2) = (w.pt0, w.pt1, w.pt2);
    let mut pt3 = get(&w.p, 0)?;
    let (p0, p1, p2) = (get(&w.p, 0)?, get(&w.p, 1)?, get(&w.p, 2)?);
    line_relative_to_line(p0, p1, p2, &mut pt0, &mut pt1);
    let d = calc_distance_double(p0, pt0);
    let pt4 = extend_line_double(pt0, p0, d);
    line_relative_to_line(p0, p1, pt4, &mut pt2, &mut pt3);
    set(&mut w.p, 0, pt0)?;
    set(&mut w.p, 1, pt1)?;
    set(&mut w.p, 2, pt3)?;
    set(&mut w.p, 3, pt2)?;
    match w.line_type {
        lt::IL | lt::ESR2 => {
            set_style(&mut w.p, 0, 0)?;
            set_style(&mut w.p, 1, 5)?;
            set_style(&mut w.p, 2, 0)?;
        }
        lt::PLANNED => {
            set_style(&mut w.p, 0, 1)?;
            set_style(&mut w.p, 1, 5)?;
            set_style(&mut w.p, 2, 1)?;
        }
        _ => {
            set_style(&mut w.p, 1, 5)?;
            let (y1, y2) = (get(&w.p, 1)?.y, get(&w.p, 2)?.y);
            let first_solid = if pt0.x <= pt1.x { y1 <= y2 } else { y1 >= y2 };
            let (s0, s2) = if first_solid { (0, 1) } else { (1, 0) };
            set_style(&mut w.p, 0, s0)?;
            set_style(&mut w.p, 2, s2)?;
        }
    }
    w.ac = 4;
    Ok(())
}

/// FORDSITE: the control line and its parallel through the third point.
pub(crate) fn ford_site(w: &mut Work<'_>) -> Result<(), EngineError> {
    let (mut pt0, mut pt1) = (w.pt0, w.pt1);
    let (p0, p1, p2) = (get(&w.p, 0)?, get(&w.p, 1)?, get(&w.p, 2)?);
    line_relative_to_line(p0, p1, p2, &mut pt0, &mut pt1);
    set_style(&mut w.p, 0, 1)?;
    set_style(&mut w.p, 1, 5)?;
    set(&mut w.p, 2, Pt { style: 1, ..pt0 })?;
    set(&mut w.p, 3, Pt { style: 5, ..pt1 })?;
    w.ac = 4;
    Ok(())
}

/// ROADBLK: a pair of perpendicular bars and two slanted ticks.
pub(crate) fn road_block(w: &mut Work<'_>) -> Result<(), EngineError> {
    let pts = [get(&w.p, 0)?, get(&w.p, 1)?, get(&w.p, 2)?, get(&w.p, 3)?];
    let (a, b) = (pts[0], pts[1]);
    let radius = calc_distance_double(a, b);
    let d = calc_distance_to_line_double(a, b, get(&w.p, 2)?);
    set(&mut w.p, 0, extend_true_line_perp_double(a, b, b, d, 0)?)?;
    set(&mut w.p, 1, extend_true_line_perp_double(a, b, a, d, 5)?)?;
    set(&mut w.p, 2, extend_true_line_perp_double(a, b, b, -d, 0)?)?;
    set(&mut w.p, 3, extend_true_line_perp_double(a, b, a, -d, 5)?)?;
    let mut mid = extend_line_double(a, mid_point_double(a, b, 0), d);
    set(
        &mut w.p,
        4,
        extend_angled_line(a, b, mid, 105.0, radius / 2.0),
    )?;
    let mut tick = extend_angled_line(a, b, mid, -75.0, radius / 2.0);
    tick.style = 5;
    set(&mut w.p, 5, tick)?;
    mid = extend_line_double(b, mid_point_double(a, b, 0), d);
    set(
        &mut w.p,
        6,
        extend_angled_line(a, b, mid, 105.0, radius / 2.0),
    )?;
    let mut tick = extend_angled_line(a, b, mid, -75.0, radius / 2.0);
    tick.style = 5;
    set(&mut w.p, 7, tick)?;
    w.ac = 8;
    Ok(())
}

/// PNO, PLD, CFL: every point dashed.
pub(crate) fn dashed(w: &mut Work<'_>) -> Result<(), EngineError> {
    for j in 0..w.vbl {
        set_style(&mut w.p, j, 1)?;
    }
    w.ac = w.vbl;
    Ok(())
}

/// FENCED: the control points, then a line of X marks along them.
pub(crate) fn fenced(w: &mut Work<'_>) -> Result<(), EngineError> {
    let mut points = (0..w.vbl)
        .map(|j| get(&w.p, j))
        .collect::<Result<Vec<_>, _>>()?;
    points.extend(line_of_x_points(w.scaled(5.0), &w.orig)?);
    w.ac = i32::try_from(points.len()).map_err(|_| EngineError::Degenerate("too many points"))?;
    w.points = points;
    Ok(())
}

/// The side of the segment the foxhole bulges to.
fn foxhole_direction(pt0: Pt, pt1: Pt) -> i32 {
    let (vertical, m) = calc_true_slope_double(pt0, pt1);
    let mut direction = 0;
    if vertical == 0 {
        direction = if pt0.y > pt1.y { 0 } else { 1 };
    }
    if vertical != 0 && m <= 1.0 {
        direction = if pt0.x < pt1.x { 3 } else { 2 };
    }
    if vertical != 0 && m > 1.0 {
        if pt0.x < pt1.x && pt0.y > pt1.y {
            direction = 1;
        }
        if pt0.x < pt1.x && pt0.y < pt1.y {
            direction = 0;
        }
        if pt0.x > pt1.x && pt0.y > pt1.y {
            direction = 1;
        }
        if pt0.x > pt1.x && pt0.y < pt1.y {
            direction = 0;
        }
    }
    direction
}

/// FOXHOLE: the control segment with a short leg at each end.
pub(crate) fn foxhole(w: &mut Work<'_>) -> Result<(), EngineError> {
    let (pt0, pt1) = (w.pt0, w.pt1);
    let direction = foxhole_direction(pt0, pt1);
    let mut d_mbr = w.d_mbr;
    if d_mbr / 20.0 > MAX_LENGTH * w.dpi {
        d_mbr = 20.0 * MAX_LENGTH * w.dpi;
    }
    if d_mbr / 20.0 < MIN_LENGTH * w.dpi {
        d_mbr = 20.0 * MIN_LENGTH * w.dpi;
    }
    if d_mbr < 250.0 * w.dpi {
        d_mbr = 250.0 * w.dpi;
    }
    if d_mbr > 500.0 * w.dpi {
        d_mbr = 500.0 * w.dpi;
    }
    set(
        &mut w.p,
        0,
        extend_directed_line(pt0, pt1, pt0, direction, d_mbr / 20.0),
    )?;
    set(&mut w.p, 1, pt0)?;
    set(&mut w.p, 2, pt1)?;
    set(
        &mut w.p,
        3,
        extend_directed_line(pt0, pt1, pt1, direction, d_mbr / 20.0),
    )?;
    w.ac = 4;
    Ok(())
}
