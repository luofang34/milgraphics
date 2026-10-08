//! What milgraphics gives the engine and gets back: one graphic in pixel
//! space, as upstream's `MultiPointHandler` would hand it to the renderer
//! after projecting its control points.

use super::base::{EngineError, Pt, Shape};
use crate::modifier::Modifiers;
use crate::sidc::SymbolId;

/// One graphic to draw.
pub(crate) struct Input<'a> {
    /// Upstream line type (a `tactical_lines` constant).
    pub(crate) line_type: i32,
    /// The symbol, for its version, identity (colour) and status (dash).
    pub(crate) symbol: &'a SymbolId,
    /// Control points in pixels, y down.
    pub(crate) pixels: Vec<Pt>,
    /// The amplifiers, as stored.
    pub(crate) modifiers: &'a Modifiers,
    /// Ground metres per pixel at the graphic, for amplifiers given in metres.
    pub(crate) meters_per_pixel: f64,
    /// Width of `text` in pixels in the label font.
    pub(crate) text_width: &'a dyn Fn(&str) -> f64,
}

impl core::fmt::Debug for Input<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Input")
            .field("line_type", &self.line_type)
            .field("symbol", &self.symbol)
            .field("pixels", &self.pixels)
            .field("modifiers", &self.modifiers)
            .field("meters_per_pixel", &self.meters_per_pixel)
            .finish_non_exhaustive()
    }
}

/// Label justification, as upstream's `ShapeInfo` gives it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Justify {
    Left,
    Center,
    Right,
}

/// A label in pixels.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Label {
    pub(crate) text: String,
    /// Upstream's modifier position (the text anchor).
    pub(crate) x: f64,
    pub(crate) y: f64,
    /// Rotation in degrees, clockwise on screen.
    pub(crate) angle_deg: f64,
    pub(crate) justify: Justify,
}

/// What the engine draws for one graphic.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Output {
    pub(crate) shapes: Vec<Shape>,
    pub(crate) labels: Vec<Label>,
}

/// Draws `input`, or says why upstream would draw nothing.
pub(crate) fn draw(input: &Input<'_>) -> Result<Output, EngineError> {
    Err(EngineError::LineType(input.line_type))
}
