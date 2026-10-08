//! The part of the screen a view shows, so generators whose output grows
//! with a line's pixel length can emit only the repeats near it. Repeats are
//! still placed where the whole line puts them, so a pattern does not shift
//! as the view moves.

use super::base::Pt;

/// A rectangle in pixels, y down.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PixelBox {
    pub(crate) min_x: f64,
    pub(crate) min_y: f64,
    pub(crate) max_x: f64,
    pub(crate) max_y: f64,
}

impl PixelBox {
    /// The distances from `a` toward `b`, between 0 and the segment's
    /// length, over which the segment is inside the box; `None` when it
    /// misses the box.
    pub(crate) fn span(&self, a: Pt, b: Pt) -> Option<(f64, f64)> {
        let (dx, dy) = (b.x - a.x, b.y - a.y);
        let (mut t0, mut t1) = (0.0_f64, 1.0_f64);
        for (p, q) in [
            (-dx, a.x - self.min_x),
            (dx, self.max_x - a.x),
            (-dy, a.y - self.min_y),
            (dy, self.max_y - a.y),
        ] {
            if p == 0.0 {
                if q < 0.0 {
                    return None;
                }
            } else if p < 0.0 {
                t0 = t0.max(q / p);
            } else {
                t1 = t1.min(q / p);
            }
        }
        let length = dx.hypot(dy);
        (t0 <= t1).then_some((t0 * length, t1 * length))
    }

    /// The indices `k` in `0..n` of repeats placed `step` apart from
    /// `first` along a segment that are within `margin` of the visible
    /// span; all of them without a box.
    pub(crate) fn repeats(
        visible: Option<&Self>,
        seg: (Pt, Pt),
        (first, step, n): (f64, f64, i32),
        margin: f64,
    ) -> core::ops::Range<i32> {
        let Some(bx) = visible else {
            return 0..n;
        };
        let Some((lo, hi)) = bx.span(seg.0, seg.1) else {
            return 0..0;
        };
        if step <= 0.0 {
            return 0..n;
        }
        let from = ((lo - margin - first) / step).floor().max(0.0);
        let to = ((hi + margin - first) / step).ceil() + 1.0;
        let clamp = |v: f64| v.min(f64::from(n)) as i32;
        clamp(from)..clamp(to)
    }
}

#[cfg(test)]
mod tests;
