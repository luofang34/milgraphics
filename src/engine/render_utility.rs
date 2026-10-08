//! Port of the pieces of mil-sym-java RenderMultipoints/clsRenderer.java,
//! clsRenderer2.java, clsUtility.java, clsUtilityGE.java and
//! clsClipPolygon2.java that sit around the shape builders in the unclipped
//! render path: interpolation of dense points, the MSR segment shapes, the
//! feint/decoy/dummy indicator, separate fill shapes, hatch fills and the
//! final flattening of shapes into polylines. Clipping is not ported.

pub(crate) mod fdi;
pub(crate) mod fills;
pub(crate) mod hatch;
pub(crate) mod interpolate;
pub(crate) mod msr;
pub(crate) mod polylines;

#[cfg(test)]
mod tests;
