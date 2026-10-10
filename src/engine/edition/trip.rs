//! The trip wire glyph: in version 16, a fixed-size glyph where the wire
//! from point 1 to point 2 starts; in base MIL-STD-2525D (code 10), a glyph
//! spanning its three points.

use super::lines_like;
use crate::engine::api::{Input, Output};

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

/// Where base MIL-STD-2525D's template (Table H-XIX) crosses the stem with
/// the wire, as a fraction of the stem from point 1, and how far the wire
/// reaches either side, in stem lengths.
const WIRE_AT: f64 = 0.6;
const WIRE_HALF: f64 = 1.1;

/// Base MIL-STD-2525D: the stem from point 1 down to point 2, ending in a
/// quarter circle toward point 3 whose radius is point 3's distance from
/// the stem; a bar through the stem at point 3's height, ending at point 3;
/// and the wire across the stem. Replaces upstream's line and its "t"s.
pub(super) fn glyph_on_points(input: &Input<'_>, out: &mut Output) {
    let (Some(like), [p1, p2, p3, ..]) = (out.shapes.first(), input.pixels.as_slice()) else {
        return;
    };
    let (dx, dy) = (p2.x - p1.x, p2.y - p1.y);
    let length = dx.hypot(dy);
    let Some(u) = super::unit((dx, dy)) else {
        return;
    };
    let across = (p3.x - p1.x) * -u.1 + (p3.y - p1.y) * u.0;
    let n = if across < 0.0 {
        (u.1, -u.0)
    } else {
        (-u.1, u.0)
    };
    let r = across.abs();
    let at = |along: f64, side: f64| {
        (
            p1.x + u.0 * along + n.0 * side,
            p1.y + u.1 * along + n.1 * side,
        )
    };
    let mut stem = vec![at(0.0, 0.0), at(length, 0.0)];
    for k in 1..=HOOK_STEPS {
        let t = f64::from(k) / f64::from(HOOK_STEPS) * core::f64::consts::FRAC_PI_2;
        stem.push(at(length + r * t.sin(), r * (1.0 - t.cos())));
    }
    let bar_at = (p3.x - p1.x) * u.0 + (p3.y - p1.y) * u.1;
    let wire_at = WIRE_AT * length;
    let glyph = lines_like(
        like,
        &[
            stem,
            vec![at(bar_at, -r), at(bar_at, r)],
            vec![
                at(wire_at, -WIRE_HALF * length),
                at(wire_at, WIRE_HALF * length),
            ],
        ],
    );
    out.shapes = vec![glyph];
    out.labels.retain(|l| l.text != "t");
}
