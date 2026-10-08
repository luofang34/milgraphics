//! Replaces mil-sym-java's `RendererSettings` singleton and the static flags
//! the multipoint path reads (`Channels._client`, `Channels._shiftLines`,
//! `Modifier2.fillAlphaCanObscureText`). Every value is explicit and passed
//! to the code that needs it; defaults are the ones the oracle runs with.

#[cfg(test)]
mod tests;

/// Upstream `CELineArray._client` / `Channels._client`: the client whose
/// conventions the pipeline follows. Always "ge" (no echelon gap in the
/// boundary line, GE spline types).
pub(crate) const CLIENT: &str = "ge";

/// Upstream `Channels._shiftLines`: channel edges are shifted for the
/// arrowhead.
pub(crate) const SHIFT_LINES: bool = true;

/// The DPI at which upstream's pixel constants are defined.
pub(crate) const BASE_DPI: i32 = 96;

/// A label font: Java `Font(family, style, size)` with the style reduced to
/// the bold flag the renderer uses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FontSpec {
    /// Font family name.
    pub(crate) family: String,
    /// Size in points.
    pub(crate) size: i32,
    /// Bold style.
    pub(crate) bold: bool,
}

/// Upstream `RendererSettings` as the multipoint pipeline reads it.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Settings {
    /// `getDeviceDPI`: scales most pixel constants by `dpi / 96`.
    pub(crate) dpi: i32,
    /// `getMPLabelFont`: font of multipoint labels.
    pub(crate) label_font: FontSpec,
    /// `getTextOutlineWidth`: 0 when the text background method is none.
    pub(crate) text_outline_width: i32,
    /// `getGroupModifiers`.
    pub(crate) group_modifiers: bool,
    /// `getAutoCollapseModifiers`.
    pub(crate) auto_collapse_modifiers: bool,
    /// `getTwoLabelOnly`.
    pub(crate) two_label_only: bool,
    /// `getPatternScale`: scale of repeated decorations.
    pub(crate) pattern_scale: f64,
    /// `getUseLineInterpolation`: upstream's default is true, and nothing in
    /// the web path overrides it.
    pub(crate) use_line_interpolation: bool,
    /// Where the view shows: not an upstream setting; generators that can
    /// leave out repeats far outside it do.
    pub(crate) visible: Option<super::visible::PixelBox>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            dpi: BASE_DPI,
            label_font: FontSpec {
                family: "PT Sans".to_owned(),
                size: 12,
                bold: true,
            },
            text_outline_width: 0,
            group_modifiers: false,
            auto_collapse_modifiers: true,
            two_label_only: false,
            pattern_scale: 1.0,
            use_line_interpolation: true,
            visible: None,
        }
    }
}

impl Settings {
    /// Upstream's `getDeviceDPI() / 96.0`, used throughout the line code.
    pub(crate) fn dpi_scale_factor(&self) -> f64 {
        f64::from(self.dpi) / f64::from(BASE_DPI)
    }
}
