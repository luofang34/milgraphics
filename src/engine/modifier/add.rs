//! The label adders of Modifier2.java (`AddModifier`, `AddModifier2`,
//! `AddIntegralModifier`, `AddIntegralAreaModifier`, `AddAreaModifier`).
//!
//! Upstream returns silently for empty text and missing points; it throws
//! (and the caller's catch ends the phase) for an index outside the pixels.

use crate::engine::base::{At, EngineError, Pt, idx};
use crate::engine::tg::{ModifierLabel, Tg};

/// `tg.Pixels.get(i)` for a Java `int` index.
pub(super) fn px(tg: &Tg, i: i32) -> Result<Pt, EngineError> {
    tg.pixels.at(idx(i, tg.pixels.len())?)
}

/// Java `Integer.size()` of the pixels as `i32`.
pub(super) fn count(tg: &Tg) -> i32 {
    i32::try_from(tg.pixels.len()).unwrap_or(i32::MAX)
}

/// Upstream `AddModifier2(tg, text, type, lineFactor, pt0, pt1, isIntegral,
/// modifierType)`.
pub(super) fn add_modifier2(
    tg: &mut Tg,
    text: &str,
    kind: i32,
    line_factor: f64,
    path: (Pt, Pt),
    is_integral: bool,
    text_id: Option<&str>,
) {
    if text.is_empty() {
        return;
    }
    tg.modifiers.push(ModifierLabel {
        text_path: [path.0, path.1],
        text_id: text_id.map(str::to_owned),
        text: text.to_owned(),
        kind,
        line_factor,
        is_integral,
        ..ModifierLabel::default()
    });
}

/// Upstream `AddModifier`: needs at least two pixels.
pub(super) fn add_modifier(tg: &mut Tg, text: &str, kind: i32, line_factor: f64, pt0: Pt, pt1: Pt) {
    if tg.pixels.len() < 2 {
        return;
    }
    add_modifier2(tg, text, kind, line_factor, (pt0, pt1), false, None);
}

/// Upstream `AddIntegralModifier(tg, text, type, lineFactor, startIndex,
/// endIndex, isIntegral)`: the path is two of the pixels.
pub(super) fn integral_modifier(
    tg: &mut Tg,
    text: &str,
    kind: i32,
    line_factor: f64,
    (start, end): (i32, i32),
    is_integral: bool,
) -> Result<(), EngineError> {
    if tg.pixels.is_empty() || end >= count(tg) {
        return Ok(());
    }
    let (p0, p1) = (px(tg, start)?, px(tg, end)?);
    add_modifier2(tg, text, kind, line_factor, (p0, p1), is_integral, None);
    Ok(())
}

/// Upstream `AddIntegralAreaModifier(tg, text, type, lineFactor, pt0, pt1,
/// isIntegral)`.
pub(super) fn area_modifier(
    tg: &mut Tg,
    text: &str,
    kind: i32,
    line_factor: f64,
    path: (Pt, Pt),
    is_integral: bool,
) {
    add_modifier2(tg, text, kind, line_factor, path, is_integral, None);
}

/// As [`area_modifier`] for a second point that may be missing, where
/// upstream adds nothing.
pub(super) fn area_modifier_opt(
    tg: &mut Tg,
    text: &str,
    kind: i32,
    line_factor: f64,
    path: (Pt, Option<Pt>),
    is_integral: bool,
) {
    if let Some(p1) = path.1 {
        add_modifier2(tg, text, kind, line_factor, (path.0, p1), is_integral, None);
    }
}

/// `AddIntegralAreaModifier` with a modifier type recorded in the label.
pub(super) fn area_modifier_id(
    tg: &mut Tg,
    text: &str,
    kind: i32,
    line_factor: f64,
    pt: Pt,
    is_integral: bool,
    text_id: &str,
) {
    add_modifier2(
        tg,
        text,
        kind,
        line_factor,
        (pt, pt),
        is_integral,
        Some(text_id),
    );
}

/// Upstream `AddAreaModifier(tg, text, type, lineFactor, pt0, pt1)`: an
/// integral label.
pub(super) fn add_area_modifier(
    tg: &mut Tg,
    text: &str,
    kind: i32,
    line_factor: f64,
    path: (Pt, Pt),
) {
    add_modifier2(tg, text, kind, line_factor, path, true, None);
}
