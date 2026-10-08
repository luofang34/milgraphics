//! Port of the parts of mil-sym-java RenderMultipoints/clsUtilityCPOF.java that
//! the unclipped render path uses: the change 1 areas and range fans
//! (`Change1TacticalAreas` and helpers), `FilterPoints2`, `ClearPixelsStyle`
//! and `LinesWithSeparateFill`.
//!
//! Upstream builds the circles, rectangles and sectors with geodesic
//! computations on latitude/longitude. Here the anchor points are pixels and
//! `meters_per_pixel` converts amplifier distances, so a distance along a
//! bearing is a planar offset (`planar`). Geodesic densification
//! (`SegmentGeoPoints`, `toGeodesic`, `postSegmentFSA`) and clipping are not
//! ported.

pub(crate) mod areas;
pub(crate) mod filter;
mod groups;
mod numeric_fields;
mod planar;
pub(crate) mod range_fan;
mod shapes;

#[cfg(test)]
mod tests;
