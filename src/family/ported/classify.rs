//! Which renderer shapes are proportional to the graphic.

use crate::engine::base::Shape;

use super::from_px;

/// Largest difference on the ground, as a fraction of the graphic's extent,
/// for a shape drawn at two pixel sizes to count as the same.
const SAME_FRACTION: f64 = 1e-4;

/// For each shape of run `a`: whether run `b` drew it at the same place on
/// the ground. Runs that drew different numbers of shapes keep everything in
/// pixels, since their shapes cannot be paired.
pub(super) fn geographic(
    a: &[Shape],
    mpp_a: f64,
    b: &[Shape],
    mpp_b: f64,
    extent: f64,
) -> Vec<bool> {
    if a.len() != b.len() {
        return vec![false; a.len()];
    }
    let tolerance = SAME_FRACTION * extent;
    a.iter()
        .zip(b)
        .map(|(sa, sb)| same(sa, mpp_a, sb, mpp_b, tolerance))
        .collect()
}

fn same(a: &Shape, mpp_a: f64, b: &Shape, mpp_b: f64, tolerance: f64) -> bool {
    let (la, lb) = (a.polylines(), b.polylines());
    // A shape empty at both sizes may still be drawn at others.
    !la.is_empty()
        && a.shape_type == b.shape_type
        && la.len() == lb.len()
        && la.iter().zip(&lb).all(|(pa, pb)| {
            pa.len() == pb.len()
                && pa.iter().zip(pb).all(|(&p, &q)| {
                    let (u, v) = (from_px(p, mpp_a), from_px(q, mpp_b));
                    u.sub(v).len() <= tolerance
                })
        })
}

#[cfg(test)]
mod tests;
