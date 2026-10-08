//! The version 16 Minefield, Dynamic Depiction: H centred above the area and
//! W centred below it, as the template shows them.

use super::centred;
use crate::engine::api::{Input, Output};
use crate::engine::settings::Settings;

/// Gap between the area's bounds and the middle of each label, in lines.
const GAP_LINES: f64 = 1.0;

pub(super) fn above_and_below(input: &Input<'_>, out: &mut Output) {
    let (mut left, mut right) = (f64::MAX, f64::MIN);
    let (mut top, mut bottom) = (f64::MAX, f64::MIN);
    for p in &input.pixels {
        left = left.min(p.x);
        right = right.max(p.x);
        top = top.min(p.y);
        bottom = bottom.max(p.y);
    }
    if left > right {
        return;
    }
    let gap = GAP_LINES * f64::from(Settings::default().label_font.size);
    let x = (left + right) / 2.0;
    let texts = [
        (&input.modifiers.additional_info, top - gap),
        (&input.modifiers.dtg_start, bottom + gap),
    ];
    for (text, y) in texts {
        if let Some(text) = text.as_deref().filter(|t| !t.is_empty()) {
            out.labels.push(centred(text, (x, y)));
        }
    }
}
