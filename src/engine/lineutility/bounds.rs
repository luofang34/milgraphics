//! Port of the bounding-box helpers of lineutility.java.

use crate::engine::base::{At, EngineError, Pt, idx};

/// Upstream `CalcMBR`: `(ulx, uly, lrx, lry)` of the first `numpts` points.
fn calc_mbr(pts: &[Pt], numpts: i32) -> Result<(f64, f64, f64, f64), EngineError> {
    let mut b = (f64::MAX, f64::MAX, -f64::MAX, -f64::MAX);
    for j in 0..numpts {
        let p = pts.at(idx(j, pts.len())?)?;
        if p.x > b.2 {
            b.2 = p.x;
        }
        if p.y > b.3 {
            b.3 = p.y;
        }
        if p.x < b.0 {
            b.0 = p.x;
        }
        if p.y < b.1 {
            b.1 = p.y;
        }
    }
    Ok(b)
}

/// Upstream `CalcMBRPoints`: sets the x/y of `ul` and `lr` to the bounding
/// box corners of the first `numpts` points.
pub(crate) fn calc_mbr_points(
    pts: &[Pt],
    numpts: i32,
    ul: &mut Pt,
    lr: &mut Pt,
) -> Result<(), EngineError> {
    let (ulx, uly, lrx, lry) = calc_mbr(pts, numpts)?;
    ul.x = ulx;
    ul.y = uly;
    lr.x = lrx;
    lr.y = lry;
    Ok(())
}

/// Upstream `MBRDistance`: the diagonal of the bounding box of the first
/// `numpts` points (the larger side when the diagonal is zero or infinite).
pub(crate) fn mbr_distance(pts: &[Pt], numpts: i32) -> Result<f64, EngineError> {
    let (ulx, uly, lrx, lry) = calc_mbr(pts, numpts)?;
    let mut result = ((lrx - ulx) * (lrx - ulx) + (lry - uly) * (lry - uly)).sqrt();
    let xdist = (lrx - ulx).abs();
    let ydist = (lry - uly).abs();
    let max = if ydist > xdist { ydist } else { xdist };
    if (result == 0.0 || result.is_infinite()) && max > 0.0 {
        result = max;
    }
    Ok(result)
}

/// Upstream `Reverse2Points`: swaps the positions (not the styles) of two
/// points.
pub(crate) fn reverse2_points(pt1: &mut Pt, pt2: &mut Pt) {
    let (x, y) = (pt1.x, pt1.y);
    pt1.x = pt2.x;
    pt1.y = pt2.y;
    pt2.x = x;
    pt2.y = y;
}
