//! Geometry shared by the label phases of Modifier2.java: the symbol's
//! bounding box, segment choice, the vertical-path shift and the helpers
//! that add one label to several places.

use super::ABOVE_MIDDLE;
use super::add::{add_area_modifier, add_modifier, count, integral_modifier, px};
use crate::engine::base::{At, EngineError, Pt, java_round};
use crate::engine::lineutility::basics::{calc_distance_double, mid_point_double};
use crate::engine::lineutility::extend::extend_along_line_double2;
use crate::engine::tactical_lines as tl;
use crate::engine::tg::Tg;

/// Upstream `Double.parseDouble` for the values the label code reads:
/// surrounding whitespace is ignored.
pub(super) fn parse_double(text: &str) -> Result<f64, EngineError> {
    text.trim()
        .parse::<f64>()
        .map_err(|_| EngineError::Number(text.to_owned()))
}

/// Upstream `removeDecimal(double)`: `String.valueOf(Math.round(v))`.
pub(super) fn remove_decimal_value(value: f64) -> String {
    format!("{}", java_round(value) as i64)
}

/// Upstream `removeDecimal(String)`: rounds the number before the first
/// space (a unit may follow) and keeps the rest.
pub(super) fn remove_decimal(text: &str) -> Result<String, EngineError> {
    match text.find(' ') {
        Some(at) if at > 0 => {
            let (number, unit) = text.split_at(at);
            Ok(format!(
                "{}{unit}",
                remove_decimal_value(parse_double(number)?)
            ))
        }
        _ => Ok(remove_decimal_value(parse_double(text)?)),
    }
}

/// Upstream `GetMBR`: the corners of the pixels' bounding box as
/// `(upper left, upper right, lower right, lower left)`.
pub(crate) fn get_mbr(tg: &Tg) -> Result<(Pt, Pt, Pt, Pt), EngineError> {
    let first = tg.pixels.at(0)?;
    let (mut ul, mut ur, mut lr, mut ll) = (first, first, first, first);
    for p in tg.pixels.iter().skip(1) {
        if p.x < ll.x {
            ll.x = p.x;
            ul.x = p.x;
        }
        if p.x > lr.x {
            lr.x = p.x;
            ur.x = p.x;
        }
        if p.y > ll.y {
            ll.y = p.y;
            lr.y = p.y;
        }
        if p.y < ul.y {
            ul.y = p.y;
            ur.y = p.y;
        }
    }
    Ok((ul, ur, lr, ll))
}

/// The pixels' middle segment. Upstream's `getVisibleMiddleSegment` walks
/// outward to a segment inside the clip bounds only when it is given any;
/// the engine does not clip.
pub(super) fn middle_segment(tg: &Tg) -> i32 {
    (count(tg) + 1) / 2 - 1
}

/// The copies of the end points a phase works with. Upstream's
/// `shiftModifierPath` moves them so that a path that would be vertical is
/// not.
#[derive(Clone, Copy, Debug)]
pub(super) struct EndPoints {
    pub(super) pt0: Pt,
    pub(super) pt1: Option<Pt>,
    pub(super) pt_last: Pt,
    pub(super) pt_next_to_last: Option<Pt>,
}

impl EndPoints {
    /// The first two and last two pixels.
    pub(super) fn of(tg: &Tg) -> Result<Self, EngineError> {
        let n = count(tg);
        Ok(Self {
            pt0: px(tg, 0)?,
            pt1: if n > 1 { Some(px(tg, 1)?) } else { None },
            pt_last: px(tg, n - 1)?,
            pt_next_to_last: if n > 1 { Some(px(tg, n - 2)?) } else { None },
        })
    }
}

/// Upstream `shiftModifierPath`: boundaries move the pixels themselves, the
/// lines that label their ends move the end-point copies.
pub(super) fn shift_modifier_path(tg: &mut Tg, ends: &mut EndPoints) -> Result<(), EngineError> {
    match tg.line_type {
        tl::BOUNDARY => {
            let mut last = -1.0;
            for j in 0..tg.pixels.len().saturating_sub(1) {
                let p0 = tg.pixels.at(j)?;
                let p1 = tg.pixels.at_mut(j + 1)?;
                if (p0.x - p1.x).abs() < 1.0 {
                    p1.x += last;
                    last = -last;
                }
            }
        }
        tl::PDF
        | tl::PL
        | tl::DECISION_LINE
        | tl::FEBA
        | tl::LOA
        | tl::LOD
        | tl::RELEASE
        | tl::HOL
        | tl::BHL
        | tl::LDLC
        | tl::LL
        | tl::EWL
        | tl::FCL
        | tl::PLD
        | tl::NFL
        | tl::FLOT
        | tl::LC
        | tl::HOLD
        | tl::BRDGHD
        | tl::HOLD_GE
        | tl::BRDGHD_GE => {
            if let Some(p1) = ends.pt1.as_mut() {
                if (ends.pt0.x - p1.x).abs() < 1.0 {
                    p1.x += 1.0;
                }
            }
            if let Some(next) = ends.pt_next_to_last.as_mut() {
                if (next.x - ends.pt_last.x).abs() < 1.0 {
                    next.x += 1.0;
                }
            }
        }
        _ => {}
    }
    Ok(())
}

/// Upstream `addModifierOnLine`: the label at the middle of the segments
/// that are furthest left, right, up and down (the first two only when
/// `two_label_only`). A midpoint is compared by identity upstream, so the
/// four start as distinct candidates and a segment can win several.
///
/// Upstream catches a failure (fewer than two pixels) inside this function
/// and goes on, so this adds nothing in that case and reports nothing.
pub(super) fn add_modifier_on_line(tg: &mut Tg, label: &str, two_label_only: bool) {
    if label.is_empty() || tg.pixels.is_empty() {
        return;
    }
    place_on_line(tg, label, two_label_only).ok();
}

/// The body of [`add_modifier_on_line`].
fn place_on_line(tg: &mut Tg, label: &str, two_label_only: bool) -> Result<(), EngineError> {
    let first = mid_point_double(px(tg, 0)?, px(tg, 1)?, 0);
    let (mut left, mut right, mut top, mut bottom) =
        ((first, -1), (first, -2), (first, -3), (first, -4));
    for j in 1..count(tg) - 1 {
        let mid = (mid_point_double(px(tg, j)?, px(tg, j + 1)?, 0), j);
        if mid.0.x <= left.0.x {
            left = mid;
        }
        if mid.0.x >= right.0.x {
            right = mid;
        }
        if mid.0.y <= top.0.y {
            top = mid;
        }
        if mid.0.y >= bottom.0.y {
            bottom = mid;
        }
    }
    let put = |tg: &mut Tg, p: Pt| add_area_modifier(tg, label, ABOVE_MIDDLE, 0.0, (p, p));
    if left.1 != right.1 {
        put(tg, left.0);
    }
    put(tg, right.0);
    if !two_label_only {
        if bottom.1 != left.1 && bottom.1 != right.1 {
            put(tg, bottom.0);
        }
        if top.1 != left.1 && top.1 != right.1 && top.1 != bottom.1 {
            put(tg, top.0);
        }
    }
    Ok(())
}

/// Upstream `addNModifier`: the hostile "N" label on the outline.
pub(super) fn add_n_modifier(tg: &mut Tg) {
    if tg.is_hostile() {
        let n = tg.n.clone();
        add_modifier_on_line(tg, &n, true);
    }
}

/// Index of the segment whose end points have the extreme `y` sum,
/// `lowest` or highest on screen (largest or smallest `y`).
fn extreme_segment(tg: &Tg, bottom: bool) -> Result<i32, EngineError> {
    let mut index = 0;
    let mut y = px(tg, 0)?.y + px(tg, 1)?.y;
    for i in 1..count(tg) - 1 {
        let sum = px(tg, i)?.y + px(tg, i + 1)?.y;
        if (bottom && sum > y) || (!bottom && sum < y) {
            index = i;
            y = sum;
        }
    }
    Ok(index)
}

/// Upstream `addModifierBottomSegment`.
pub(super) fn add_modifier_bottom_segment(tg: &mut Tg, text: &str) -> Result<(), EngineError> {
    let index = extreme_segment(tg, true)?;
    integral_modifier(tg, text, ABOVE_MIDDLE, 0.0, (index, index + 1), false)
}

/// Upstream `addModifierTopSegment`.
pub(super) fn add_modifier_top_segment(tg: &mut Tg, text: &str) -> Result<(), EngineError> {
    let index = extreme_segment(tg, false)?;
    integral_modifier(tg, text, ABOVE_MIDDLE, 0.0, (index, index + 1), false)
}

/// Upstream `addDTG`: W and W1 on two lines, W1 always on the outer one.
pub(super) fn add_dtg(tg: &mut Tg, kind: i32, factors: (f64, f64), pt0: Pt, pt1: Pt) {
    let dash = if tg.w.is_empty() || tg.w1.is_empty() {
        ""
    } else {
        " - "
    };
    let first = format!("{}{dash}", tg.w);
    let second = tg.w1.clone();
    add_modifier(tg, &first, kind, factors.0.min(factors.1), pt0, pt1);
    add_modifier(tg, &second, kind, factors.0.max(factors.1), pt0, pt1);
}

/// Upstream `getHighestPointLeftOfCenter`: the topmost point not right of
/// the centre, or the first point if none is left of it.
pub(super) fn highest_point_left_of_center(pixels: &[Pt], center: Pt) -> Result<Pt, EngineError> {
    let mut highest = pixels.at(0)?;
    let mut found = false;
    for p in pixels {
        if p.x <= center.x {
            if !found {
                highest = *p;
                found = true;
            } else if p.y < highest.y {
                highest = *p;
            }
        }
    }
    Ok(highest)
}

/// Upstream `getRFALines`: label lines of the fire support areas.
pub(super) fn get_rfa_lines(tg: &Tg) -> i32 {
    let mut lines = 1;
    if !tg.t.is_empty() {
        lines += 1;
    }
    if !tg.w.is_empty() || !tg.w1.is_empty() {
        lines += 1;
    }
    lines
}

/// Upstream `getPixelsMiddleSegment` (CFL only): a segment of `string_width`
/// centred on the middle of the polyline's length. Upstream sums the
/// lengths in an `int`.
pub(super) fn pixels_middle_segment(
    tg: &Tg,
    string_width: f64,
    seg: (Pt, Pt),
) -> Result<(Pt, Pt), EngineError> {
    if tg.line_type != tl::CFL {
        return Ok(seg);
    }
    let n = tg.pixels.len();
    let mut total: i32 = 0;
    for j in 0..n.saturating_sub(1) {
        let dist = calc_distance_double(tg.pixels.at(j)?, tg.pixels.at(j + 1)?);
        total = (f64::from(total) + dist) as i32;
    }
    let mid = f64::from(total / 2);
    total = 0;
    for j in 0..n.saturating_sub(1) {
        let (pt0, pt1) = (tg.pixels.at(j)?, tg.pixels.at(j + 1)?);
        total = (f64::from(total) + calc_distance_double(pt0, pt1)) as i32;
        if f64::from(total) >= mid {
            let remainder = f64::from(total) - mid;
            let mid_pt = extend_along_line_double2(pt1, pt0, remainder);
            let pt2 = extend_along_line_double2(mid_pt, pt0, string_width / 2.0);
            let pt3 = extend_along_line_double2(mid_pt, pt1, string_width / 2.0);
            let (mut s0, mut s1) = seg;
            (s0.x, s0.y, s1.x, s1.y) = (pt2.x, pt2.y, pt3.x, pt3.y);
            return Ok((s0, s1));
        }
    }
    Ok(seg)
}

/// Java `String.split(",")` for the tests of this module.
#[cfg(test)]
pub(super) fn split_commas_for_tests(text: &str) -> Vec<&str> {
    super::post_areas::split_commas(text)
}
