//! DECEIVE, DISRUPT and CONTAIN from DISMSupport.java.

use super::support::{
    MAX_LENGTH, arc_approximation_double, clamp_size, draw_endpiece_deltas_double, put,
};
use crate::engine::base::{At, EngineError, Pt};
use crate::engine::lineutility::basics::calc_distance_double;
use crate::engine::lineutility::bounds::reverse2_points;
use crate::engine::lineutility::extend::extend_line_double;
use crate::engine::lineutility::relative::point_relative_to_line;
use crate::engine::lineutility::slope::calc_true_slope_double2;
use crate::engine::settings::Settings;
use std::f64::consts::PI;

fn styled(p: Pt, style: i32) -> Pt {
    let mut q = p;
    q.style = style;
    q
}

/// Upstream `GetDISMDeceiveDouble`: the DECEIVE triangle, 4 points.
pub(crate) fn get_dism_deceive_double(points: &mut Vec<Pt>) -> Result<(), EngineError> {
    let save = [points.at(0)?, points.at(1)?, points.at(2)?];
    put(points, 0, styled(save[0], 1));
    put(points, 1, styled(save[1], 5));
    put(points, 2, styled(save[2], 1));
    put(points, 3, styled(save[0], 5));
    Ok(())
}

fn push_segment(points: &mut Vec<Pt>, counter: &mut usize, a: Pt, b: Pt) {
    put(points, *counter, styled(a, 0));
    put(points, *counter + 1, styled(b, 5));
    *counter += 2;
}

/// Upstream `GetDISMDisruptDouble`: the DISRUPT figure, 20 points.
pub(crate) fn get_dism_disrupt_double(
    points: &mut Vec<Pt>,
    settings: &Settings,
) -> Result<i32, EngineError> {
    let s = [points.at(0)?, points.at(1)?, points.at(2)?];
    let mut counter = 0;
    push_segment(points, &mut counter, s[0], s[1]);
    push_segment(points, &mut counter, s[1], s[2]);
    let center = Pt::new((s[0].x + s[1].x) / 2.0, (s[0].y + s[1].y) / 2.0);
    let (ex, ey) = (s[2].x - s[1].x, s[2].y - s[1].y);
    let arrow = [
        s[2],
        Pt::new(center.x + ex * 4.0 / 5.0, center.y + ey * 4.0 / 5.0),
        Pt::new(s[0].x + ex * 3.0 / 5.0, s[0].y + ey * 3.0 / 5.0),
    ];
    let start = Pt::new(center.x - ex / 5.0, center.y - ey / 5.0);
    push_segment(points, &mut counter, start, arrow[1]);
    push_segment(points, &mut counter, s[0], arrow[2]);

    let height =
        ((s[1].x - s[0].x) * (s[1].x - s[0].x) + (s[1].y - s[0].y) * (s[1].y - s[0].y)).sqrt();
    let length =
        ((s[2].x - s[1].x) * (s[2].x - s[1].x) + (s[2].y - s[1].y) * (s[2].y - s[1].y)).sqrt();
    let diag = clamp_size((height + length) / 15.0, settings.dpi_scale_factor());
    let angle = (s[1].y - s[2].y).atan2(s[1].x - s[2].x);
    let d1 = diag * (angle - PI / 6.0).cos();
    let d2 = diag * (angle - PI / 6.0).sin();
    let d3 = diag * (angle + PI / 6.0).cos();
    let d4 = diag * (angle + PI / 6.0).sin();
    for tip in arrow {
        for p in draw_endpiece_deltas_double(tip, d1, d2, d3, d4) {
            put(points, counter, p);
            counter += 1;
        }
    }
    Ok(counter as i32)
}

/// Orders the first two points so that the arc bulges away from the third.
fn orient_contain_points(sp: &mut [Pt; 3], center: Pt) {
    let relative = point_relative_to_line(sp[0], sp[1], sp[2]);
    let (_, m) = calc_true_slope_double2(sp[0], sp[1]);
    let [a, b, _] = sp;
    if m != 0.0 {
        if a.y > b.y && relative.x > center.x {
            reverse2_points(a, b);
        }
        if a.y < b.y && relative.x < center.x {
            reverse2_points(a, b);
        }
    } else {
        if a.x < b.x && relative.y > center.y {
            reverse2_points(a, b);
        }
        if a.x > b.x && relative.y < center.y {
            reverse2_points(a, b);
        }
    }
}

/// A spoke from `start` toward the centre, shortened to the pixel limit.
fn push_spoke(points: &mut Vec<Pt>, counter: &mut usize, start: Pt, center: Pt, max: f64) {
    let mut end = Pt::new((start.x + center.x) / 2.0, (start.y + center.y) / 2.0);
    if calc_distance_double(start, end) > max {
        end = extend_line_double(end, start, -max);
    }
    push_segment(points, counter, start, end);
}

/// Upstream `GetDISMContainDouble`: the CONTAIN figure (arrow, arc and
/// spokes).
pub(crate) fn get_dism_contain_double(
    points: &mut Vec<Pt>,
    settings: &Settings,
) -> Result<i32, EngineError> {
    let mut sp = [points.at(0)?, points.at(1)?, points.at(2)?];
    let center = Pt::new((sp[0].x + sp[1].x) / 2.0, (sp[0].y + sp[1].y) / 2.0);
    orient_contain_points(&mut sp, center);
    let mut perp_length = ((center.x - sp[2].x) * (center.x - sp[2].x)
        + (center.y - sp[2].y) * (center.y - sp[2].y))
        .sqrt();
    if perp_length < 1.0 {
        perp_length = 1.0;
    }
    let angle1 = (sp[0].y - sp[1].y).atan2(sp[0].x - sp[1].x);
    let perp = Pt::new(
        center.x + (angle1 + PI / 2.0).cos() * perp_length,
        center.y + (angle1 + PI / 2.0).sin() * perp_length,
    );
    let mut counter = 0;
    put(points, 0, styled(center, 14));
    put(points, 1, styled(sp[2], 5));
    counter += 2;

    let radius = ((center.x - sp[0].x) * (center.x - sp[0].x)
        + (center.y - sp[0].y) * (center.y - sp[0].y))
        .sqrt();
    let dpi = settings.dpi_scale_factor();
    let diag = clamp_size((perp_length + radius) / 20.0, dpi);
    let angle2 = (perp.y - center.y).atan2(perp.x - center.x);
    let (dx1, dy1) = ((angle2 + PI / 4.0).cos(), (angle2 + PI / 4.0).sin());
    let (dx2, dy2) = ((angle2 - PI / 4.0).cos(), (angle2 - PI / 4.0).sin());
    let head = [
        Pt::new(center.x + dx1 * diag, center.y + dy1 * diag),
        center,
        Pt::new(center.x + dx2 * diag, center.y + dy2 * diag),
    ];
    for (i, p) in head.iter().enumerate() {
        put(points, counter, styled(*p, if i == 2 { 5 } else { 0 }));
        counter += 1;
    }

    let bounds = (
        center.x - radius,
        center.y - radius,
        center.x + radius,
        center.y + radius,
    );
    let arc = arc_approximation_double(bounds, (sp[0].x, sp[0].y), (sp[1].x, sp[1].y));
    for (i, p) in arc.iter().enumerate() {
        put(points, counter, styled(*p, if i == 16 { 5 } else { 0 }));
        counter += 1;
    }

    let max = MAX_LENGTH * dpi;
    push_spoke(points, &mut counter, sp[0], center, max);
    push_spoke(points, &mut counter, sp[1], center, max);
    let back = Pt::new(
        center.x - (perp.x - center.x) * radius / perp_length,
        center.y - (perp.y - center.y) * radius / perp_length,
    );
    push_spoke(points, &mut counter, back, center, max);
    let spread = [
        (dx1, dy1),
        (dx2, dy2),
        ((angle2 + PI / 8.0).cos(), (angle2 + PI / 8.0).sin()),
        ((angle2 - PI / 8.0).cos(), (angle2 - PI / 8.0).sin()),
        (
            (angle2 + 3.0 * PI / 8.0).cos(),
            (angle2 + 3.0 * PI / 8.0).sin(),
        ),
        (
            (angle2 - 3.0 * PI / 8.0).cos(),
            (angle2 - 3.0 * PI / 8.0).sin(),
        ),
    ];
    for (dx, dy) in spread {
        let start = Pt::new(center.x - dx * radius, center.y - dy * radius);
        push_spoke(points, &mut counter, start, center, max);
    }
    Ok(counter as i32)
}
