//! LRO, LVO and ICING: crosses, ovals and ticks along the line.

use super::super::inside_outside::get_inside_outside_double2;
use super::super::work::{Work, get, set};
use crate::engine::base::{EngineError, Pt};
use crate::engine::lineutility::basics::{calc_distance_double, calc_segment_angle_double};
use crate::engine::lineutility::extend::{
    extend_along_line_double_style, extend_along_line_double2, extend_directed_line,
    extend_directed_line_style,
};
use crate::engine::lineutility::transform::rotate_geometry_double;
use crate::engine::tactical_lines as lt;
use std::f64::consts::PI;

/// Upstream `GetXPoints`: the X marks along the line.
fn get_x_points(orig: &[Pt], segment_length: f64, vbl: i32) -> Result<Vec<Pt>, EngineError> {
    let mut out = Vec::new();
    let x_size = segment_length / 6.0;
    for j in 0..vbl - 1 {
        let (a, b) = (get(orig, j)?, get(orig, j + 1)?);
        let d = calc_distance_double(a, b);
        let num = ((d - segment_length) / segment_length) as i32;
        let dist_interval = d / f64::from(num);
        for k in 0..num {
            let pt0 =
                extend_along_line_double2(a, b, dist_interval / 2.0 + dist_interval * f64::from(k));
            let pt1 = extend_along_line_double2(pt0, b, x_size);
            let pt2 = extend_along_line_double2(pt0, b, -x_size);
            let pt3 = extend_directed_line(a, pt1, pt1, 2, x_size);
            let mut pt4 = extend_directed_line(a, pt1, pt1, 3, x_size);
            pt4.style = 5;
            let pt5 = extend_directed_line(a, pt2, pt2, 2, x_size);
            let mut pt6 = extend_directed_line(a, pt2, pt2, 3, x_size);
            pt6.style = 5;
            out.extend([pt3, pt6, pt5, pt4]);
        }
    }
    Ok(out)
}

/// One ellipse of 37 points around `center`, rotated by `angle` degrees.
fn oval(
    e: &mut [Pt],
    center: Pt,
    oval_width: f64,
    oval_length: f64,
    angle: i32,
) -> Result<(), EngineError> {
    for l in 1..37_i32 {
        let factor = (20.0 * f64::from(l)) * PI / 180.0;
        let p = super::super::work::at_mut(e, l - 1)?;
        p.x = center.x + f64::from((oval_width * factor.cos()) as i32);
        p.y = center.y + f64::from((oval_length * factor.sin()) as i32);
        p.style = 0;
    }
    rotate_geometry_double(e, 36, f64::from(angle))?;
    let mut last = get(e, 35)?;
    last.style = 5;
    set(e, 36, last)
}

/// Upstream `GetLVOPoints`: the ovals along the line and one at its end.
fn get_lvo_points(
    orig: &[Pt],
    oval_width: f64,
    segment_length: f64,
    vbl: i32,
) -> Result<Vec<Pt>, EngineError> {
    let oval_length = oval_width * 2.0;
    let mut out = Vec::new();
    let mut e = vec![Pt::default(); 37];
    for j in 0..vbl - 1 {
        e.iter_mut().for_each(|p| *p = Pt::default());
        let (a, b) = (get(orig, j)?, get(orig, j + 1)?);
        let d = calc_distance_double(a, b);
        let how_many = ((d - segment_length) / segment_length) as i32;
        let dist_interval = d / f64::from(how_many);
        let d_angle = calc_segment_angle_double(a, b) + PI / 2.0;
        let angle = (d_angle * 180.0 / PI) as i32;
        for k in 0..how_many {
            let center = extend_along_line_double2(a, b, f64::from(k) * dist_interval);
            oval(&mut e, center, oval_width, oval_length, angle)?;
            out.extend(e.iter().copied());
        }
        if j == vbl - 2 {
            oval(&mut e, b, oval_width, oval_length, angle)?;
            out.extend(e.iter().copied());
        }
    }
    Ok(out)
}

/// Upstream `GetIcingPointsDouble`.
fn get_icing_points_double(p: &mut [Pt], length: f64, vbl: i32) -> Result<i32, EngineError> {
    let orig: Vec<Pt> = (0..vbl).map(|j| get(p, j)).collect::<Result<_, _>>()?;
    let mut counter = 0;
    for j in 0..vbl - 1 {
        let (a, b) = (get(&orig, j)?, get(&orig, j + 1)?);
        let dist = calc_distance_double(a, b);
        let num_segments = (f64::from(dist as i32) / length) as i32;
        let dist_interval = dist / f64::from(num_segments);
        let direction = get_inside_outside_double2(a, b, &orig, vbl, j, lt::ICING)?;
        for k in 0..num_segments {
            let off = f64::from(k) * dist_interval;
            let pt0 = if k == 0 {
                a
            } else {
                extend_along_line_double_style(a, b, off, 0)
            };
            let pt1 = extend_along_line_double_style(a, b, off + length * 2.0 / 3.0, 5);
            let mid = extend_along_line_double_style(a, b, off + length / 3.0, 0);
            let pt2 = extend_directed_line_style(a, b, mid, direction, length / 3.0, 5);
            for (i, q) in [pt0, pt1, mid, pt2].into_iter().enumerate() {
                set(p, counter + i32::try_from(i).unwrap_or(0), q)?;
            }
            counter += 4;
        }
    }
    Ok(counter)
}

pub(super) fn lro(w: &mut Work<'_>) -> Result<(), EngineError> {
    let (s30, s4) = (w.scaled(30.0), w.scaled(4.0));
    let xs = get_x_points(&w.orig, s30, w.save)?;
    let lvo = get_lvo_points(&w.orig, s4, s30, w.save)?;
    let x_count = i32::try_from(xs.len()).unwrap_or(i32::MAX);
    for (k, q) in xs.iter().enumerate() {
        set(&mut w.p, i32::try_from(k).unwrap_or(0), *q)?;
    }
    if x_count > 0 {
        super::super::work::set_style(&mut w.p, x_count - 1, 5)?;
    }
    for (k, q) in lvo.iter().enumerate() {
        set(&mut w.p, x_count + i32::try_from(k).unwrap_or(0), *q)?;
    }
    w.ac = x_count + i32::try_from(lvo.len()).unwrap_or(0);
    Ok(())
}

pub(super) fn lvo(w: &mut Work<'_>) -> Result<(), EngineError> {
    let pts = get_lvo_points(&w.orig, w.scaled(4.0), w.scaled(20.0), w.save)?;
    for (k, q) in pts.iter().enumerate() {
        set(&mut w.p, i32::try_from(k).unwrap_or(0), *q)?;
    }
    w.ac = i32::try_from(pts.len()).unwrap_or(0);
    Ok(())
}

pub(super) fn icing(w: &mut Work<'_>) -> Result<(), EngineError> {
    let length = w.scaled(15.0);
    w.vbl = get_icing_points_double(&mut w.p, length, w.save)?;
    w.ac = w.vbl;
    Ok(())
}
