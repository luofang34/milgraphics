//! Port of mil-sym-java JavaLineArray/lineutility.java: the pixel-space
//! geometry primitives the line builders share (distances, slopes, line
//! extension, arrowheads, arcs and circles, ditch spikes, bounding
//! boxes and exterior offsets).
//!
//! Java overloads get distinct Rust names: the four-argument
//! `ExtendAlongLineDouble` is `extend_along_line_double_style`, the
//! `Point2D`/`Object[]` variants collapse onto the `Pt` versions, and the
//! second `PointRelativeToLine` is `point_relative_to_line_at`.
//! `ref<...>` out-parameters are returned values.

pub(crate) mod arc;
pub(crate) mod arrow;
pub(crate) mod basics;
pub(crate) mod bounds;
pub(crate) mod channel_pixels;
pub(crate) mod circle;
pub(crate) mod ditch;
pub(crate) mod extend;
pub(crate) mod exterior;
pub(crate) mod intersect;
pub(crate) mod relative;
pub(crate) mod saafr;
pub(crate) mod slope;
pub(crate) mod squall;
pub(crate) mod transform;

#[cfg(test)]
mod tests;

/// Direction code: extend to the left of the line.
pub(crate) const EXTEND_LEFT: i32 = 0;
/// Direction code: extend to the right of the line.
pub(crate) const EXTEND_RIGHT: i32 = 1;
/// Direction code: extend above the line.
pub(crate) const EXTEND_ABOVE: i32 = 2;
/// Direction code: extend below the line.
pub(crate) const EXTEND_BELOW: i32 = 3;
