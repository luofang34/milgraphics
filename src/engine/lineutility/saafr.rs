//! Port of the SAAFR (standard use army aircraft flight route) helpers of
//! lineutility.java.

use super::extend::{extend_directed_line, extend_line2_double};
use super::slope::calc_true_slope_double;
use crate::engine::base::{At, EngineError, Pt};

/// Upstream `GetSAAFRMiddleLine`: the shortened centre line of each leg, two
/// points per leg. A segment point's `style` holds the leg's half width.
pub(crate) fn get_saafr_middle_line(pts: &[Pt]) -> Result<Vec<Pt>, EngineError> {
    let legs = pts
        .iter()
        .take(pts.len().saturating_sub(1))
        .filter(|p| p.style > 0)
        .count();
    let capacity = legs * 2;
    let mut out: Vec<Pt> = Vec::with_capacity(capacity);
    let mut last_seg: Option<Pt> = None;
    for (j, p) in pts.iter().enumerate() {
        if p.style < 0 && j + 1 != pts.len() {
            continue;
        }
        if let Some(first_seg) = last_seg {
            let last = *p;
            last_seg = Some(last);
            let d_mrr = f64::from(first_seg.style);
            let pt0 = extend_line2_double(last, first_seg, -d_mrr, 0);
            let pt1 = extend_line2_double(first_seg, last, -d_mrr, 5);
            if out.len() + 2 > capacity {
                return Err(EngineError::Index {
                    index: i64::try_from(out.len() + 1).unwrap_or(i64::MAX),
                    len: capacity,
                });
            }
            out.push(pt0);
            out.push(pt1);
        } else {
            last_seg = Some(*p);
        }
    }
    Ok(out)
}

/// The four side points of a leg: the first leg point and the second, each
/// offset in `first` and `second` directions by `d_mrr`.
fn side_points(pts: &[Pt], d_mrr: f64) -> Result<[Pt; 4], EngineError> {
    let (a, b) = (pts.at(0)?, pts.at(1)?);
    let (_, m) = calc_true_slope_double(a, b);
    let (d1, d2) = if m < 1.0 { (2, 3) } else { (0, 1) };
    Ok([
        extend_directed_line(a, b, a, d1, d_mrr),
        extend_directed_line(a, b, b, d1, d_mrr),
        extend_directed_line(a, b, a, d2, d_mrr),
        extend_directed_line(a, b, b, d2, d_mrr),
    ])
}

/// Upstream `GetSAAFRSegment`: replaces the two leg points with the six
/// points of the shortened centre line and the two side lines. `line_type`
/// is unused upstream and so omitted.
pub(crate) fn get_saafr_segment(pts: &mut [Pt], d_mrr: f64) -> Result<(), EngineError> {
    let (a, b) = (pts.at(0)?, pts.at(1)?);
    let pt1 = extend_line2_double(a, b, -d_mrr, 5);
    let pt0 = extend_line2_double(b, a, -d_mrr, 0);
    let [mut pt2, mut pt3, mut pt4, mut pt5] = side_points(pts, d_mrr)?;
    pt2.style = 0;
    pt3.style = 5;
    pt4.style = 0;
    pt5.style = 5;
    *pts.at_mut(0)? = Pt { style: 5, ..pt0 };
    *pts.at_mut(1)? = pt1;
    *pts.at_mut(2)? = pt2;
    *pts.at_mut(3)? = pt3;
    *pts.at_mut(4)? = pt4;
    *pts.at_mut(5)? = pt5;
    Ok(())
}

/// Upstream `GetSAAFRFillSegment`: replaces the leg points with the four
/// corners of the fill quad.
pub(crate) fn get_saafr_fill_segment(pts: &mut [Pt], d_mrr: f64) -> Result<(), EngineError> {
    let [pt2, pt3, pt4, pt5] = side_points(pts, d_mrr)?;
    // Check capacity before writing so a short slice is left untouched.
    pts.at(3)?;
    *pts.at_mut(0)? = pt2;
    *pts.at_mut(1)? = pt3;
    *pts.at_mut(2)? = pt5;
    *pts.at_mut(3)? = pt4;
    Ok(())
}
