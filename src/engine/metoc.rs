//! Port of mil-sym-java JavaTacticalRenderer/clsMETOC.java: the shapes of
//! the weather and oceanographic (METOC) line and area types. Most types
//! hand their outline to the general line builder; the spline types, the
//! ice openings and the leading line are built here.
//!
//! Three upstream calls reach into code that lives elsewhere. They are
//! behind [`MetocSupport`] so this module does not depend on it:
//! `arraysupport.GetLineArray2`, `clsChannelUtility.GetPartitions2` and
//! `Channels.GetChannel1Double`.
//!
//! The weather-type lookup (`getWeatherLinetype`, `IsWeather`) is
//! [`crate::engine::line_type`].
//!
//! Upstream's conversion from paths to polylines has no case for cubic
//! segments, so the cubic that the solid-curve types (ISOBAR, ISOTHERM,
//! COASTLINE, ...) draw does not reach the output; only their `_GE`
//! variants, which add the sampled points, do. This port keeps that
//! behaviour; see [`path::GeneralPath::into_path_ops`].

pub(crate) mod bezier;
pub(crate) mod get_shape;
pub(crate) mod ice_openings;
pub(crate) mod itcz;
pub(crate) mod parallel_lines;
pub(crate) mod path;
pub(crate) mod properties;
pub(crate) mod shape_properties;
pub(crate) mod splines;

#[cfg(test)]
mod tests;

use crate::engine::base::{EngineError, Shape};
use crate::engine::tg::Tg;

/// Upstream `P1`: a run of segments, as indexes of the first and last
/// control point it covers (the run draws through `end + 1`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Partition {
    pub(crate) start: i32,
    pub(crate) end: i32,
}

/// The code `clsMETOC` calls outside its own file.
pub(crate) trait MetocSupport {
    /// `arraysupport.GetLineArray2(tg, tg.Pixels, shapes, null, null)`:
    /// appends the shapes of `tg.line_type` built from `tg.pixels`.
    fn line_array(&self, tg: &mut Tg, shapes: &mut Vec<Shape>) -> Result<(), EngineError>;

    /// `clsChannelUtility.GetPartitions2(tg)`: the runs of `tg.pixels` long
    /// enough to draw as channels. Upstream fails (null) for fewer than two
    /// points.
    fn partitions(&self, tg: &Tg) -> Result<Vec<Partition>, EngineError>;

    /// `Channels.GetChannel1Double(tg, line, line, out, n, n, width, 0,
    /// null)` for a fresh graphic of type `CHANNEL`. `line` holds `n`
    /// `x, y` pairs and `out` is zeroed with `6 * n` entries. An error
    /// leaves `out` as far as it was filled; upstream catches it.
    fn channel_points(
        &self,
        line: &[f64],
        out: &mut [f64],
        channel_width: i32,
    ) -> Result<(), EngineError>;
}

/// A fill pattern drawn from a raster image upstream (`TexturePaint`). The
/// image is not part of the port: the host picks the pattern by `line_type`
/// and applies it to the shape at `shape_index`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PatternFill {
    pub(crate) shape_index: usize,
    pub(crate) line_type: i32,
}

/// `arraysupport.getScaledSize`: `size` grows with the line width above the
/// default of 3, by `pattern_scale` per two pixels, up to a width of 100.
pub(crate) fn scaled_size(size: f64, line_width: f64, pattern_scale: f64) -> f64 {
    if line_width <= 3.0 {
        return size;
    }
    let width = if line_width > 100.0 {
        100.0
    } else {
        line_width
    };
    size * (1.0 + ((width - 3.0) / 2.0) * pattern_scale)
}
