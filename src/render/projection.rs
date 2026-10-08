//! What a host supplies to render: a projection and font metrics.

use crate::geo::GeoPoint;

/// A position on screen in pixels, y downward.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScreenPoint {
    /// Pixels from the left edge.
    pub x: f64,
    /// Pixels from the top edge.
    pub y: f64,
}

impl ScreenPoint {
    pub(crate) fn sub(self, o: Self) -> (f64, f64) {
        (self.x - o.x, self.y - o.y)
    }
}

/// The host's map projection and camera.
///
/// Implementations may be flat (Web Mercator), globe or vertical
/// perspective; the library never assumes which.
pub trait Projection {
    /// Screen position of the ground at `point` — on the host's terrain
    /// where it has terrain — or `None` when it is hidden: behind the
    /// horizon, behind the camera or outside the view volume. Graphics are
    /// clamped to the ground; control points with altitudes are refused at
    /// construction until heights are carried through.
    fn project(&self, point: GeoPoint) -> Option<ScreenPoint>;

    /// Geographic position under a screen point, if it hits the surface.
    fn unproject(&self, point: ScreenPoint) -> Option<GeoPoint>;

    /// Largest distance in pixels a drawn chord may stray from the true
    /// curve before it is subdivided.
    fn tolerance_px(&self) -> f64 {
        0.5
    }

    /// Whether any part of the geodesic from `a` to `b` might be visible,
    /// asked when both ends are hidden. The default `true` makes the library
    /// search the segment for a visible span, which costs up to 63
    /// projections; hosts that can cheaply prove a segment hidden (both ends
    /// well behind the globe, say) return `false` to skip that search.
    fn segment_may_be_visible(&self, a: GeoPoint, b: GeoPoint) -> bool {
        let _ = (a, b);
        true
    }
}

/// A label font.
#[derive(Clone, Debug, PartialEq)]
pub struct Font {
    /// Family name, e.g. "PT Sans".
    pub family: String,
    /// Size in pixels.
    pub size_px: f64,
    /// Bold weight.
    pub bold: bool,
}

impl Default for Font {
    fn default() -> Self {
        Self {
            family: "PT Sans".to_owned(),
            size_px: 12.0,
            bold: true,
        }
    }
}

/// Text measurement, supplied by the host so labels match what it draws.
pub trait FontMetrics {
    /// Identifies the metrics (font files and rules), for cache keys.
    fn identity(&self) -> &str;

    /// Advance width of `text` in pixels.
    fn text_width_px(&self, font: &Font, text: &str) -> f64;

    /// Distance between baselines in pixels.
    fn line_height_px(&self, font: &Font) -> f64 {
        font.size_px * 1.2
    }
}

/// Deterministic metrics for tests and headless use: every character
/// advances by the same fraction of the font size.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FixedAdvanceMetrics {
    /// Advance per character, in ems.
    pub advance_em: f64,
}

impl Default for FixedAdvanceMetrics {
    fn default() -> Self {
        Self { advance_em: 0.6 }
    }
}

impl FontMetrics for FixedAdvanceMetrics {
    fn identity(&self) -> &str {
        "fixed-advance"
    }

    fn text_width_px(&self, font: &Font, text: &str) -> f64 {
        text.chars().count() as f64 * self.advance_em * font.size_px
    }
}
