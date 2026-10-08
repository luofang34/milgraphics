//! REEF, RESTRICTED_AREA and PIPE: patterns repeated along each segment.

use super::super::work::{Work, get, scaled_size, set, set_style};
use crate::engine::base::{EngineError, Pt};
use crate::engine::lineutility::basics::{calc_distance_double, mid_point_double};
use crate::engine::lineutility::extend::{extend_along_line_double2, extend_directed_line};

/// A copy of the first `vbl` points.
fn originals(p: &[Pt], vbl: i32) -> Result<Vec<Pt>, EngineError> {
    (0..vbl).map(|j| get(p, j)).collect()
}

/// Upstream `GetReefPoints`.
fn get_reef_points(p: &mut [Pt], length: f64, vbl: i32) -> Result<i32, EngineError> {
    let orig = originals(p, vbl)?;
    let mut counter = 0;
    for j in 0..vbl - 1 {
        let (a, b) = (get(&orig, j)?, get(&orig, j + 1)?);
        let direction = if a.x < b.x { 2 } else { 3 };
        let num_segs = (calc_distance_double(a, b) / length) as i32;
        for k in 0..num_segs {
            let pt0 = extend_along_line_double2(a, b, length * f64::from(k));
            let pt1 = extend_along_line_double2(pt0, b, length * 0.35);
            let pt1 = extend_directed_line(a, b, pt1, direction, length);
            let pt2 = extend_along_line_double2(pt0, b, length * 0.4);
            let pt2 = extend_directed_line(a, b, pt2, direction, length * 0.6);
            let pt3 = extend_along_line_double2(pt0, b, length * 0.75);
            let pt3 = extend_directed_line(a, b, pt3, direction, length * 1.35);
            let pt4 = extend_along_line_double2(a, b, length * f64::from(k + 1));
            for q in [pt0, pt1, pt2, pt3, pt4] {
                set(p, counter, q)?;
                counter += 1;
            }
        }
        if num_segs == 0 {
            set(p, counter, a)?;
            set(p, counter + 1, b)?;
            counter += 2;
        }
    }
    set(p, counter, get(&orig, vbl - 1)?)?;
    Ok(counter + 1)
}

/// Upstream `GetRestrictedAreaPoints`.
fn get_restricted_area_points(p: &mut [Pt], length: f64, vbl: i32) -> Result<i32, EngineError> {
    let orig = originals(p, vbl)?;
    let mut counter = 0;
    for j in 0..vbl - 1 {
        let (a, b) = (get(&orig, j)?, get(&orig, j + 1)?);
        let num_segs = (calc_distance_double(a, b) / length) as i32;
        let direction = if a.x < b.x { 3 } else { 2 };
        for k in 0..num_segs {
            let mut pt0 = extend_along_line_double2(a, b, length * f64::from(k));
            pt0.style = 0;
            let mut pt1 =
                extend_along_line_double2(a, b, length * f64::from(k) + length * 2.0 / 3.0);
            pt1.style = 5;
            let pt2 = mid_point_double(pt0, pt1, 0);
            let mut pt3 = extend_directed_line(a, b, pt2, direction, length * 2.0 / 3.0);
            pt3.style = 5;
            for q in [pt2, pt3, pt0, pt1] {
                set(p, counter, q)?;
                counter += 1;
            }
        }
        if num_segs == 0 {
            set(p, counter, a)?;
            set(p, counter + 1, b)?;
            counter += 2;
        }
    }
    set_style(p, counter - 1, 0)?;
    set(p, counter, get(&orig, vbl - 1)?)?;
    Ok(counter + 1)
}

/// Upstream `GetPipePoints`. Upstream's last store reads one slot past
/// itself and throws when there is none; the exception is caught inside
/// the method, so the count (already advanced) is still returned.
fn get_pipe_points(p: &mut [Pt], length: f64, vbl: i32) -> Result<i32, EngineError> {
    let orig = originals(p, vbl)?;
    let mut x_points: Vec<Pt> = Vec::new();
    let mut counter = 0;
    for j in 0..vbl - 1 {
        let (a, b) = (get(&orig, j)?, get(&orig, j + 1)?);
        let num_segs = (calc_distance_double(a, b) / length) as i32;
        for k in 0..num_segs {
            let mut pt0 = extend_along_line_double2(a, b, length * f64::from(k));
            pt0.style = 0;
            let mut pt1 = extend_along_line_double2(a, b, length * f64::from(k) + length / 2.0);
            pt1.style = 5;
            let mut pt2 = extend_along_line_double2(a, b, length * f64::from(k) + length / 2.0);
            pt2.style = 20;
            set(p, counter, pt0)?;
            set(p, counter + 1, pt1)?;
            counter += 2;
            x_points.push(pt2);
        }
        if num_segs == 0 {
            set(p, counter, Pt { style: 0, ..a })?;
            set(p, counter + 1, Pt { style: 5, ..b })?;
        } else {
            set(
                p,
                counter,
                Pt {
                    style: 0,
                    ..get(p, counter - 1)?
                },
            )?;
            set(p, counter + 1, Pt { style: 5, ..b })?;
        }
        counter += 2;
    }
    for q in &x_points {
        set(p, counter, *q)?;
        counter += 1;
    }
    let old = counter;
    counter += 1;
    match get(p, counter) {
        Ok(next) => set(p, old, next)?,
        Err(_) => return Ok(counter),
    }
    Ok(counter)
}

pub(super) fn reef(w: &mut Work<'_>) -> Result<(), EngineError> {
    w.vbl = get_reef_points(&mut w.p, scaled_size(w.tg, 40.0), w.save)?;
    w.ac = w.vbl;
    Ok(())
}

pub(super) fn restricted_area(w: &mut Work<'_>) -> Result<(), EngineError> {
    w.vbl = get_restricted_area_points(&mut w.p, scaled_size(w.tg, 15.0), w.save)?;
    w.ac = w.vbl;
    Ok(())
}

pub(super) fn pipe(w: &mut Work<'_>) -> Result<(), EngineError> {
    w.vbl = get_pipe_points(&mut w.p, scaled_size(w.tg, 20.0), w.save)?;
    w.ac = w.vbl;
    Ok(())
}
