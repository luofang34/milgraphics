//! DELAY, WITHDRAW, RETIRE, PURSUIT, ENVELOPMENT and INFILTRATION from
//! DISMSupport.java: a line with a semicircular arc and an arrowhead.

use super::support::{
    arc_approximation_double, clamp_size, draw_endpiece_deltas_double, put, reverse_delay_arc,
    set_style,
};
use crate::engine::base::{At, EngineError, Pt};
use crate::engine::lineutility::arc::{calc_clockwise_center_double, get_arc_points_double};
use crate::engine::lineutility::basics::calc_distance_double;
use crate::engine::lineutility::extend::{extend_along_line_double_style, extend_line2_double};
use crate::engine::lineutility::relative::{closest_point_on_line, point_relative_to_line_at};
use crate::engine::settings::Settings;
use crate::engine::tactical_lines as lt;
use std::f64::consts::PI;

fn dist(a: Pt, b: Pt) -> f64 {
    ((b.x - a.x) * (b.x - a.x) + (b.y - a.y) * (b.y - a.y)).sqrt()
}

/// The arrowhead barbs for an arrow tip at `tip` pointing along `angle`.
fn arrow_barbs(tip: Pt, angle: f64, size: f64) -> [Pt; 4] {
    let dx1 = size * (angle - PI / 4.0).cos();
    let dy1 = size * (angle - PI / 4.0).sin();
    let dx2 = size * (angle + PI / 4.0).cos();
    let dy2 = size * (angle + PI / 4.0).sin();
    draw_endpiece_deltas_double(tip, dx1, dy1, dx2, dy2)
}

fn push_arc(points: &mut Vec<Pt>, counter: &mut usize, arc: &[Pt; 17]) {
    for p in arc {
        let mut q = *p;
        q.style = 0;
        put(points, *counter, q);
        *counter += 1;
    }
    if let Some(last) = counter.checked_sub(1) {
        if let Some(p) = points.get_mut(last) {
            p.style = 5;
        }
    }
}

/// Upstream `GetDelayGraphicEtcDouble`: DELAY, WITHDRAW, DISENGAGE, WDRAWUP,
/// RETIRE, FPOL, RPOL and PURSUIT.
pub(crate) fn get_delay_graphic_etc_double(
    points: &mut Vec<Pt>,
    line_type: i32,
    settings: &Settings,
) -> Result<i32, EngineError> {
    let p0 = points.at(0)?;
    let p1 = points.at(1)?;
    let p2 = point_relative_to_line_at(p0, p1, p1, points.at(2)?);
    put(points, 2, p2);
    let save = [p0, p1, p2];
    let mut first = p0;
    first.style = 14;
    put(points, 0, first);
    let mut second = p1;
    second.style = 5;
    put(points, 1, second);
    let mut counter = 2;

    let length = dist(save[0], save[1]);
    let radius = dist(save[1], save[2]) / 2.0;
    let pursuit = line_type == lt::PURSUIT;
    let mut angle = if pursuit {
        (p2.y - p1.y).atan2(p2.x - p1.x)
    } else {
        (p1.y - p0.y).atan2(p1.x - p0.x)
    };
    let diag = clamp_size((length + radius * 2.0) / 20.0, settings.dpi_scale_factor());
    let center = Pt::new((save[1].x + save[2].x) / 2.0, (save[1].y + save[2].y) / 2.0);
    let bounds = (
        center.x - radius,
        center.y - radius,
        center.x + radius,
        center.y + radius,
    );
    let reverse = reverse_delay_arc(&save)?;
    let (a, b) = if reverse {
        (save[2], save[1])
    } else {
        (save[1], save[2])
    };
    let arc = arc_approximation_double(bounds, (a.x, a.y), (b.x, b.y));
    if pursuit {
        angle += if reverse { -PI / 2.0 } else { PI / 2.0 };
    }
    let tip = if pursuit { save[2] } else { save[0] };
    for p in arrow_barbs(tip, angle, diag) {
        put(points, counter, p);
        counter += 1;
    }
    push_arc(points, &mut counter, &arc);
    if pursuit {
        for d in [radius * 2.0 - diag, radius * 2.0 + diag] {
            put(
                points,
                counter,
                extend_along_line_double_style(save[1], save[2], d, 0),
            );
            counter += 1;
        }
    }
    Ok(counter as i32)
}

fn is_envelopment_arc_reversed(points: &[Pt]) -> Result<bool, EngineError> {
    if points.len() < 4 {
        return Ok(false);
    }
    let mut seize = [points.at(1)?, points.at(2)?];
    calc_clockwise_center_double(&mut seize)?;
    let d = calc_distance_double(seize.at(0)?, points.at(3)?);
    let mut seize = [points.at(2)?, points.at(1)?];
    calc_clockwise_center_double(&mut seize)?;
    let reversed = calc_distance_double(seize.at(0)?, points.at(3)?);
    Ok(reversed > d)
}

/// Upstream `GetEnvelopmentGraphicDouble`: the ENVELOPMENT arrow.
pub(crate) fn get_envelopment_graphic_double(
    points: &mut Vec<Pt>,
    settings: &Settings,
) -> Result<i32, EngineError> {
    let reverse = is_envelopment_arc_reversed(points)?;
    set_style(points, 0, 14)?;
    set_style(points, 1, 5)?;
    let mut counter = 2;
    let (p0, p1) = (points.at(0)?, points.at(1)?);
    let reach = extend_line2_double(p0, p1, calc_distance_double(p1, points.at(2)?), 0);
    let p2 = closest_point_on_line(p0, reach, points.at(2)?);
    put(points, 2, p2);

    let length = dist(p0, p1);
    let radius = dist(p1, p2) / 2.0;
    let mut angle = (p2.y - p1.y).atan2(p2.x - p1.x);
    let diag = clamp_size((length + radius * 2.0) / 20.0, settings.dpi_scale_factor());
    let center = Pt::new((p1.x + p2.x) / 2.0, (p1.y + p2.y) / 2.0);
    let bounds = (
        center.x - radius,
        center.y - radius,
        center.x + radius,
        center.y + radius,
    );
    let (a, b) = if reverse { (p1, p2) } else { (p2, p1) };
    let arc = arc_approximation_double(bounds, (a.x, a.y), (b.x, b.y));
    angle += if reverse { PI / 2.0 } else { -PI / 2.0 };
    for p in arrow_barbs(p2, angle, diag) {
        put(points, counter, p);
        counter += 1;
    }
    push_arc(points, &mut counter, &arc);
    Ok(counter as i32)
}

/// Upstream `GetInfiltrationDouble`: the S-shaped INFILTRATION path through
/// the three control points. The result has three extra trailing slots (null
/// upstream) for the caller to overwrite with the arrowhead.
pub(crate) fn get_infiltration_double(
    points: &[Pt],
    settings: &Settings,
) -> Result<Vec<Pt>, EngineError> {
    let mut p1 = points.at(0)?;
    let p3 = points.at(1)?;
    let mut p5 = points.at(2)?;
    p1.style = 0;
    let dpi = f64::from(settings.dpi);
    if p5.x == 0.0 && p5.y == 0.0 {
        p5.x = p3.x;
        p5.y = p3.y;
    }
    let d1 = calc_distance_double(p1, p3) as i32;
    let d2 = calc_distance_double(p3, p5) as i32;
    if d1 == 0 {
        p1.x -= 1.0;
        p1.y -= 1.0;
    }
    if d2 == 0 {
        p5.x += 1.0;
        p5.y += 1.0;
    }
    let mut arc_distance = dpi / 4.0;
    let mut p2 = Pt::styled(0.0, 0.0, 1);
    let mut p4 = Pt::styled(0.0, 0.0, 1);
    if p5.y >= p3.y {
        p2.y = p3.y - arc_distance;
        p4.y = p3.y + arc_distance;
    } else {
        p2.y = p3.y + arc_distance;
        p4.y = p3.y - arc_distance;
    }
    arc_distance = dpi / 4.0;
    if f64::from(d2) < arc_distance {
        arc_distance = (f64::from(d2) * 0.5).max(3.0);
    }
    if p5.x >= p3.x {
        p2.x = p3.x - arc_distance;
        p4.x = p3.x + arc_distance;
    } else {
        p2.x = p3.x + arc_distance;
        p4.x = p3.x - arc_distance;
    }
    let anchor1 = Pt::new(p2.x, p3.y);
    let anchor2 = Pt::new(p4.x, p3.y);
    let mut path = vec![p1];
    if arc_distance < f64::from(d1) {
        path.extend(get_arc_points_double(p2, p3, anchor1, 5));
        path.pop();
    }
    if arc_distance < f64::from(d2) {
        path.extend(get_arc_points_double(p3, p4, anchor2, 5));
    } else {
        path.push(p3);
    }
    path.push(p5);
    for (i, p) in path.iter_mut().enumerate() {
        p.style = i32::from(i != 0);
    }
    let extra = path.len() + 3;
    path.resize(extra, Pt::default());
    Ok(path)
}
