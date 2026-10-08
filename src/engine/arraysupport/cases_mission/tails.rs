//! FOLLA, FOLSP and FERRY: lines whose arrowheads are written at the end of
//! the point array.

use crate::engine::arraysupport::work::{MAX_LENGTH, MIN_LENGTH, Work, get, set, set_style};
use crate::engine::base::{EngineError, Pt};
use crate::engine::lineutility::arrow::get_arrow_head4_double;
use crate::engine::lineutility::basics::reverse_points_double2;
use crate::engine::lineutility::extend::extend_line_double;

/// Upstream's clamp of `dMBR / div` to the arrow size limits, then to
/// `[floor, ceiling]` when given.
fn clamp_mbr(w: &Work<'_>, div: f64, floor: Option<f64>, ceiling: f64) -> f64 {
    let mut d = w.d_mbr;
    if d / div > MAX_LENGTH * w.dpi {
        d = div * MAX_LENGTH * w.dpi;
    }
    if d / div < MIN_LENGTH * w.dpi {
        d = div * MIN_LENGTH * w.dpi;
    }
    if let Some(f) = floor {
        if d < f * w.dpi {
            d = f * w.dpi;
        }
    }
    if d > ceiling * w.dpi {
        d = ceiling * w.dpi;
    }
    d
}

/// Writes the three arrowhead points to `p[at..at + 3]` in order.
fn put_arrow(p: &mut [Pt], at: i32, arrow: &[Pt; 3], reversed: bool) -> Result<(), EngineError> {
    for (k, i) in (0_i32..3).zip(0_usize..) {
        let src = if reversed { 2 - i } else { i };
        set(p, at + k, arrow.get(src).copied().unwrap_or_default())?;
    }
    Ok(())
}

/// `p[dst] = new POINT2(p[src])`, with `style` when given.
fn copy_to(p: &mut [Pt], dst: i32, src: i32, style: Option<i32>) -> Result<(), EngineError> {
    let mut pt = get(p, src)?;
    if let Some(st) = style {
        pt.style = st;
    }
    set(p, dst, pt)
}

/// FOLLA: a dashed tail and two arrowheads.
pub(crate) fn folla(w: &mut Work<'_>) -> Result<(), EngineError> {
    reverse_points_double2(&mut w.p, w.save)?;
    let d = clamp_mbr(w, 10.0, None, 150.0);
    let n = w.vbl;
    let size = (d as i32) / 10;
    let mut arrow = [Pt::default(); 3];
    let (p0, p1) = (get(&w.p, 0)?, get(&w.p, 1)?);
    set(&mut w.p, 0, extend_line_double(p1, p0, -2.0 * d / 10.0))?;
    for k in 0..n - 14 {
        set_style(&mut w.p, k, 18)?;
    }
    set_style(&mut w.p, n - 15, 5)?;
    let (p0, p1) = (get(&w.p, 0)?, get(&w.p, 1)?);
    let pt0 = extend_line_double(p1, p0, 5.0 * d / 10.0);
    get_arrow_head4_double(pt0, p0, size, size, &mut arrow, 0)?;
    put_arrow(&mut w.p, n - 14, &arrow, false)?;
    let pt3 = extend_line_double(p1, p0, d / 10.0);
    get_arrow_head4_double(pt0, pt3, size, size, &mut arrow, 0)?;
    set_style(&mut w.p, n - 12, 0)?;
    set(
        &mut w.p,
        n - 11,
        Pt {
            style: 0,
            ..arrow[2]
        },
    )?;
    set(
        &mut w.p,
        n - 10,
        Pt {
            style: 0,
            ..arrow[0]
        },
    )?;
    copy_to(&mut w.p, n - 9, n - 14, Some(5))?;
    let (a, b) = (get(&w.p, n - 16)?, get(&w.p, n - 15)?);
    get_arrow_head4_double(a, b, size, size, &mut arrow, 0)?;
    put_arrow(&mut w.p, n - 8, &arrow, false)?;
    set_style(&mut w.p, n - 6, 0)?;
    // The first point becomes the tip of the last arrowhead.
    let tip = extend_line_double(a, b, -0.75 * d / 10.0);
    set(&mut w.p, 1, Pt { style: 5, ..tip })?;
    let head = (d / 10.0) as i32;
    get_arrow_head4_double(a, tip, head, head, &mut arrow, 0)?;
    put_arrow(&mut w.p, n - 5, &arrow, true)?;
    set_style(&mut w.p, n - 5, 0)?;
    copy_to(&mut w.p, n - 2, n - 8, Some(5))?;
    copy_to(&mut w.p, n - 1, n - 7, None)?;
    w.ac = 16;
    Ok(())
}

/// FOLSP: a tail and two arrowheads, the last one filled.
pub(crate) fn folsp(w: &mut Work<'_>) -> Result<(), EngineError> {
    reverse_points_double2(&mut w.p, w.save)?;
    let d = clamp_mbr(w, 15.0, Some(100.0), 500.0);
    let n = w.vbl;
    let size = (d as i32) / 20;
    let mut arrow = [Pt::default(); 3];
    let (p0, p1) = (get(&w.p, 0)?, get(&w.p, 1)?);
    set(&mut w.p, 0, extend_line_double(p1, p0, -d / 8.75))?;
    set_style(&mut w.p, n - 15, 5)?;
    let (p0, p1) = (get(&w.p, 0)?, get(&w.p, 1)?);
    let pt0 = extend_line_double(p1, p0, d / 4.0);
    get_arrow_head4_double(pt0, p0, size, size, &mut arrow, 0)?;
    put_arrow(&mut w.p, n - 14, &arrow, false)?;
    set_style(&mut w.p, n - 12, 0)?;
    let pt3 = extend_line_double(p1, p0, d / 15.0);
    get_arrow_head4_double(pt0, pt3, size, size, &mut arrow, 0)?;
    for k in 0..3 {
        let a = arrow.get(2 - k).copied().unwrap_or_default();
        set(&mut w.p, n - 11 + k as i32, Pt { style: 0, ..a })?;
    }
    copy_to(&mut w.p, n - 8, n - 14, Some(5))?;
    let (a, b) = (get(&w.p, n - 16)?, get(&w.p, n - 15)?);
    get_arrow_head4_double(a, b, size, size, &mut arrow, 9)?;
    put_arrow(&mut w.p, n - 7, &arrow, false)?;
    for k in (1..=4).rev() {
        set_style(&mut w.p, n - k, 5)?;
    }
    w.ac = 12;
    Ok(())
}

/// FERRY: arrowheads at both ends of the two-way line.
pub(crate) fn ferry(w: &mut Work<'_>) -> Result<(), EngineError> {
    let d = clamp_mbr(w, 10.0, None, 250.0);
    let n = w.vbl;
    let size = (d as i32) / 10;
    let mut arrow = [Pt::default(); 3];
    let (a, b) = (get(&w.p, n - 8)?, get(&w.p, n - 7)?);
    get_arrow_head4_double(a, b, size, size, &mut arrow, 9)?;
    put_arrow(&mut w.p, n - 6, &arrow, false)?;
    let (a, b) = (get(&w.p, 1)?, get(&w.p, 0)?);
    get_arrow_head4_double(a, b, size, size, &mut arrow, 9)?;
    put_arrow(&mut w.p, n - 3, &arrow, false)?;
    w.ac = 8;
    Ok(())
}
