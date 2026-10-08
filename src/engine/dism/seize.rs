//! SEIZE, CAPTURE and EVACUATE from DISMSupport.java: a circle at the first
//! control point and an arc with an arrowhead sweeping to the second.

use super::support::{MAX_LENGTH, arc_approximation_double, put};
use crate::engine::base::{At, EngineError, Pt};
use crate::engine::lineutility::basics::calc_distance_double;
use crate::engine::lineutility::relative::intersect_polygon;
use crate::engine::settings::Settings;
use std::f64::consts::PI;

/// Upstream's `sqrt(2)` approximation used for the arc radius.
const SQRT2_APPROX: f64 = 1_414_213_562_373.0 / 1_000_000_000_000.0;

fn styled(p: Pt, style: i32) -> Pt {
    let mut q = p;
    q.style = style;
    q
}

fn is_seize_arc_reversed(p: &[Pt]) -> Result<bool, EngineError> {
    let (p0, p1, p2) = (p.at(0)?, p.at(1)?, p.at(2)?);
    let angle = (p0.y - p1.y).atan2(p0.x - p1.x);
    let (dx1, dy1) = ((angle + PI / 4.0).cos(), (angle + PI / 4.0).sin());
    let (dx2, dy2) = ((angle - PI / 4.0).cos(), (angle - PI / 4.0).sin());
    let chord = ((p1.x - p0.x) * (p1.x - p0.x) + (p1.y - p0.y) * (p1.y - p0.y)).sqrt();
    let radius = chord / SQRT2_APPROX;
    let center = Pt::new(p0.x - dx1 * radius, p0.y - dy1 * radius);
    let d = calc_distance_double(center, p2);
    let center_rev = Pt::new(p0.x - dx2 * radius, p0.y - dy2 * radius);
    let d_rev = calc_distance_double(center_rev, p2);
    Ok(d_rev > d)
}

fn arc_bounds(center: Pt, radius: f64) -> (f64, f64, f64, f64) {
    (
        center.x - radius,
        center.y - radius,
        center.x + radius,
        center.y + radius,
    )
}

/// The arc points kept outside the circle, ending where the arc leaves it.
fn forward_arc_points(circle: &[Pt], arc: &[Pt; 17], center: Pt, circle_radius: f64) -> Vec<Pt> {
    let mut out = Vec::new();
    let mut prev: Option<Pt> = None;
    for (j, p) in arc.iter().enumerate() {
        if calc_distance_double(center, *p) >= circle_radius {
            out.push(styled(*p, 0));
        } else if let Some(before) = prev {
            let hit = intersect_polygon(circle, *p, before).unwrap_or(*p);
            out.extend(std::iter::repeat_n(styled(hit, 0), 17 - j));
            return out;
        } else {
            return arc.iter().map(|q| styled(*q, 0)).collect();
        }
        prev = Some(*p);
    }
    out
}

/// The arc points kept outside the circle, starting where the arc leaves it.
fn reversed_arc_points(circle: &[Pt], arc: &[Pt; 17], center: Pt, circle_radius: f64) -> Vec<Pt> {
    let mut out = Vec::new();
    let mut outside = false;
    let mut prev: Option<Pt> = None;
    for (j, p) in arc.iter().enumerate() {
        if outside || calc_distance_double(center, *p) >= circle_radius {
            if let (false, Some(before)) = (outside, prev) {
                let hit = intersect_polygon(circle, *p, before).unwrap_or(before);
                out.extend(std::iter::repeat_n(styled(hit, 0), j));
            }
            out.push(styled(*p, 0));
            outside = true;
        }
        prev = Some(*p);
    }
    if !outside {
        return arc.iter().map(|q| styled(*q, 0)).collect();
    }
    out
}

/// The arc from the second control point toward the circle, with the points
/// inside the circle dropped.
fn seize_arc_points(
    sp: &[Pt; 3],
    circle: &[Pt; 17],
    circle_radius: f64,
    reversed: bool,
) -> Vec<Pt> {
    let angle = (sp[0].y - sp[1].y).atan2(sp[0].x - sp[1].x);
    let d1 = ((angle + PI / 4.0).cos(), (angle + PI / 4.0).sin());
    let d2 = ((angle - PI / 4.0).cos(), (angle - PI / 4.0).sin());
    let chord = calc_chord(sp[0], sp[1]);
    let arc_radius = chord / SQRT2_APPROX;
    let (start_d, center_d) = if reversed { (d1, d2) } else { (d2, d1) };
    let arc_start = Pt::new(
        sp[0].x - start_d.0 * circle_radius,
        sp[0].y - start_d.1 * circle_radius,
    );
    let center = Pt::new(
        sp[0].x - center_d.0 * arc_radius,
        sp[0].y - center_d.1 * arc_radius,
    );
    let bounds = arc_bounds(center, arc_radius);
    if reversed {
        let arc = arc_approximation_double(bounds, (arc_start.x, arc_start.y), (sp[1].x, sp[1].y));
        reversed_arc_points(circle, &arc, sp[0], circle_radius)
    } else {
        let arc = arc_approximation_double(bounds, (sp[1].x, sp[1].y), (arc_start.x, arc_start.y));
        forward_arc_points(circle, &arc, sp[0], circle_radius)
    }
}

fn calc_chord(a: Pt, b: Pt) -> f64 {
    ((b.x - a.x) * (b.x - a.x) + (b.y - a.y) * (b.y - a.y)).sqrt()
}

/// The arrowhead at the end of the arc.
fn seize_arrow_head(sp: &[Pt; 3], reversed: bool, settings: &Settings) -> [Pt; 3] {
    let angle = (sp[0].y - sp[1].y).atan2(sp[0].x - sp[1].x);
    let chord = calc_chord(sp[0], sp[1]);
    let mut factor = 1.0;
    let max = MAX_LENGTH * settings.dpi_scale_factor();
    if chord / 8.0 > max {
        factor = chord / (8.0 * max);
    }
    if factor == 0.0 {
        factor = 1.0;
    }
    let sign = if reversed { -1.0 } else { 1.0 };
    [
        Pt::new(
            sp[1].x - (sp[1].x - sp[0].x) / (8.0 * factor),
            sp[1].y - (sp[1].y - sp[0].y) / (8.0 * factor),
        ),
        sp[1],
        Pt::new(
            sp[1].x + (angle + sign * PI / 2.0).cos() * (chord / 8.0) / factor,
            sp[1].y + (angle + sign * PI / 2.0).sin() * (chord / 8.0) / factor,
        ),
    ]
}

/// Upstream `GetDISMSeizeDouble`. `radius` is the circle radius from a
/// four-point definition, or 0 to derive it from the control points.
pub(crate) fn get_dism_seize_double(
    points: &mut Vec<Pt>,
    radius: f64,
    settings: &Settings,
) -> Result<i32, EngineError> {
    let sp = [points.at(0)?, points.at(1)?, points.at(2)?];
    let mut circle_radius = f64::from(25.0_f32 * 0.9_f32);
    if radius > 0.0 {
        circle_radius = radius;
    }
    if radius == 0.0 {
        let chord = calc_distance_double(sp[0], sp[1]);
        if circle_radius > chord / 2.0 {
            circle_radius = chord / 2.0;
        }
    }
    let circle = arc_approximation_double(
        arc_bounds(sp[0], circle_radius),
        (sp[0].x, sp[0].y),
        (sp[0].x, sp[0].y),
    );
    let mut counter = 0;
    for (j, p) in circle.iter().enumerate() {
        put(points, counter, styled(*p, if j == 16 { 5 } else { 0 }));
        counter += 1;
    }
    let reversed = is_seize_arc_reversed(&sp)?;
    for p in seize_arc_points(&sp, &circle, circle_radius, reversed) {
        put(points, counter, p);
        counter += 1;
    }
    if let Some(p) = counter.checked_sub(1).and_then(|i| points.get_mut(i)) {
        p.style = 5;
    }
    for (j, p) in seize_arrow_head(&sp, reversed, settings).iter().enumerate() {
        put(points, counter, styled(*p, if j == 2 { 5 } else { 0 }));
        counter += 1;
    }
    Ok(counter as i32)
}
