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
    /// The symbol's draw rule and point range, as upstream's `MSInfo` has them.
    pub(crate) ms_info: Option<super::line_type::classes::MsInfo>,
    /// The operator's colours, in place of the symbol's.
    pub(crate) style: Style,
}

/// Colours chosen by the operator; absent ones follow the symbol.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Style {
    pub(crate) line_color: Option<crate::style::Rgba>,
    pub(crate) fill_color: Option<crate::style::Rgba>,
}

impl Style {
    /// The overrides of a definition.
    pub(crate) fn of(style: &crate::definition::StyleOverrides) -> Self {
        let parse = |c: &Option<String>| c.as_deref().and_then(crate::style::Rgba::parse_hex);
        Self {
            line_color: parse(&style.line_color),
            fill_color: parse(&style.fill_color),
        }
    }
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
    /// The graphic's outline is left out under the text.
    pub(crate) knockout: bool,
}

/// What the engine draws for one graphic.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Output {
    pub(crate) shapes: Vec<Shape>,
    pub(crate) labels: Vec<Label>,
}

/// Draws `input`, or says why upstream would draw nothing.
pub(crate) fn draw(input: &Input<'_>) -> Result<Output, EngineError> {
    let mut out = super::pipeline::render(input)?;
    super::edition::adjust(input, &mut out);
    Ok(out)
}
