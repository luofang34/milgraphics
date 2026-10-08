//! SPTBYFIRE and ATKBYFIRE from DISMSupport.java, with the point
//! reordering that makes the arrows point away from the base.

use super::support::{MAX_LENGTH, MIN_LENGTH, clamp_size, put};
use crate::engine::base::{At, EngineError, Pt};
use crate::engine::lineutility::basics::{calc_distance_double, mid_point_double};
use crate::engine::lineutility::bounds::reverse2_points;
use crate::engine::lineutility::extend::extend_directed_line;
use crate::engine::lineutility::slope::calc_distance_to_line_double;
use crate::engine::lineutility::{EXTEND_ABOVE, EXTEND_BELOW, EXTEND_LEFT, EXTEND_RIGHT};
use crate::engine::settings::Settings;
use std::f64::consts::PI;

fn styled(p: Pt, style: i32) -> Pt {
    let mut q = p;
    q.style = style;
    q
}

fn push_segment(points: &mut Vec<Pt>, counter: &mut usize, a: Pt, b: Pt) {
    put(points, *counter, styled(a, 0));
    put(points, *counter + 1, styled(b, 5));
    *counter += 2;
}

/// Two barbs meeting at `tip`, offset by `(dx1, dy1)` and `(dx2, dy2)`.
fn push_head(points: &mut Vec<Pt>, counter: &mut usize, tip: Pt, d1: (f64, f64), d2: (f64, f64)) {
    let head = [
        Pt::new(tip.x + d1.0, tip.y + d1.1),
        tip,
        Pt::new(tip.x + d2.0, tip.y + d2.1),
    ];
    for (k, p) in head.iter().enumerate() {
        put(points, *counter, styled(*p, if k == 2 { 5 } else { 0 }));
        *counter += 1;
    }
}

fn swap_xy(p: &mut [Pt], a: usize, b: usize) -> Result<(), EngineError> {
    let mut pa = p.at(a)?;
    let mut pb = p.at(b)?;
    reverse2_points(&mut pa, &mut pb);
    *p.at_mut(a)? = pa;
    *p.at_mut(b)? = pb;
    Ok(())
}

/// Upstream `ReorderAtkByFirePoints`: orders the back line so the base
/// reads left to right (or top to bottom) with the arrow on the near side.
fn reorder_atk_by_fire_points(sp: &mut [Pt; 3]) -> Result<(), EngineError> {
    let dist = calc_distance_to_line_double(sp[1], sp[2], sp[0]);
    if (sp[1].x - sp[2].x).abs() > 2.0 {
        let above = extend_directed_line(sp[1], sp[2], sp[2], EXTEND_ABOVE, dist);
        let below = extend_directed_line(sp[1], sp[2], sp[2], EXTEND_BELOW, dist);
        let to_above = calc_distance_double(sp[0], above);
        let to_below = calc_distance_double(sp[0], below);
        let flip = if to_above < to_below {
            sp[2].x < sp[1].x
        } else {
            sp[2].x > sp[1].x
        };
        if flip {
            swap_xy(sp, 1, 2)?;
        }
    } else {
        let left = extend_directed_line(sp[1], sp[2], sp[2], EXTEND_LEFT, dist);
        let right = extend_directed_line(sp[1], sp[2], sp[2], EXTEND_RIGHT, dist);
        let to_left = calc_distance_double(sp[0], left);
        let to_right = calc_distance_double(sp[0], right);
        let flip = if to_right < to_left {
            sp[2].y < sp[1].y
        } else {
            sp[2].y > sp[1].y
        };
        if flip {
            swap_xy(sp, 1, 2)?;
        }
    }
    Ok(())
}

/// Upstream `ReorderSptByFirePoints`: swaps both point pairs when the
/// arrows would cross the base.
fn reorder_spt_by_fire_points(p: &mut [Pt; 4]) -> Result<(), EngineError> {
    let mid = mid_point_double(p[0], p[1], 0);
    let flip = if (p[2].x - p[3].x).abs() > 2.0 {
        let dist = calc_distance_to_line_double(p[1], p[2], mid);
        let above = extend_directed_line(p[1], p[2], p[2], EXTEND_ABOVE, dist);
        let below = extend_directed_line(p[1], p[2], p[2], EXTEND_BELOW, dist);
        if calc_distance_double(p[0], above) < calc_distance_double(p[0], below) {
            p[2].x < p[1].x
        } else {
            p[2].x > p[1].x
        }
    } else {
        let dist = calc_distance_to_line_double(p[1], p[2], mid);
        let left = extend_directed_line(p[1], p[2], p[2], EXTEND_LEFT, dist);
        let right = extend_directed_line(p[1], p[2], p[2], EXTEND_RIGHT, dist);
        if calc_distance_double(p[0], left) < calc_distance_double(p[0], right) {
            p[2].y > p[1].y
        } else {
            p[2].y < p[1].y
        }
    };
    if flip {
        swap_xy(p, 0, 1)?;
        swap_xy(p, 2, 3)?;
    }
    Ok(())
}

/// Upstream `GetDISMSupportByFireDouble`: SPTBYFIRE, 16 points.
pub(crate) fn get_dism_support_by_fire_double(
    points: &mut Vec<Pt>,
    settings: &Settings,
) -> Result<i32, EngineError> {
    let mut sp = [points.at(0)?, points.at(1)?, points.at(2)?, points.at(3)?];
    reorder_spt_by_fire_points(&mut sp)?;
    let mut counter = 0;
    push_segment(points, &mut counter, sp[0], sp[1]);
    push_segment(points, &mut counter, sp[0], sp[2]);
    let mut diag = clamp_size(
        ((sp[0].x - sp[1].x) * (sp[0].x - sp[1].x) + (sp[0].y - sp[1].y) * (sp[0].y - sp[1].y))
            .sqrt()
            / 10.0,
        settings.dpi_scale_factor(),
    );
    let barbs = |from: Pt, to: Pt, diag: f64| {
        let a = (from.y - to.y).atan2(from.x - to.x);
        (
            ((a + PI / 6.0).cos() * diag, (a + PI / 6.0).sin() * diag),
            ((a - PI / 6.0).cos() * diag, (a - PI / 6.0).sin() * diag),
        )
    };
    let (d1, d2) = barbs(sp[0], sp[2], diag);
    push_head(points, &mut counter, sp[2], d1, d2);
    push_segment(points, &mut counter, sp[1], sp[3]);
    let (d1, d2) = barbs(sp[1], sp[3], diag);
    push_head(points, &mut counter, sp[3], d1, d2);

    let angle = (sp[1].y - sp[0].y).atan2(sp[1].x - sp[0].x);
    diag *= 2.0;
    let back1 = (
        (angle - PI / 4.0).cos() * diag,
        (angle - PI / 4.0).sin() * diag,
    );
    let back2 = (
        (angle + PI / 4.0).cos() * diag,
        (angle + PI / 4.0).sin() * diag,
    );
    let tail0 = Pt::new(sp[0].x - back1.0, sp[0].y - back1.1);
    push_segment(points, &mut counter, tail0, sp[0]);
    let tail1 = Pt::new(sp[1].x + back2.0, sp[1].y + back2.1);
    push_segment(points, &mut counter, tail1, sp[1]);
    Ok(counter as i32)
}

/// Upstream `GetDISMATKBYFIREDouble`: ATKBYFIRE, 11 points.
pub(crate) fn get_dism_atk_by_fire_double(
    points: &mut Vec<Pt>,
    settings: &Settings,
) -> Result<i32, EngineError> {
    let mut sp = [points.at(0)?, points.at(1)?, points.at(2)?];
    reorder_atk_by_fire_points(&mut sp)?;
    let mut counter = 0;
    push_segment(points, &mut counter, sp[1], sp[2]);
    let mid = Pt::new((sp[1].x + sp[2].x) / 2.0, (sp[1].y + sp[2].y) / 2.0);
    push_segment(points, &mut counter, mid, sp[0]);

    let back = ((sp[1].x - sp[2].x) * (sp[1].x - sp[2].x)
        + (sp[1].y - sp[2].y) * (sp[1].y - sp[2].y))
        .sqrt();
    let depth =
        ((sp[0].x - mid.x) * (sp[0].x - mid.x) + (sp[0].y - mid.y) * (sp[0].y - mid.y)).sqrt();
    let dpi = settings.dpi_scale_factor();
    let mut diag = (back + depth) / 20.0;
    if diag > MAX_LENGTH / 5.0 * dpi {
        diag = MAX_LENGTH / 5.0 * dpi;
    }
    if diag < MIN_LENGTH * dpi {
        diag = MIN_LENGTH * dpi;
    }

    let angle = (mid.y - sp[0].y).atan2(mid.x - sp[0].x);
    let d1 = (
        (angle + PI / 6.0).cos() * diag,
        (angle + PI / 6.0).sin() * diag,
    );
    let d2 = (
        (angle - PI / 6.0).cos() * diag,
        (angle - PI / 6.0).sin() * diag,
    );
    push_head(points, &mut counter, sp[0], d1, d2);

    let angle = (sp[1].y - sp[2].y).atan2(sp[1].x - sp[2].x);
    let e1 = (
        (angle - PI / 4.0).cos() * diag * 2.0,
        (angle - PI / 4.0).sin() * diag * 2.0,
    );
    let e2 = (
        (angle + PI / 4.0).cos() * diag * 2.0,
        (angle + PI / 4.0).sin() * diag * 2.0,
    );
    push_segment(
        points,
        &mut counter,
        Pt::new(sp[1].x + e1.0, sp[1].y + e1.1),
        sp[1],
    );
    push_segment(
        points,
        &mut counter,
        Pt::new(sp[2].x - e2.0, sp[2].y - e2.1),
        sp[2],
    );
    Ok(counter as i32)
}
