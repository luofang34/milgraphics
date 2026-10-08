//! The pipeline stages the channel code calls that upstream implements in
//! other files (flot.java, DISMSupport.java, clsRenderer2). The integration
//! supplies one implementation; tests use a stub.

use crate::engine::base::{EngineError, Pt, Shape};
use crate::engine::tg::Tg;

/// Calls out of the channel code into other ported modules.
pub(crate) trait ChannelExternals {
    /// Upstream `flot.GetFlotCountDouble`: the number of points the flot
    /// along the first `counter` points of `pts` needs.
    fn flot_count(&self, pts: &[Pt], segment_length: f64, counter: i32)
    -> Result<i32, EngineError>;

    /// Upstream `flot.GetFlotDouble`: replaces `pts` (pre-sized to the larger
    /// of `counter` and the flot count) with the flot and returns its point
    /// count.
    fn flot(&self, pts: &mut [Pt], segment_length: f64, counter: i32) -> Result<i32, EngineError>;

    /// Upstream `DISMSupport.GetDISMCoverDoubleRevC`: the cover glyph for the
    /// first `counter` points of `pts`, returning its point count.
    fn dism_cover_rev_c(
        &self,
        pts: &mut [Pt],
        line_type: i32,
        counter: i32,
    ) -> Result<i32, EngineError>;

    /// The FLOT shapes upstream's `DrawLCSingleLineSegments` gets from
    /// `clsRenderer2.GetLineArray`: a hostile FLOT graphic with the parent's
    /// line thickness drawn along `pixels`.
    fn lc_flot_shapes(&self, parent: &Tg, pixels: Vec<Pt>) -> Result<Vec<Shape>, EngineError>;
}

/// Externals for tests: the flot keeps its input points, the cover glyph is
/// empty and the small-angle FLOT draws one red polyline through the points.
#[cfg(test)]
#[derive(Debug, Default)]
pub(crate) struct StubExternals;

#[cfg(test)]
impl ChannelExternals for StubExternals {
    fn flot_count(&self, _pts: &[Pt], _len: f64, counter: i32) -> Result<i32, EngineError> {
        Ok(counter)
    }

    fn flot(&self, _pts: &mut [Pt], _len: f64, counter: i32) -> Result<i32, EngineError> {
        Ok(counter)
    }

    fn dism_cover_rev_c(&self, _p: &mut [Pt], _lt: i32, _n: i32) -> Result<i32, EngineError> {
        Ok(0)
    }

    fn lc_flot_shapes(&self, _parent: &Tg, pixels: Vec<Pt>) -> Result<Vec<Shape>, EngineError> {
        let mut shape = Shape::new(crate::engine::base::shape_type::POLYLINE);
        for (i, p) in pixels.into_iter().enumerate() {
            if i == 0 {
                shape.move_to(p);
            } else {
                shape.line_to(p);
            }
        }
        Ok(vec![shape])
    }
}
