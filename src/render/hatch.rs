//! Hatch lines: parallel strokes clipped to an area on screen.

use crate::construction::PartRole;
use crate::render::{ScreenItem, ScreenPoint, ScreenShape};
use crate::style::{DashPattern, Fill, Hatch, Stroke};

/// Most lines one hatch may draw; a larger area keeps its outline and the
/// lines that fit, so a graphic zoomed far in stays cheap.
const MAX_LINES: usize = 2000;

/// The segments of `hatch` inside `ring` (closed, first point not repeated),
/// by the even-odd rule. Lines are placed from the ring's first vertex, so the
/// pattern moves with the graphic.
pub(crate) fn segments(ring: &[ScreenPoint], hatch: &Hatch) -> Vec<[ScreenPoint; 2]> {
    let (Some(&origin), true) = (ring.first(), ring.len() >= 3) else {
        return Vec::new();
    };
    if !(hatch.spacing_px.is_finite() && hatch.spacing_px > 0.5) {
        return Vec::new();
    }
    // Along the lines, and across them; y grows downward on screen.
    let a = hatch.angle_deg.to_radians();
    let along = (a.cos(), -a.sin());
    let across = (a.sin(), a.cos());
    let frame = |p: ScreenPoint| {
        let (x, y) = (p.x - origin.x, p.y - origin.y);
        (x * along.0 + y * along.1, x * across.0 + y * across.1)
    };
    let local: Vec<(f64, f64)> = ring.iter().map(|&p| frame(p)).collect();
    let (lo, hi) = local
        .iter()
        .fold((f64::MAX, f64::MIN), |(lo, hi), &(_, v)| {
            (lo.min(v), hi.max(v))
        });
    let first = (lo / hatch.spacing_px).ceil() as i64;
    let last = (hi / hatch.spacing_px).floor() as i64;
    let mut out = Vec::new();
    for k in first..=last.min(first.saturating_add(MAX_LINES as i64)) {
        let v = k as f64 * hatch.spacing_px;
        let mut cuts: Vec<f64> = Vec::new();
        for (i, &(u0, v0)) in local.iter().enumerate() {
            let (u1, v1) = local
                .get(i + 1)
                .or(local.first())
                .copied()
                .unwrap_or((u0, v0));
            // Half-open in v, so a vertex on the line is counted once.
            if (v0 <= v) != (v1 <= v) {
                cuts.push(u0 + (v - v0) / (v1 - v0) * (u1 - u0));
            }
        }
        cuts.sort_by(f64::total_cmp);
        for pair in cuts.chunks_exact(2) {
            let &[u0, u1] = pair else {
                continue;
            };
            // A line touching a vertex enters and leaves at once.
            if u1 - u0 > 1e-6 {
                let point = |u: f64| ScreenPoint {
                    x: origin.x + u * along.0 + v * across.0,
                    y: origin.y + u * along.1 + v * across.1,
                };
                out.push([point(u0), point(u1)]);
            }
        }
    }
    out
}

/// The hatch lines of every hatched area among `items`, as decorations,
/// leaving the label `boxes` clear so their text stays legible.
pub(crate) fn items<'a>(
    items: impl Iterator<Item = &'a ScreenItem>,
    boxes: &[[ScreenPoint; 4]],
) -> Vec<ScreenItem> {
    let mut out = Vec::new();
    for item in items {
        let (Fill::Hatch(h), ScreenShape::Polygon(ring)) = (item.fill, &item.shape) else {
            continue;
        };
        let stroke = Stroke {
            color: h.color,
            width_px: h.width_px,
            dash: DashPattern::Solid,
        };
        let lines = segments(ring, &h)
            .into_iter()
            .flat_map(|line| outside(line, boxes));
        out.extend(lines.map(|[a, b]| ScreenItem {
            pick: item.pick.clone(),
            role: PartRole::Hatch,
            shape: ScreenShape::Polyline(vec![a, b]),
            stroke: Some(stroke),
            fill: Fill::None,
            decoration: true,
        }));
    }
    out
}

/// The parts of `line` outside every convex quadrilateral in `boxes`.
pub(super) fn outside(line: [ScreenPoint; 2], boxes: &[[ScreenPoint; 4]]) -> Vec<[ScreenPoint; 2]> {
    let [a, b] = line;
    // Parameter intervals of the line hidden by a box, merged in order.
    let mut hidden: Vec<(f64, f64)> = boxes.iter().filter_map(|q| inside(a, b, q)).collect();
    hidden.sort_by(|x, y| x.0.total_cmp(&y.0));
    let at = |t: f64| ScreenPoint {
        x: a.x + (b.x - a.x) * t,
        y: a.y + (b.y - a.y) * t,
    };
    let mut out = Vec::new();
    let mut from = 0.0_f64;
    for (t0, t1) in hidden {
        if t0 > from + 1e-9 {
            out.push([at(from), at(t0)]);
        }
        from = from.max(t1);
    }
    if from < 1.0 - 1e-9 {
        out.push([at(from), at(1.0)]);
    }
    out
}

/// The interval of `a`–`b` (as parameters in 0..=1) inside the convex `quad`
/// (Cyrus–Beck), if any.
fn inside(a: ScreenPoint, b: ScreenPoint, quad: &[ScreenPoint; 4]) -> Option<(f64, f64)> {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    // Orientation of the quad, so each edge's inward side is known.
    let edges = || quad.iter().zip(quad.iter().cycle().skip(1));
    let area: f64 = edges().map(|(p, q)| p.x * q.y - q.x * p.y).sum();
    let (mut lo, mut hi) = (0.0_f64, 1.0_f64);
    for (p, q) in edges() {
        // Inward normal of edge p→q.
        let (nx, ny) = if area > 0.0 {
            (-(q.y - p.y), q.x - p.x)
        } else {
            (q.y - p.y, -(q.x - p.x))
        };
        let num = nx * (a.x - p.x) + ny * (a.y - p.y);
        let den = nx * dx + ny * dy;
        if den.abs() < 1e-12 {
            if num < 0.0 {
                return None;
            }
        } else {
            let t = -num / den;
            if den > 0.0 {
                lo = lo.max(t);
            } else {
                hi = hi.min(t);
            }
        }
    }
    (lo < hi).then_some((lo, hi))
}

#[cfg(test)]
mod tests;
