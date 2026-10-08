//! `AddBoundaryModifiers` and `GetBoundarySegmentTooShort` of Modifier2.java:
//! the name, echelon and T1 labels on one segment of a boundary or
//! engineer work line.

use super::ABOVE_MIDDLE;
use super::add::{integral_modifier, px};
use super::layout::middle_segment;
use crate::engine::base::EngineError;
use crate::engine::lineutility::basics::calc_distance_double;
use crate::engine::tactical_lines as tl;
use crate::engine::tg::Tg;

/// Upstream `GetBoundarySegmentTooShort`: true for a boundary segment
/// shorter than 1.25 times its widest label. Upstream catches a missing
/// segment and answers false.
fn boundary_segment_too_short(tg: &Tg, segment: i32, text_width: &dyn Fn(&str) -> f64) -> bool {
    let measure = |text: &str| -> i32 {
        if text.is_empty() {
            0
        } else {
            text_width(text) as i32
        }
    };
    let (Ok(pt0), Ok(pt1)) = (px(tg, segment), px(tg, segment + 1)) else {
        return false;
    };
    let dist = calc_distance_double(pt0, pt1);
    let total = measure(&tg.echelon_symbol)
        .max(measure(&tg.t))
        .max(measure(&tg.t1));
    tg.line_type == tl::BOUNDARY && dist < 1.25 * f64::from(total)
}

/// Upstream `AddBoundaryModifiers`: labels the middle segment, or, when it
/// is too short for them, labels it anyway.
pub(super) fn add_boundary_modifiers(
    tg: &mut Tg,
    text_width: &dyn Fn(&str) -> f64,
) -> Result<(), EngineError> {
    let country = if tg.as_.is_empty() {
        String::new()
    } else {
        format!(" ({})", tg.as_)
    };
    let seg = middle_segment(tg);
    let (pt0, pt1) = (px(tg, seg)?, px(tg, seg + 1)?);
    let (t_factor, t1_factor) = if pt0.x < pt1.x {
        (-1.3, 1.0)
    } else if pt0.x == pt1.x {
        if pt1.y < pt0.y {
            (-1.0, 1.0)
        } else {
            (1.0, -1.0)
        }
    } else {
        (1.0, -1.3)
    };
    let too_short = boundary_segment_too_short(tg, seg, text_width);
    // A segment too short for the labels is labelled anyway, with the
    // echelon symbol nudged by a hair more.
    let echelon_factor = if too_short { -0.2020 } else { -0.20 };
    let name = format!("{}{country}", tg.t);
    let path = (seg, seg + 1);
    integral_modifier(tg, &name, ABOVE_MIDDLE, t_factor, path, true)?;
    if !tg.echelon_symbol.is_empty() {
        let echelon = tg.echelon_symbol.clone();
        integral_modifier(tg, &echelon, ABOVE_MIDDLE, echelon_factor, path, true)?;
    }
    let t1 = tg.t1.clone();
    integral_modifier(tg, &t1, ABOVE_MIDDLE, t1_factor, path, true)
}
