//! Port of mil-sym-java JavaTacticalRenderer/Modifier2.java: the label logic.
//!
//! Three phases mirror upstream. [`add_modifiers_geo`] places labels from the
//! control points before the geometry exists, [`add_modifiers2`] places those
//! that depend on the drawn points, and [`display_modifiers2`] turns the
//! recorded [`ModifierLabel`](crate::engine::tg::ModifierLabel)s into text
//! with a pixel position, angle and justification. [`scale_modifiers`]
//! collapses labels that do not fit small areas.
//!
//! Not ported: image modifiers (`getImageModifier`, `areaImage`, the
//! decision-point, anchor and mine symbols of `DECISION_LINE`, `MINED`,
//! `FENCED`, `LAA`, the contamination areas, `DEPICT`, anchorages and mine
//! lines), which the application renders as single-point symbols;
//! `createTextOutline`/`getTextShape` (text outlines through `TextLayout`);
//! the clip-bounds branch of `getVisibleMiddleSegment` (the engine does not
//! clip); `RemoveModifiers`, which upstream runs for the CPOF clients only;
//! the deprecated `DisplayModifiers`, `AddNameAboveDTG` and the unused
//! `AddOffsetModifier`.
//!
//! Text width is the injected `text_width`, which stands for AWT's
//! `FontMetrics.stringWidth`. Width-based placement decisions use it as is
//! (truncated to whole pixels where upstream stores it in an `int`);
//! `display_modifiers2` adds 1 to it as upstream does.
//!
//! Upstream records the same `POINT2` objects in `tg.Pixels` and in a
//! label's text path. Here labels hold copies taken when the label is added,
//! which is equivalent because upstream restores `Pixels` from a deep copy
//! before anything else can move those points.

mod add;
mod boundary;
pub(crate) mod center_label;
pub(crate) mod display;
pub(crate) mod geo;
mod geo_areas;
mod geo_lines;
mod geo_routes;
mod group_strings;
pub(crate) mod integral_shapes;
pub(crate) mod layout;
pub(crate) mod post;
mod post_areas;
mod post_sector;
mod post_tasks;
pub(crate) mod scale;
mod type_sets;

#[cfg(test)]
mod tests;

/// Placement `toEnd`: next to the first point, on the opposite side of the
/// line.
pub(crate) const TO_END: i32 = 1;
/// Placement `aboveMiddle`: between both points, at the angle between them.
pub(crate) const ABOVE_MIDDLE: i32 = 2;
/// Placement `area`: one point, text always upright.
pub(crate) const AREA: i32 = 3;
/// Placement `screen`: one point, for screen, cover and guard.
pub(crate) const SCREEN: i32 = 4;
/// Placement `aboveEnd`: next to the first point, above the line.
pub(crate) const ABOVE_END: i32 = 5;
/// Placement `aboveMiddlePerpendicular`: between both points, rotated a
/// quarter turn.
pub(crate) const ABOVE_MIDDLE_PERPENDICULAR: i32 = 6;
/// Placement `aboveStartInside`: at the start, inside the shape.
pub(crate) const ABOVE_START_INSIDE: i32 = 7;
/// Placement `aboveEndInside`: at the back of the line, inside the shape.
pub(crate) const ABOVE_END_INSIDE: i32 = 8;
/// Placement `areaImage`: one point, for an image.
pub(crate) const AREA_IMAGE: i32 = 9;

/// `ShapeInfo.justify_left`.
pub(crate) const JUSTIFY_LEFT: i32 = 0;
/// `ShapeInfo.justify_center`.
pub(crate) const JUSTIFY_CENTER: i32 = 1;
/// `ShapeInfo.justify_right`.
pub(crate) const JUSTIFY_RIGHT: i32 = 2;

/// A label as `DisplayModifiers2` records it on its `Shape2`.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PlacedLabel {
    /// The label text.
    pub(crate) text: String,
    /// Upstream's modifier position (where the text is drawn), x in pixels.
    pub(crate) x: f64,
    /// The modifier position's y in pixels.
    pub(crate) y: f64,
    /// Rotation in degrees (`theta * 180 / PI`), clockwise on screen.
    pub(crate) angle_deg: f64,
    /// One of the `JUSTIFY_*` constants.
    pub(crate) justify: i32,
    /// The point the label belongs to (upstream's modifier anchor).
    pub(crate) anchor: (f64, f64),
    /// Offset from the anchor to the text (upstream's anchor offset); a
    /// client moves the text by it.
    pub(crate) anchor_offset: (f64, f64),
    /// The modifier this label came from ("T", "W", ...), if recorded.
    pub(crate) text_id: Option<String>,
}
