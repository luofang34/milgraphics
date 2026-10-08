//! Port of `GetArrowHead4Double` from lineutility.java.

use crate::engine::base::{At, EngineError, Pt};
use std::f64::consts::{FRAC_PI_2, PI};

/// Upstream `GetArrowHead4Double`: writes the three arrowhead points (left
/// base corner, tip, right base corner) into `result[0..3]`. The tip is
/// `end_line_point`; `n_bi_sector` is the head length and `n_base` its base
/// width. Base corners are truncated to whole pixels as upstream does.
/// `styl` selects the point styles: the last point ends the polyline (5, or
/// 10 for fills, 5 for dashed variants).
pub(crate) fn get_arrow_head4_double(
    start_line_point: Pt,
    end_line_point: Pt,
    n_bi_sector: i32,
    n_base: i32,
    result: &mut [Pt],
    styl: i32,
) -> Result<(), EngineError> {
    result.at(2)?;
    let dy = end_line_point.y - start_line_point.y;
    let dx = end_line_point.x - start_line_point.x;
    let angle = if dy == 0.0 {
        if dx > 0.0 { PI } else { 0.0 }
    } else {
        (dx / dy).atan() + FRAC_PI_2
    };
    let mut corner = start_line_point;
    corner.style = 0;
    // Upstream's four overlapping sign tests reduce to the sign of dy, with
    // dy == 0 counting as positive.
    let sign = if dy < 0.0 { -1.0 } else { 1.0 };
    let hyp = sign * f64::from(n_bi_sector);
    let base_x = end_line_point.x + hyp * angle.cos();
    let base_y = end_line_point.y - hyp * angle.sin();
    let half = sign * (f64::from(n_base) / 2.0);
    let left_x = base_x - half * angle.sin();
    let left_y = base_y - half * angle.cos();
    let right_x = base_x + half * angle.sin();
    let right_y = base_y + half * angle.cos();
    corner.x = f64::from(left_x as i32);
    corner.y = f64::from(left_y as i32);
    let p0 = corner;
    corner.x = f64::from(right_x as i32);
    corner.y = f64::from(right_y as i32);
    let p2 = corner;
    let (body, last) = match styl {
        0 => (0, 5),
        9 => (9, 10),
        18 => (18, 5),
        s => (s, 5),
    };
    *result.at_mut(0)? = Pt { style: body, ..p0 };
    *result.at_mut(1)? = Pt {
        style: body,
        ..end_line_point
    };
    *result.at_mut(2)? = Pt { style: last, ..p2 };
    Ok(())
}
