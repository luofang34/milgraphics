//! The edges between control points that the renderer strokes whole.
//!
//! A front or an obstacle line is drawn as one line along its control points
//! with pixel-sized teeth, ticks or glyphs woven into the same shapes, so no
//! shape is the same on the ground at two pixel sizes and the classifier
//! keeps all of it in pixels. The line itself is proportional to the control
//! points: an edge that stroked segments cover end to end at both sizes is a
//! geographic line, and only what lies off it is drawn per view.

use crate::engine::base::{Shape, shape_type};
use crate::plane::Xy;
use crate::style::{Fill, Rgba, Stroke};

use super::{from_px, shape_fill, shape_stroke};

/// Share of an edge stroked segments must cover. A label break or a gap in
/// a broken line leaves more than this uncovered at the smaller size, where
/// it spans a large part of the edge.
const COVERED: f64 = 0.98;
/// Distance, in pixels, within which a stroked segment lies on an edge.
pub(crate) const ON_EDGE_PX: f64 = 0.5;

/// One run of the renderer: its shapes, which of them are already
/// geographic parts, and its ground metres per pixel.
pub(super) struct Run<'a> {
    pub(super) shapes: &'a [Shape],
    pub(super) skip: &'a [bool],
    pub(super) mpp: f64,
}

/// The edges between consecutive control points `xy`, the closing edge
/// included, that both runs stroke whole, by index of their first point,
/// each with the stroke of the shape drawing most of it.
pub(super) fn edges(xy: &[Xy], a: &Run<'_>, b: &Run<'_>, fallback: Rgba) -> Vec<(usize, Stroke)> {
    let edges = edge_list(xy);
    let (ca, cb) = (coverage(&edges, a), coverage(&edges, b));
    // A run that strokes nothing along any edge has dropped the line at its
    // size (wire is not drawn below a minimum length) and says nothing.
    let a_silent = ca.iter().all(|c| c.0 <= 0.0);
    edges
        .iter()
        .enumerate()
        .filter_map(|(i, _)| {
            let (cov_a, _) = ca.get(i)?;
            let (cov_b, shape) = cb.get(i)?;
            let shape = b.shapes.get((*shape)?)?;
            ((a_silent || *cov_a >= COVERED) && *cov_b >= COVERED)
                .then(|| (i, shape_stroke(shape, fallback)))
        })
        .collect()
}

/// Edges from each control point to the next, closing the ring when there
/// are more than two.
pub(super) fn edge_list(xy: &[Xy]) -> Vec<(Xy, Xy)> {
    let mut edges: Vec<(Xy, Xy)> = xy
        .windows(2)
        .filter_map(|w| match w {
            &[p, q] => Some((p, q)),
            _ => None,
        })
        .collect();
    if let (true, Some(&last), Some(&first)) = (xy.len() > 2, xy.last(), xy.first()) {
        edges.push((last, first));
    }
    edges
}

/// For each edge: the share stroked segments of `run` cover, and the shape
/// covering most of it.
fn coverage(edges: &[(Xy, Xy)], run: &Run<'_>) -> Vec<(f64, Option<usize>)> {
    edges
        .iter()
        .map(|&(p, q)| {
            let Some(along) = q.sub(p).unit() else {
                return (0.0, None);
            };
            let len = q.sub(p).len();
            let tolerance = ON_EDGE_PX * run.mpp;
            let mut spans = Vec::new();
            let mut best = (0.0, None);
            for (i, shape) in run.shapes.iter().enumerate() {
                if run.skip.get(i).copied().unwrap_or(false) || !stroked(shape) {
                    continue;
                }
                let before = spans.len();
                for line in shape.polylines() {
                    let pts: Vec<Xy> = line.into_iter().map(|v| from_px(v, run.mpp)).collect();
                    for w in pts.windows(2) {
                        if let &[u, v] = w {
                            spans.extend(on_edge(u.sub(p), v.sub(p), along, len, tolerance));
                        }
                    }
                }
                let mine = covered(spans.get(before..).unwrap_or_default().to_vec());
                if mine > best.0 {
                    best = (mine, Some(i));
                }
            }
            (covered(spans) / len, best.1)
        })
        .collect()
}

/// Whether a shape is a stroked line, not a fill or a filled outline.
pub(crate) fn stroked(shape: &Shape) -> bool {
    shape.shape_type != shape_type::FILL && shape_fill(shape) == Fill::None
}

/// The span along an edge of direction `along` and length `len`, from its
/// start, that the segment `u`–`v` (relative to that start) covers, when the
/// segment lies on the edge within `tolerance`.
pub(crate) fn on_edge(u: Xy, v: Xy, along: Xy, len: f64, tolerance: f64) -> Option<(f64, f64)> {
    if u.cross(along).abs() > tolerance || v.cross(along).abs() > tolerance {
        return None;
    }
    let (s, t) = (u.dot(along), v.dot(along));
    let (lo, hi) = (s.min(t), s.max(t));
    (lo >= -tolerance && hi <= len + tolerance && hi > lo).then(|| (lo.max(0.0), hi.min(len)))
}

#[cfg(test)]
mod tests;

/// Total length of the union of `spans`.
fn covered(mut spans: Vec<(f64, f64)>) -> f64 {
    spans.sort_by(|x, y| x.0.total_cmp(&y.0));
    let (mut total, mut end) = (0.0, f64::NEG_INFINITY);
    for (lo, hi) in spans {
        let lo = lo.max(end);
        if hi > lo {
            total += hi - lo;
            end = hi;
        }
    }
    total
}
