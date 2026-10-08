//! Frontal Attack's bar across the arrow tip.

use crate::engine::api::Output;

/// The template centres the bar on the tip with twice the length of the
/// arrowhead's base; upstream draws it about as long as the base. The bar
/// keeps its centre and direction, so it stays in proportion to the arrow.
///
/// Upstream's lines are the two sides, the two halves of the arrowhead
/// (each from a wing to the tip) and the bar, then the "A".
pub(super) fn widen_bar(out: &mut Output) {
    let Some(shape) = out.shapes.first_mut() else {
        return;
    };
    let mut lines = shape.polylines();
    let (Some(upper), Some(lower)) = (lines.get(2), lines.get(3)) else {
        return;
    };
    let (Some(&tip), Some(&w1), Some(&w2)) = (upper.first(), upper.get(1), lower.get(1)) else {
        return;
    };
    let base = (w1.0 - w2.0).hypot(w1.1 - w2.1);
    let Some(bar) = lines.get_mut(4) else {
        return;
    };
    let [a, b] = bar.as_slice() else {
        return;
    };
    let mid = ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
    // Only the bar upstream centres on the tip.
    if (mid.0 - tip.0).hypot(mid.1 - tip.1) > base / 4.0 {
        return;
    }
    let Some(u) = super::unit((b.0 - a.0, b.1 - a.1)) else {
        return;
    };
    *bar = vec![
        (mid.0 - u.0 * base, mid.1 - u.1 * base),
        (mid.0 + u.0 * base, mid.1 + u.1 * base),
    ];
    super::set_lines(shape, &lines);
}
