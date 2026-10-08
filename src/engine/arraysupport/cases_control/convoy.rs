//! MSDZ, CONVOY and HCONVOY.

use crate::engine::arraysupport::work::{Work, get, set, set_style};
use crate::engine::base::{EngineError, Pt};
use crate::engine::lineutility::arrow::get_arrow_head4_double;
use crate::engine::lineutility::basics::calc_distance_double;
use crate::engine::lineutility::circle::calc_circle_double;
use crate::engine::lineutility::extend::{
    extend_along_line_double, extend_directed_line, extend_line_double, extend_line2_double,
};
use crate::engine::lineutility::slope::calc_true_slope_double;

/// MSDZ: concentric circles about the first point through the others.
pub(crate) fn msdz(w: &mut Work<'_>) -> Result<(), EngineError> {
    let pt3 = get(&w.p, 3)?;
    let mut circle = [Pt::default(); 100];
    let center = w.pt0;
    let rings = [(w.pt1, 0), (w.pt2, 100)];
    for (through, at) in rings {
        calc_circle_double(
            center,
            calc_distance_double(center, through),
            100,
            &mut circle,
            0,
        )?;
        for (j, c) in (0..).zip(circle.iter()) {
            set(&mut w.p, at + j, *c)?;
        }
        set_style(&mut w.p, at + 99, 5)?;
    }
    if w.save == 4 {
        calc_circle_double(
            center,
            calc_distance_double(center, pt3),
            100,
            &mut circle,
            0,
        )?;
        for (j, c) in (0..).zip(circle.iter()) {
            set(&mut w.p, 200 + j, *c)?;
        }
    }
    w.ac = w.vbl;
    Ok(())
}

/// The bounding-box size limited to the convoy range.
fn convoy_size(w: &Work<'_>) -> f64 {
    w.d_mbr.max(150.0 * w.dpi).min(500.0 * w.dpi)
}

/// CONVOY: a box with an arrowhead at the front.
pub(crate) fn convoy(w: &mut Work<'_>) -> Result<(), EngineError> {
    let width = convoy_size(w) / 25.0;
    let (p0, p1) = (get(&w.p, 0)?, get(&w.p, 1)?);
    let (_, m) = calc_true_slope_double(p1, p0);
    let pt0 = extend_line2_double(p1, p0, -width * 3.0, 0);
    let (d0, d1) = if m < 1.0 { (2, 3) } else { (0, 1) };
    set(&mut w.p, 0, extend_directed_line(pt0, p1, pt0, d0, width))?;
    set(&mut w.p, 1, extend_directed_line(pt0, p1, p1, d0, width))?;
    set(&mut w.p, 2, extend_directed_line(pt0, p1, p1, d1, width))?;
    set(&mut w.p, 3, extend_directed_line(pt0, p1, pt0, d1, width))?;
    let pt2 = extend_line_double(p1, pt0, width * 3.0);
    let mut arrow = [Pt::default(); 3];
    let n = (width * 3.0) as i32;
    get_arrow_head4_double(pt0, pt2, n, n, &mut arrow, 0)?;
    let d = calc_distance_double(get(&w.p, 0)?, arrow[0]);
    let d_other = calc_distance_double(get(&w.p, 3)?, arrow[0]);
    set_style(&mut w.p, 3, 5)?;
    if d < d_other {
        let first = get(&w.p, 0)?;
        set(&mut w.p, 4, Pt { style: 0, ..first })?;
        let last = get(&w.p, 3)?;
        set(&mut w.p, 8, last)?;
    } else {
        // Upstream aliases slot 4 to slot 3, so clearing one clears both.
        set_style(&mut w.p, 3, 0)?;
        let aliased = get(&w.p, 3)?;
        set(&mut w.p, 4, aliased)?;
        let first = get(&w.p, 0)?;
        set(&mut w.p, 8, first)?;
    }
    for (k, a) in (5..).zip(arrow.iter()) {
        set(&mut w.p, k, Pt { style: 0, ..*a })?;
    }
    w.ac = 9;
    Ok(())
}

/// HCONVOY: a box with an arrowhead drawn inside its front.
pub(crate) fn hconvoy(w: &mut Work<'_>) -> Result<(), EngineError> {
    let width = convoy_size(w) / 25.0;
    let (pt0, pt1) = (get(&w.p, 0)?, get(&w.p, 1)?);
    let pt2 = extend_along_line_double(pt0, pt1, width * 2.0);
    let n = (width as i32) * 2;
    let mut arrow = [Pt::default(); 3];
    get_arrow_head4_double(pt0, pt2, n, n, &mut arrow, 0)?;
    let (_, m) = calc_true_slope_double(pt1, pt2);
    let (d0, d1) = if m < 1.0 { (2, 3) } else { (0, 1) };
    set(&mut w.p, 0, extend_directed_line(pt2, pt1, pt2, d0, width))?;
    set(&mut w.p, 1, extend_directed_line(pt2, pt1, pt1, d0, width))?;
    set(&mut w.p, 2, extend_directed_line(pt2, pt1, pt1, d1, width))?;
    set(&mut w.p, 3, extend_directed_line(pt2, pt1, pt2, d1, width))?;
    let first = get(&w.p, 0)?;
    set(&mut w.p, 4, first)?;
    set(&mut w.p, 5, Pt { style: 0, ..pt2 })?;
    set(&mut w.p, 6, arrow[1])?;
    set(&mut w.p, 7, arrow[0])?;
    set(
        &mut w.p,
        8,
        Pt {
            style: 0,
            ..arrow[2]
        },
    )?;
    set(&mut w.p, 9, arrow[1])?;
    w.ac = 10;
    Ok(())
}
