//! SCREEN, GUARD, COVER and SARA from DISMSupport.java: two arrows with a
//! jagged shaft, both starting at the first control point.

use super::support::{clamp_size, get_tg_font_size, put};
use crate::engine::base::{At, EngineError, Pt};
use crate::engine::lineutility::basics::{
    calc_distance_double, get_quadrant_double, mid_point_double,
};
use crate::engine::lineutility::relative::line_relative_to_line;
use crate::engine::settings::Settings;
use crate::engine::tactical_lines as lt;
use std::f64::consts::PI;

fn styled(p: Pt, style: i32) -> Pt {
    let mut q = p;
    q.style = style;
    q
}

/// One arm: a jagged shaft from `from` to `end` and an arrowhead at `end`.
/// `js` is +sign for the left arm and -sign for the right arm; `filled`
/// marks the arrowhead as a fill segment (SARA, MOVEMENT_TO_CONTACT).
fn push_jaggy_arm(
    points: &mut Vec<Pt>,
    counter: &mut usize,
    (from, end): (Pt, Pt),
    js: f64,
    delta: f64,
    filled: bool,
) -> Result<(), EngineError> {
    let angle = (from.y - end.y).atan2(from.x - end.x);
    let shaft = (
        (angle + js * PI / 4.0).cos() * delta,
        (angle + js * PI / 4.0).sin() * delta,
    );
    let mid = mid_point_double(from, end, 0);
    let jaggy = [
        styled(from, 0),
        Pt::new(mid.x - shaft.0, mid.y - shaft.1),
        Pt::new(mid.x + shaft.0, mid.y + shaft.1),
        styled(end, 0),
    ];
    for (k, p) in jaggy.iter().enumerate() {
        put(points, *counter, styled(*p, if k == 3 { 5 } else { 0 }));
        *counter += 1;
    }
    let barb = (
        (angle - js * PI / 4.0).cos() * delta,
        (angle - js * PI / 4.0).sin() * delta,
    );
    let arrow = [
        Pt::new(end.x + shaft.0, end.y + shaft.1),
        end,
        Pt::new(end.x + barb.0, end.y + barb.1),
    ];
    let fill_style = if filled { 9 } else { 0 };
    for (k, p) in arrow.iter().enumerate() {
        let style = if k == 2 {
            if filled { 9 } else { 5 }
        } else {
            fill_style
        };
        put(points, *counter, styled(*p, style));
        *counter += 1;
    }
    if filled {
        let first = counter
            .checked_sub(3)
            .ok_or(EngineError::Degenerate("arrow has no first point"))?;
        let copy = points.at(first)?;
        put(points, *counter, styled(copy, 10));
        *counter += 1;
    }
    Ok(())
}

/// Copies the points of `src` that the narrow-figure fallback draws as two
/// plain lines from the first point.
fn plain_lines(points: &mut Vec<Pt>, origin: Pt, a: Pt, b: Pt) -> i32 {
    put(points, 0, styled(origin, 0));
    put(points, 1, styled(a, 5));
    put(points, 2, styled(origin, 0));
    put(points, 3, b);
    4
}

/// Upstream `GetDISMCoverDouble`: SCREEN, GUARD, COVER and SARA from three
/// control points.
pub(crate) fn get_dism_cover_double(
    points: &mut Vec<Pt>,
    line_type: i32,
    settings: &Settings,
) -> Result<i32, EngineError> {
    let (pt0, pt1, pt2) = (points.at(0)?, points.at(1)?, points.at(2)?);
    let mut pt3 = Pt::default();
    let mut pt4 = Pt::default();
    let mut pt2_out = pt2;
    line_relative_to_line(pt1, pt2, pt0, &mut pt3, &mut pt4);
    line_relative_to_line(pt3, pt0, pt1, &mut pt2_out, &mut pt4);
    let quadrant = get_quadrant_double(pt0, pt4);
    let sign = if (pt1.x < pt2.x && (quadrant == 1 || quadrant == 4))
        || (pt1.x > pt2.x && (quadrant == 2 || quadrant == 3))
    {
        -1.0
    } else {
        1.0
    };
    let initial = styled(pt0, 0);
    let (end0, end1) = (styled(pt1, 0), styled(pt2, 0));
    let length = calc_distance_double(initial, end0).min(calc_distance_double(initial, end1));
    if get_tg_font_size(length) <= 0.0 {
        return Ok(plain_lines(points, initial, end0, end1));
    }
    let delta = clamp_size(length / 15.0, settings.dpi_scale_factor());
    let sara = line_type == lt::SARA;
    let mut counter = 0;
    for (end, js) in [(end0, sign), (end1, -sign)] {
        let mut from = initial;
        if !sara {
            let angle = (initial.y - end.y).atan2(initial.x - end.x);
            from.x -= 30.0 * angle.cos();
            from.y -= 30.0 * angle.sin();
        }
        push_jaggy_arm(points, &mut counter, (from, end), js, delta, sara)?;
    }
    Ok(counter as i32)
}

/// The sign that flips the jaggies for the quadrant of the first point.
fn rev_c_sign(pt0: Pt, pt1: Pt, pt2: Pt) -> f64 {
    let mut pt3 = Pt::default();
    let mut pt4 = Pt::default();
    let mut out = pt2;
    line_relative_to_line(pt1, pt2, pt0, &mut pt3, &mut pt4);
    line_relative_to_line(pt3, pt0, pt1, &mut out, &mut pt4);
    let q = get_quadrant_double(pt0, pt4);
    let flip = (pt1.x < pt2.x && q == 1)
        || (pt1.x > pt2.x && q == 2)
        || (pt1.x > pt2.x && q == 3)
        || (pt1.x < pt2.x && q == 4);
    if flip { -1.0 } else { 1.0 }
}

/// Upstream `GetDISMCoverDoubleRevC`. With `vbl_save_counter == 4` the four
/// control points are the two arrow lines (rev C); with fewer, the first
/// three points describe the figure (rev B).
pub(crate) fn get_dism_cover_double_rev_c(
    points: &mut Vec<Pt>,
    line_type: i32,
    vbl_save_counter: i32,
    settings: &Settings,
) -> Result<i32, EngineError> {
    let rev_c = vbl_save_counter == 4;
    let mut orig = [Pt::default(); 4];
    if rev_c {
        for (k, slot) in orig.iter_mut().enumerate() {
            *slot = points.at(k)?;
        }
        put(points, 1, orig[0]);
        put(points, 2, orig[3]);
        let first = points.at_mut(0)?;
        first.x = (orig[1].x + orig[2].x) / 2.0;
        first.y = (orig[1].y + orig[2].y) / 2.0;
    }
    let sign = rev_c_sign(points.at(0)?, points.at(1)?, points.at(2)?);
    let t = if line_type == lt::SARA { 0.0 } else { 1.0 };
    let go_left_then_right = points.at(1)?.x <= points.at(2)?.x;
    let mut sp = [points.at(0)?, points.at(1)?, points.at(2)?];
    for p in &mut sp {
        p.style = 0;
    }
    let len01 = calc_distance_double(sp[0], sp[1]);
    let len02 = calc_distance_double(sp[0], sp[2]);
    let length = if len01 > len02 { len02 } else { len01 };
    if get_tg_font_size(length) <= 0.0 {
        return Ok(plain_lines(points, sp[0], sp[1], sp[2]));
    }
    let delta = clamp_size(length / 15.0, settings.dpi_scale_factor());
    let filled = line_type == lt::SARA || line_type == lt::MOVEMENT_TO_CONTACT;
    let (left, right) = if rev_c {
        ((orig[1], orig[0]), (orig[2], orig[3]))
    } else {
        let left_x = if go_left_then_right {
            sp[0].x - 30.0 * t
        } else {
            sp[0].x + 30.0 * t
        };
        let right_x = if go_left_then_right {
            left_x + 60.0 * t
        } else {
            left_x - 60.0 * t
        };
        let from_left = Pt::new(left_x, sp[0].y);
        let from_right = Pt::new(right_x, sp[0].y);
        ((from_left, sp[1]), (from_right, sp[2]))
    };
    let mut counter = 0;
    push_jaggy_arm(points, &mut counter, left, sign, delta, filled)?;
    push_jaggy_arm(points, &mut counter, right, -sign, delta, filled)?;
    Ok(counter as i32)
}
