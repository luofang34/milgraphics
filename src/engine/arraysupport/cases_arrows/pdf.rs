//! PDF: the line with two end ticks as a rectangle and two arrowheads.

use super::{arrow, clamp_mbr};
use crate::engine::arraysupport::work::{Work, get, set, set_style};
use crate::engine::base::{EngineError, Pt};
use crate::engine::flot::get_scaled_size;
use crate::engine::lineutility::basics::get_pixels_min;
use crate::engine::lineutility::extend::extend_line_double;
use crate::engine::lineutility::slope::calc_true_slope_double;

/// The two points `half` either side of `center`, perpendicular to a line of
/// slope `m`, as `(lower side, upper side)` in upstream's call order
/// `(-half, +half)`.
fn ticks(vertical: i32, m: f64, offset_x: f64, center: Pt, half: f64, seed: Pt) -> Vec<(Pt, Pt)> {
    let mut out = Vec::new();
    if vertical != 0 && m != 0.0 {
        let b = center.y + (1.0 / m) * center.x;
        let b1 = (-1.0 / m) * offset_x + b;
        let intercept = Pt {
            x: offset_x,
            y: b1,
            ..seed
        };
        out.push((
            extend_line_double(intercept, center, -half),
            extend_line_double(intercept, center, half),
        ));
    }
    if vertical != 0 && m == 0.0 {
        out.push((
            Pt {
                y: center.y - half,
                ..center
            },
            Pt {
                y: center.y + half,
                ..center
            },
        ));
    }
    if vertical == 0 {
        out.push((
            Pt {
                x: center.x - half,
                ..center
            },
            Pt {
                x: center.x + half,
                ..center
            },
        ));
    }
    out
}

/// PDF.
pub(crate) fn pdf(w: &mut Work<'_>) -> Result<(), EngineError> {
    let pt0 = get(&w.p, 1)?;
    let pt1 = get(&w.p, 0)?;
    set(&mut w.p, 0, pt0)?;
    set(&mut w.p, 1, pt1)?;
    let (min_x, _) = get_pixels_min(&[pt0, pt1, w.pt2], 3)?;
    let offset_x = if min_x < 0.0 { min_x - 100.0 } else { 0.0 };
    set_style(&mut w.p, 2, 5)?;
    let d_mbr = clamp_mbr(w.d_mbr, w.dpi, 20.0, None, Some(500.0));
    let half = get_scaled_size(
        2.0,
        f64::from(w.tg.line_thickness) / 2.0,
        w.tg.pattern_scale,
    );
    let seed = w.pt0;

    let center = extend_line_double(pt0, pt1, -d_mbr / 10.0);
    let (vertical, m) = calc_true_slope_double(pt0, pt1);
    for (low, high) in ticks(vertical, m, offset_x, center, half, seed) {
        set(&mut w.p, 3, Pt { style: 0, ..low })?;
        set(&mut w.p, 4, Pt { style: 0, ..high })?;
    }
    // The far end lists the sides the other way round.
    let center = extend_line_double(pt1, pt0, -d_mbr / 10.0);
    for (low, high) in ticks(vertical, m, offset_x, center, half, seed) {
        set(&mut w.p, 5, Pt { style: 0, ..high })?;
        set(&mut w.p, 6, Pt { style: 0, ..low })?;
    }
    let corner = get(&w.p, 3)?;
    set(&mut w.p, 7, Pt { style: 5, ..corner })?;

    let n = (d_mbr as i32) / 20;
    let a = arrow(get(&w.p, 1)?, get(&w.p, 0)?, n, n, 0)?;
    for (j, pt) in (0..).zip(a) {
        set(&mut w.p, 8 + j, pt)?;
    }
    let a = arrow(get(&w.p, 1)?, get(&w.p, 2)?, n, n, 0)?;
    for (j, pt) in (0..).zip(a) {
        set(&mut w.p, 11 + j, Pt { style: 0, ..pt })?;
    }
    w.ac = 14;
    Ok(())
}
