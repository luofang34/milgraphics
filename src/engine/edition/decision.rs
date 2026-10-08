//! The version 16 Decision Line: a five-pointed star at each end, which
//! holds the end-of-line text (`T/AS`).

use super::centred;
use super::lines_like;
use crate::engine::api::{Input, Output};

/// Smallest distance from a star's centre to its points, in pixels.
const OUTER: f64 = 26.0;
/// Distance from the centre to the notches between points, as a share of
/// the distance to the points, as for a regular star.
const INNER: f64 = 0.381_966;
/// A regular star is about 1.05 times its point distance wide at its
/// centre; the text may reach a little into the side points, as in the
/// template.
const TEXT_TO_OUTER: f64 = 0.95;

/// Adds an upright star beyond each end of the line, placed so the line
/// ends on the star's outline, with the line's text centred in it.
pub(super) fn stars(input: &Input<'_>, out: &mut Output) {
    let text = text(input);
    let Some(line) = out.shapes.first() else {
        return;
    };
    let lines = line.polylines();
    let Some(points) = lines.first() else {
        return;
    };
    let ends = [
        (points.first(), points.get(1)),
        (
            points.last(),
            points.len().checked_sub(2).and_then(|i| points.get(i)),
        ),
    ];
    let width = text.as_deref().map_or(0.0, |t| (input.text_width)(t));
    let outline = star(OUTER.max(TEXT_TO_OUTER * width));
    let mut drawn = Vec::new();
    for (end, inner) in ends {
        let (Some(&e), Some(&i)) = (end, inner) else {
            continue;
        };
        let Some(o) = super::unit((e.0 - i.0, e.1 - i.1)) else {
            continue;
        };
        let Some(reach) = boundary(&outline, (-o.0, -o.1)) else {
            continue;
        };
        let centre = (e.0 + o.0 * reach, e.1 + o.1 * reach);
        if let Some(text) = &text {
            out.labels.push(centred(text, centre));
        }
        drawn.push(
            outline
                .iter()
                .map(|p| (centre.0 + p.0, centre.1 + p.1))
                .collect(),
        );
    }
    if !drawn.is_empty() {
        let shape = lines_like(line, &drawn);
        out.shapes.push(shape);
    }
}

/// "T/AS", or whichever of the two is set.
fn text(input: &Input<'_>) -> Option<String> {
    let set = |v: &Option<String>| v.clone().filter(|t| !t.is_empty());
    match (
        set(&input.modifiers.designation),
        set(&input.modifiers.country),
    ) {
        (Some(t), Some(c)) => Some(format!("{t}/{c}")),
        (t, c) => t.or(c),
    }
}

/// The outline of a star with points `outer` from the origin, point up on
/// screen, closed.
fn star(outer: f64) -> Vec<(f64, f64)> {
    let mut outline: Vec<(f64, f64)> = (0..10)
        .map(|k| {
            let r = if k % 2 == 0 { outer } else { outer * INNER };
            let a = (f64::from(k) * 36.0 - 90.0).to_radians();
            (r * a.cos(), r * a.sin())
        })
        .collect();
    if let Some(&first) = outline.first() {
        outline.push(first);
    }
    outline
}

/// How far from the origin a ray along unit `d` leaves the closed
/// `outline`.
fn boundary(outline: &[(f64, f64)], d: (f64, f64)) -> Option<f64> {
    outline
        .windows(2)
        .filter_map(|w| {
            let [a, b] = w else { return None };
            // Solve t·d = a + s·(b − a) for t > 0 and s in [0, 1].
            let e = (b.0 - a.0, b.1 - a.1);
            let den = d.0 * e.1 - d.1 * e.0;
            if den.abs() < 1e-12 {
                return None;
            }
            let t = (a.0 * e.1 - a.1 * e.0) / den;
            let s = (a.0 * d.1 - a.1 * d.0) / den;
            (t > 0.0 && (0.0..=1.0).contains(&s)).then_some(t)
        })
        .min_by(f64::total_cmp)
}
