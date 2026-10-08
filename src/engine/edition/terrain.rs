//! The version 16 terrain areas: Restricted Terrain, hatched, and Severely
//! Restricted Terrain, cross hatched, each holding H (the cause of the
//! restriction); and Human Terrain, holding H under "HT". The Sector 1 and
//! Sector 2 fields of the terrain areas are not drawn.

use super::{bounds_centre, centred, font_px, text};
use crate::engine::api::{Input, Output};
use crate::engine::base::shape_type;
use crate::engine::render_utility::hatch::HATCH_FORWARD_DIAGONAL;

/// Replaces the labels of the hatched area with H at its centre and, for a
/// cross hatch, lays a second hatch across the first.
pub(super) fn restricted(input: &Input<'_>, out: &mut Output, cross: bool) {
    out.labels.clear();
    if cross {
        let hatched = out.shapes.iter().find(|s| s.pattern_fill.is_some());
        if let Some(mut second) = hatched.cloned() {
            second.shape_type = shape_type::FILL;
            second.fill_color = None;
            if let Some(hatch) = second.pattern_fill.as_mut() {
                hatch.style = HATCH_FORWARD_DIAGONAL;
            }
            out.shapes.push(second);
        }
    }
    let (Some(h), Some(at)) = (
        text(&input.modifiers.additional_info),
        bounds_centre(&input.pixels),
    ) else {
        return;
    };
    out.labels.push(centred(h, at));
}

/// Adds H one line under the "HT" label.
pub(super) fn human(input: &Input<'_>, out: &mut Output) {
    let Some(h) = text(&input.modifiers.additional_info) else {
        return;
    };
    let Some(label) = out.labels.iter().find(|l| l.text == "HT") else {
        return;
    };
    let mut below = label.clone();
    below.text = h.to_owned();
    below.y += font_px();
    out.labels.push(below);
}
