//! Port of the arc helpers of lineutility.java: `ArcArrayDouble`,
//! `GetArcPointsDouble` and `CalcClockwiseCenterDouble`.

use super::basics::{calc_distance_double, get_pixels_min, get_quadrant_double};
use super::extend::extend_line_double;
use super::slope::calc_true_slope_double;
use crate::engine::base::{At, EngineError, Pt};
use crate::engine::tactical_lines as lt;
use std::f64::consts::PI;

/// Upstream `GetArcPointsDouble`: `num_segments + 1` points along the
/// shorter arc from `start` to `end` around `center`.
pub(crate) fn get_arc_points_double(start: Pt, end: Pt, center: Pt, num_segments: i32) -> Vec<Pt> {
    let dx_start = start.x - center.x;
    let dy_start = start.y - center.y;
    let dx_end = end.x - center.x;
    let dy_end = end.y - center.y;
    let radius = (dx_start * dx_start + dy_start * dy_start).sqrt();
    let angle_start = dy_start.atan2(dx_start);
    let angle_end = dy_end.atan2(dx_end);
    let mut sweep = angle_end - angle_start;
    while sweep > PI {
        sweep -= 2.0 * PI;
    }
    while sweep < -PI {
        sweep += 2.0 * PI;
    }
    let mut points = vec![start];
    for i in 1..=num_segments {
        let current = angle_start + (sweep * f64::from(i) / f64::from(num_segments));
        points.push(Pt::new(
            center.x + radius * current.cos(),
            center.y + radius * current.sin(),
        ));
    }
    points
}

/// Start and end angles of the arc drawn for `linetype`, given the base
/// direction `m` and whether the radius point lies left of the centre point.
fn arc_angles(linetype: i32, m: f64, a_left_of_e: bool) -> (f64, f64) {
    let deg = |d: f64| d * PI / 180.0;
    let (mut start, mut end) = match linetype {
        lt::CLUSTER => (m - deg(90.0), m - deg(90.0) + deg(180.0)),
        lt::ISOLATE | lt::CORDONKNOCK | lt::CORDONSEARCH | lt::DENY | lt::AREA_DEFENSE => {
            (m, m + deg(330.0))
        }
        lt::TURN_REVD | lt::TURN => (m, m + deg(90.0)),
        lt::OCCUPY | lt::RETAIN | lt::SECURE | lt::CONTROL | lt::LOCATE => (m, m + deg(338.0)),
        _ => (0.0, 2.0 * PI),
    };
    if a_left_of_e {
        match linetype {
            lt::ISOLATE | lt::CORDONKNOCK | lt::CORDONSEARCH | lt::DENY | lt::AREA_DEFENSE => {
                start = m - PI;
                end = start + deg(330.0);
            }
            lt::OCCUPY | lt::RETAIN | lt::SECURE | lt::CONTROL | lt::LOCATE => {
                start = m - PI;
                end = start + deg(338.0);
            }
            lt::TURN_REVD | lt::TURN => {
                start = m - PI;
                end = start + deg(90.0);
            }
            lt::CLUSTER => {
                start = m - PI + deg(90.0);
                end = start - deg(180.0);
            }
            _ => {}
        }
    }
    (start, end)
}

/// Upstream `ArcArrayDouble` without the geodesic converter: replaces the
/// first 26 points with an arc of radius `d_radius` around the centre derived
/// from `pts[0]` (the centre side) and `pts[1]` (the direction point).
/// Coordinates are truncated to whole pixels as upstream does.
pub(crate) fn arc_array_double(
    pts: &mut [Pt],
    d_radius: f64,
    linetype: i32,
) -> Result<(), EngineError> {
    let a = pts.at(1)?;
    let e = pts.at(0)?;
    let (_, slope) = calc_true_slope_double(a, e);
    let m = slope.atan();
    let length = calc_distance_double(a, e);
    let (startangle, endangle) = arc_angles(linetype, m, a.x < e.x);
    let numarcpts = 26;
    let increment = (endangle - startangle) / f64::from(numarcpts - 1);
    let mut c = e;
    if d_radius != 0.0 && length != 0.0 {
        c.x = (e.x - (d_radius / length) * (a.x - e.x)) as i32 as f64;
        c.y = (e.y - (d_radius / length) * (a.y - e.y)) as i32 as f64;
    }
    for j in 0..numarcpts {
        let ang = startangle + f64::from(j) * increment;
        let x = (d_radius * ang.cos()) as i32 as f64 + c.x;
        let y = (d_radius * ang.sin()) as i32 as f64 + c.y;
        *pts.at_mut(usize::try_from(j).unwrap_or(0))? = Pt::new(x, y);
    }
    Ok(())
}

/// Upstream `ArcArrayDouble` with its geographic converter, as the renderer
/// runs it: an arc around `pts[0]` through `pts[1]`, its angles taken as
/// azimuths clockwise from north and its points not truncated. The pixel
/// frame stands in for the converter's ground: over a graphic's extent it is
/// conformal, so azimuths and distances carry over.
pub(crate) fn arc_array_geographic(pts: &mut [Pt], linetype: i32) -> Result<(), EngineError> {
    let center = pts.at(0)?;
    let through = pts.at(1)?;
    let (dx, dy) = (through.x - center.x, through.y - center.y);
    // Azimuth from north, clockwise; pixel y grows southward.
    let mut m = dx.atan2(-dy);
    if m < 0.0 {
        m += PI;
    }
    let length = calc_distance_double(through, center);
    let (startangle, endangle) = arc_angles(linetype, m, through.x < center.x);
    let numarcpts = 26;
    let increment = (endangle - startangle) / f64::from(numarcpts - 1);
    for j in 0..numarcpts {
        let az = startangle + f64::from(j) * increment;
        let x = center.x + length * az.sin();
        let y = center.y - length * az.cos();
        *pts.at_mut(usize::try_from(j).unwrap_or(0))? = Pt::new(x, y);
    }
    Ok(())
}

/// Upstream `CalcClockwiseCenterDouble`: replaces `pts[0]` with the centre of
/// the clockwise arc through the two points and `pts[1]` with a point 50
/// pixels beyond `pts[0]` from the centre; returns the radius.
pub(crate) fn calc_clockwise_center_double(pts: &mut [Pt]) -> Result<f64, EngineError> {
    let pt1 = pts.at(0)?;
    let pt2 = pts.at(1)?;
    let mut c = pt1;
    let (mut offset_x, _) = get_pixels_min(&[pt1, pt2], 2)?;
    if offset_x < 0.0 {
        offset_x -= 100.0;
    } else {
        offset_x = 0.0;
    }
    let mid = Pt {
        x: (pt1.x + pt2.x) / 2.0,
        y: (pt1.y + pt2.y) / 2.0,
        ..pt1
    };
    let d_length = calc_distance_double(pt1, pt2);
    let d_radius = d_length / 2.0_f64.sqrt();
    let quadrant = get_quadrant_double(pt1, pt2);
    let (_, m) = calc_true_slope_double(pt1, pt2);
    let sign = if matches!(quadrant, 1 | 4) { 1.0 } else { -1.0 };
    if m != 0.0 {
        let b1 = mid.y + (1.0 / m) * mid.x;
        let b2 = (-1.0 / m) * offset_x + b1;
        let intercept = Pt {
            x: offset_x,
            y: b2,
            ..pt1
        };
        c = extend_line_double(intercept, mid, sign * d_length / 2.0);
    } else {
        c.x = mid.x;
        c.y = if pt1.x < pt2.x {
            mid.y + d_length / 2.0
        } else {
            mid.y - d_length / 2.0
        };
    }
    let e = extend_line_double(c, pt1, 50.0);
    *pts.at_mut(0)? = c;
    *pts.at_mut(1)? = e;
    Ok(d_radius)
}
