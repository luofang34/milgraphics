//! Line and fill styles, resolved from the symbol and its overrides.

use crate::definition::StyleOverrides;
use crate::sidc::SymbolId;

#[cfg(test)]
mod tests;

/// An sRGB colour with alpha.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Rgba {
    /// Red.
    pub r: u8,
    /// Green.
    pub g: u8,
    /// Blue.
    pub b: u8,
    /// Alpha, 255 opaque.
    pub a: u8,
}

impl Rgba {
    /// Opaque black.
    pub const BLACK: Self = Self::opaque(0, 0, 0);
    /// Opaque red.
    pub const RED: Self = Self::opaque(255, 0, 0);

    /// An opaque colour.
    pub const fn opaque(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Parses `#rrggbb` or `#rrggbbaa`.
    pub fn parse_hex(text: &str) -> Option<Self> {
        let hex = text.strip_prefix('#')?;
        if !(hex.len() == 6 || hex.len() == 8) || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }
        let byte = |i: usize| {
            hex.get(i..i + 2)
                .and_then(|h| u8::from_str_radix(h, 16).ok())
        };
        Some(Self {
            r: byte(0)?,
            g: byte(2)?,
            b: byte(4)?,
            a: if hex.len() == 8 { byte(6)? } else { 255 },
        })
    }

    /// `#rrggbbaa`.
    pub fn to_hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}{:02x}", self.r, self.g, self.b, self.a)
    }
}

/// Dash patterns the engine can draw as fixed layers. Lengths are in
/// multiples of the line width, matching how map engines scale dashes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DashPattern {
    /// Continuous.
    Solid,
    /// Equal dashes and gaps of two line widths: planned or anticipated.
    Dashed,
}

impl DashPattern {
    /// Dash and gap lengths in line widths; empty for solid.
    pub fn array(self) -> &'static [f64] {
        match self {
            Self::Solid => &[],
            Self::Dashed => &[2.0, 2.0],
        }
    }
}

/// A line style. Width is in screen pixels: the standard scales line width
/// with the display, not with the ground.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stroke {
    /// Colour.
    pub color: Rgba,
    /// Width in pixels.
    pub width_px: f64,
    /// Dash pattern.
    pub dash: DashPattern,
}

/// How an area or arrowhead interior is painted.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Fill {
    /// Not filled.
    None,
    /// Filled with a colour.
    Solid(Rgba),
}

/// Default line width in pixels.
pub const DEFAULT_LINE_WIDTH_PX: f64 = 3.0;

/// The palette a graphic is drawn with.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Palette {
    pub(crate) line: Stroke,
    pub(crate) solid_line: Stroke,
    pub(crate) fill: Option<Rgba>,
}

/// Resolves colours and dash from the symbol and the operator's overrides.
///
/// Control measures are black, or red for hostile and suspect identities;
/// planned/anticipated status (digit 7 = 1) is dashed.
pub(crate) fn palette(symbol: &SymbolId, overrides: &StyleOverrides) -> Palette {
    let base = match symbol.identity() {
        5 | 6 => Rgba::RED,
        _ => Rgba::BLACK,
    };
    let color = overrides
        .line_color
        .as_deref()
        .and_then(Rgba::parse_hex)
        .unwrap_or(base);
    let dash = if symbol.status() == 1 {
        DashPattern::Dashed
    } else {
        DashPattern::Solid
    };
    let solid_line = Stroke {
        color,
        width_px: DEFAULT_LINE_WIDTH_PX,
        dash: DashPattern::Solid,
    };
    Palette {
        line: Stroke { dash, ..solid_line },
        solid_line,
        fill: overrides.fill_color.as_deref().and_then(Rgba::parse_hex),
    }
}
