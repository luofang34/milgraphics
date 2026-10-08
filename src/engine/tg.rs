//! Port of mil-sym-java JavaTacticalRenderer/TGLight.java and the `Modifier2`
//! record: the mutable bag the pipeline reads and rewrites while it turns a
//! line type and pixel points into shapes and labels.
//!
//! Upstream's `visibleModifiers` flag is always true, so its guarded getters
//! reduce to plain fields here (including `get_H` and `get_H2`, whose
//! RECTANGULAR exception only matters when modifiers are hidden). Geographic points (`LatLongs`), the texture
//! paint and the mask flag are not carried: the pixel-space engine never
//! reads them.

use crate::engine::base::{EngineError, Pt};
use crate::engine::settings::{FontSpec, Settings};
use crate::style::Rgba;

#[cfg(test)]
mod tests;

/// `BasicStroke.CAP_BUTT`.
pub(crate) const CAP_BUTT: i32 = 0;
/// `BasicStroke.CAP_ROUND`.
pub(crate) const CAP_ROUND: i32 = 1;
/// `BasicStroke.CAP_SQUARE`.
pub(crate) const CAP_SQUARE: i32 = 2;

/// Upstream `Modifier2`'s data: one label with its anchor path and
/// placement rule. The placement logic lives with the label code.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ModifierLabel {
    /// Anchor points: both for labels along a segment, the first alone for
    /// labels at a point.
    pub(crate) text_path: [Pt; 2],
    /// The modifier this label came from ("T", "W", "H", ...), if any.
    pub(crate) text_id: Option<String>,
    /// Identifier of the feature the label belongs to.
    pub(crate) feature_id: Option<String>,
    /// The label text.
    pub(crate) text: String,
    /// Instance count of this modifier.
    pub(crate) iteration: i32,
    /// Text justification as `ShapeInfo` defines it.
    pub(crate) justify: i32,
    /// Placement type (upstream's `toEnd`, `aboveMiddle`, `area`, ...).
    pub(crate) kind: i32,
    /// Multiple of the line height to offset the label from its anchor.
    pub(crate) line_factor: f64,
    /// True for text that is part of the symbol rather than a modifier.
    pub(crate) is_integral: bool,
    /// False once the label is known not to fit its area.
    pub(crate) fits_mbr: bool,
}

impl Default for ModifierLabel {
    fn default() -> Self {
        Self {
            text_path: [Pt::default(); 2],
            text_id: None,
            feature_id: None,
            text: String::new(),
            iteration: 0,
            justify: 0,
            kind: 0,
            line_factor: 0.0,
            is_integral: false,
            fits_mbr: true,
        }
    }
}

/// Upstream `TGLight`: a tactical graphic as the pipeline sees it.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Tg {
    /// Control points, then (after construction) the drawn points.
    pub(crate) pixels: Vec<Pt>,
    /// Labels placed so far.
    pub(crate) modifiers: Vec<ModifierLabel>,
    /// Label font.
    pub(crate) font: FontSpec,
    /// Size in pixels of the symbol drawn inside some areas.
    pub(crate) icon_size: i32,
    /// Keep the unit symbol's aspect ratio.
    pub(crate) keep_unit_ratio: bool,
    /// One of the `tactical_lines` constants.
    pub(crate) line_type: i32,
    /// 0 solid, 1 dashed, as `getLineStroke` reads it.
    pub(crate) line_style: i32,
    /// Line colour.
    pub(crate) line_color: Option<Rgba>,
    /// Fill pattern style.
    pub(crate) fill_style: i32,
    /// Fill colour.
    pub(crate) fill_color: Option<Rgba>,
    /// Label background colour.
    pub(crate) font_back_color: Option<Rgba>,
    /// Label text colour.
    pub(crate) text_color: Option<Rgba>,
    /// Line width in pixels.
    pub(crate) line_thickness: i32,
    /// Modifier T (name).
    pub(crate) t: String,
    /// Modifier T1.
    pub(crate) t1: String,
    /// Modifier T2.
    pub(crate) t2: String,
    /// Modifier AM (distance, metres).
    pub(crate) am: String,
    /// Additional AM values.
    pub(crate) am1: String,
    /// Modifier AN (azimuth).
    pub(crate) an: String,
    /// Modifier V.
    pub(crate) v: String,
    /// Modifier AP.
    pub(crate) ap: String,
    /// Modifier AS (country code text).
    pub(crate) as_: String,
    /// Modifier X (altitude/depth).
    pub(crate) x: String,
    /// Modifier X1.
    pub(crate) x1: String,
    /// Modifier H.
    pub(crate) h: String,
    /// Location text; takes precedence over `h` in `location()`.
    pub(crate) y: String,
    /// Modifier H1.
    pub(crate) h1: String,
    /// Modifier N, "ENY" unless set.
    pub(crate) n: String,
    /// Modifier H2.
    pub(crate) h2: String,
    /// Range fan only: left azimuth, right azimuth, min radius, max radius.
    pub(crate) lrmm: String,
    /// Modifier W (date-time group).
    pub(crate) w: String,
    /// Modifier W1.
    pub(crate) w1: String,
    /// Two digits of the standard identity, "00" before a symbol id is set.
    pub(crate) standard_identity: String,
    /// Echelon text for control measures ("II", "XX", ...), possibly empty.
    pub(crate) echelon_symbol: String,
    /// The symbol code.
    pub(crate) symbol_id: String,
    /// "P" present or "A" anticipated.
    pub(crate) status: String,
    /// Interpolate extra points along lines.
    pub(crate) use_line_interpolation: bool,
    /// Dash with an explicit dash array.
    pub(crate) use_dash_array: bool,
    /// Hatch the area fill.
    pub(crate) use_hatch_fill: bool,
    /// Set when the points were clipped.
    pub(crate) was_clipped: bool,
    /// Leave out the optional range and azimuth labels of range fans.
    pub(crate) hide_optional_labels: bool,
    /// One of the `CAP_*` constants.
    pub(crate) line_cap: i32,
    /// Scale of repeated decorations.
    pub(crate) pattern_scale: f64,
}

impl Tg {
    /// A graphic with upstream's field defaults and the settings' font,
    /// pattern scale and interpolation flag.
    pub(crate) fn new(settings: &Settings) -> Self {
        Self {
            pixels: Vec::new(),
            modifiers: Vec::new(),
            font: settings.label_font.clone(),
            icon_size: 50,
            keep_unit_ratio: true,
            line_type: 0,
            line_style: 0,
            line_color: None,
            fill_style: 0,
            fill_color: None,
            font_back_color: Some(Rgba::opaque(255, 255, 255)),
            text_color: None,
            line_thickness: 0,
            t: String::new(),
            t1: String::new(),
            t2: String::new(),
            am: String::new(),
            am1: String::new(),
            an: String::new(),
            v: String::new(),
            ap: String::new(),
            as_: String::new(),
            x: String::new(),
            x1: String::new(),
            h: String::new(),
            y: String::new(),
            h1: String::new(),
            n: "ENY".to_owned(),
            h2: String::new(),
            lrmm: String::new(),
            w: String::new(),
            w1: String::new(),
            standard_identity: "00".to_owned(),
            echelon_symbol: String::new(),
            symbol_id: "00000000".to_owned(),
            status: "P".to_owned(),
            use_line_interpolation: settings.use_line_interpolation,
            use_dash_array: false,
            use_hatch_fill: false,
            was_clipped: false,
            hide_optional_labels: false,
            line_cap: CAP_SQUARE,
            pattern_scale: settings.pattern_scale,
        }
    }

    /// Upstream `get_Location`: the location text, `y` if set, else `h`.
    pub(crate) fn location(&self) -> &str {
        if self.y.is_empty() { &self.h } else { &self.y }
    }

    /// Upstream `isHostile`: standard identity suspect/joker or
    /// hostile/faker.
    pub(crate) fn is_hostile(&self) -> bool {
        matches!(self.standard_identity.chars().nth(1), Some('5' | '6'))
    }

    /// Upstream `set_SymbolId`. Stores the code, and for control measures
    /// (symbol set 25) derives the standard identity, status (planned sets a
    /// dashed line), echelon text and, when `as_` is empty, the country
    /// text. `country_text` maps a numeric country code to its three-letter
    /// code. A malformed code leaves the later fields unset, as upstream's
    /// exception does, and is reported.
    pub(crate) fn set_symbol_id(
        &mut self,
        symbol_id: &str,
        country_text: &dyn Fn(i32) -> Option<String>,
    ) -> Result<(), EngineError> {
        symbol_id.clone_into(&mut self.symbol_id);
        let long_enough = symbol_id.len() >= 20;
        let symbol_set = if long_enough {
            digits(symbol_id, 4, 6)?
        } else {
            0
        };
        if symbol_set != 25 {
            return Ok(());
        }
        let identity = digits(symbol_id, 2, 4)?;
        self.standard_identity = format!("{identity:02}");
        self.status = "P".to_owned();
        if digits(symbol_id, 6, 7)? == 1 {
            self.status = "A".to_owned();
            self.line_style = 1;
        }
        self.echelon_symbol = echelon_text(digits(symbol_id, 8, 10)?)
            .unwrap_or_default()
            .to_owned();
        let country = if symbol_id.len() == 30 {
            digits(symbol_id, 27, 30)?
        } else {
            0
        };
        if country > 0 && self.as_.is_empty() {
            self.as_ = country_text(country).unwrap_or_default();
        }
        Ok(())
    }
}

/// Java `Integer.parseInt(symbolID.substring(from, to))`.
fn digits(text: &str, from: usize, to: usize) -> Result<i32, EngineError> {
    let part = text
        .get(from..to)
        .ok_or(EngineError::Degenerate("symbol code too short"))?;
    part.parse::<i32>()
        .map_err(|_| EngineError::Number(part.to_owned()))
}

/// Upstream `SymbolUtilities.getEchelonText`.
fn echelon_text(amplifier: i32) -> Option<&'static str> {
    Some(match amplifier {
        11 => "\u{d8}",
        12 => "\u{2022}",
        13 => "\u{2022}\u{2022}",
        14 => "\u{2022}\u{2022}\u{2022}",
        15 => "I",
        16 => "II",
        17 => "III",
        18 => "X",
        21 => "XX",
        22 => "XXX",
        23 => "XXXX",
        24 => "XXXXX",
        25 => "XXXXXX",
        26 => "++",
        _ => return None,
    })
}
