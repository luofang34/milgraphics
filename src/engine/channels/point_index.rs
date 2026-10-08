//! Checked access to point arrays by Java `int` index, so the channel code
//! can keep upstream's index arithmetic and still fail (not panic) where
//! Java throws.

use crate::engine::base::{At, EngineError, Pt, idx};

/// The point at Java index `i`.
pub(crate) fn at_i(pts: &[Pt], i: i32) -> Result<Pt, EngineError> {
    pts.at(idx(i, pts.len())?)
}

/// The point at Java index `i`, for update.
pub(crate) fn mut_i(pts: &mut [Pt], i: i32) -> Result<&mut Pt, EngineError> {
    let len = pts.len();
    pts.at_mut(idx(i, len)?)
}

/// Stores `p` at Java index `i`.
pub(crate) fn set_i(pts: &mut [Pt], i: i32, p: Pt) -> Result<(), EngineError> {
    *mut_i(pts, i)? = p;
    Ok(())
}

/// A new array of `n` zero points, as `new POINT2[n]` followed by
/// `InitializePOINT2Array`.
pub(crate) fn new_pts(n: i32) -> Result<Vec<Pt>, EngineError> {
    let len = usize::try_from(n).map_err(|_| EngineError::Index {
        index: i64::from(n),
        len: 0,
    })?;
    Ok(vec![Pt::default(); len])
}

/// The first `n` points of `pts` as a new array.
pub(crate) fn first_n(pts: &[Pt], n: i32) -> Result<Vec<Pt>, EngineError> {
    let mut out = Vec::new();
    for k in 0..n {
        out.push(at_i(pts, k)?);
    }
    Ok(out)
}
