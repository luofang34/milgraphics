//! The spike tip the occluded, OFY and stationary fronts share.

use crate::engine::base::{At, EngineError, Pt};
use crate::engine::lineutility::extend::extend_directed_line;

/// Inputs of one spike's tip: the segment `a` to `b`, the spike's base
/// midpoint `pt0` and the spike size.
#[derive(Clone, Copy, Debug)]
pub(crate) struct SpikeTip {
    pub(crate) a: Pt,
    pub(crate) b: Pt,
    pub(crate) pt0: Pt,
    pub(crate) size: f64,
    pub(crate) too_long: bool,
    pub(crate) not_vertical: bool,
    /// The style the tip point ends with.
    pub(crate) style: i32,
}

/// Writes the spike's tip at `spike[*n]` and advances `n`: perpendicular to
/// the segment at `pt0` (above for a rightward segment, below otherwise),
/// the segment end when the spike would overrun it, or a horizontal offset
/// for a vertical segment.
pub(crate) fn push_spike_tip(
    spike: &mut [Pt],
    n: &mut usize,
    t: &SpikeTip,
) -> Result<(), EngineError> {
    if t.too_long {
        *spike.at_mut(*n)? = t.b;
    } else if t.not_vertical {
        let dir = if t.a.x < t.b.x { 2 } else { 3 };
        let mut p = extend_directed_line(t.a, t.b, t.pt0, dir, t.size);
        p.style = 0;
        *spike.at_mut(*n)? = p;
    } else {
        let p = spike.at_mut(*n)?;
        p.x = if t.a.y > t.b.y {
            t.pt0.x - t.size
        } else {
            t.pt0.x + t.size
        };
        p.y = t.pt0.y;
    }
    *n += 1;
    spike.at_mut(*n - 1)?.style = t.style;
    Ok(())
}
