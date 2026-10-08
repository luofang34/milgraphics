//! The version 16 Trip Wire: the wire from point 1 to point 2 with the trip
//! wire glyph at point 1.

use super::lines_like;
use crate::engine::api::Output;

/// Glyph lengths in units of `UNIT_PX`, along the wire (`u`, toward point
/// 2) and up from it (`v`, to the left of the wire's direction on screen),
/// in the template's proportions.
const STEM_UP: f64 = 21.0;
const STEM_DOWN: f64 = 12.5;
const BAR_UP: f64 = 13.0;
const BAR_BACK: f64 = 10.0;
const BAR_FORWARD: f64 = 11.0;
const WIRE_BACK: f64 = 19.0;
/// Radius of the hook that turns the stem's foot toward point 2.
const HOOK: f64 = 12.5;
/// Pixels per unit: the glyph is about 70 pixels tall.
const UNIT_PX: f64 = 1.5;
const HOOK_STEPS: u32 = 8;

/// Adds the glyph where the wire starts and drops upstream's "t" labels,
/// which the glyph replaces.
pub(super) fn glyph(out: &mut Output) {
    let Some(wire) = out.shapes.first() else {
        return;
    };
    let lines = wire.polylines();
    let Some([p1, p2, ..]) = lines.first().map(Vec::as_slice) else {
        return;
    };
    let Some(u) = super::unit((p2.0 - p1.0, p2.1 - p1.1)) else {
        return;
    };
    let v = (u.1, -u.0);
    let at = |a: f64, b: f64| {
        let (a, b) = (a * UNIT_PX, b * UNIT_PX);
        (p1.0 + u.0 * a + v.0 * b, p1.1 + u.1 * a + v.1 * b)
    };
    let mut stem = vec![at(0.0, STEM_UP), at(0.0, -STEM_DOWN)];
    // A quarter circle from the stem's foot, centred ahead of it.
    for k in 1..=HOOK_STEPS {
        let t = f64::from(k) / f64::from(HOOK_STEPS) * core::f64::consts::FRAC_PI_2;
        stem.push(at(HOOK * (1.0 - t.cos()), -STEM_DOWN - HOOK * t.sin()));
    }
    let glyph = lines_like(
        wire,
        &[
            stem,
            vec![at(-BAR_BACK, BAR_UP), at(BAR_FORWARD, BAR_UP)],
            vec![at(-WIRE_BACK, 0.0), at(0.0, 0.0)],
        ],
    );
    out.shapes.push(glyph);
    out.labels.retain(|l| l.text != "t");
}
