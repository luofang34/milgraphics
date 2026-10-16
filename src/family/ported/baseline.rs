//! The edges between control points that the renderer strokes whole.
//!
//! A front or an obstacle line is drawn as one line along its control points
//! with pixel-sized teeth, ticks or glyphs woven into the same shapes, so no
//! shape is the same on the ground at two pixel sizes and the classifier
//! keeps all of it in pixels. The line itself is proportional to the control
//! points: an edge that stroked segments cover end to end at both sizes,
//! straight or along the spline of a smooth line, is a geographic line, and
//! only what lies off it is drawn per view.

use crate::construction::{GeoGeometry, PartId, PartRole};
use crate::engine::base::{Shape, shape_type};
use crate::family::Ctx;
use crate::geo::GeoPoint;
use crate::plane::{LocalPlane, Xy};
use crate::style::{Fill, Rgba, Stroke};

use super::{from_px, shape_fill, shape_stroke};

mod path;
#[cfg(test)]
mod tests;

pub(crate) use path::{Edge, Paths, path};

/// Share of an edge stroked segments must cover. A label break or a gap in
/// a broken line leaves more than this uncovered at the smaller size, where
/// it spans a large part of the edge.
const COVERED: f64 = 0.98;
/// Longest stretch, in pixels at the larger size, a spline may leave
/// uncovered: its line is drawn from samples a fixed number of pixels
/// apart, and a type that stops at its last sample ends short of the curve
/// by a share of the edge the larger size alone cannot rule out.
const SPLINE_GAP_PX: f64 = 4.0;
/// Largest distance, in pixels at the larger size, of a spline's chords from
/// the curve, in the coverage test and on the ground.
const FLATNESS_PX: f64 = 0.1;

/// One run of the renderer: its shapes, which of them are already
/// geographic parts, and its ground metres per pixel.
pub(super) struct Run<'a> {
    shapes: &'a [Shape],
    skip: &'a [bool],
    mpp: f64,
}

impl<'a> Run<'a> {
    pub(super) fn shapes(&self) -> &'a [Shape] {
        self.shapes
    }

    /// Whether shape `i` is already a geographic part.
    pub(super) fn skips(&self, i: usize) -> bool {
        self.skip.get(i).copied().unwrap_or(false)
    }

    pub(super) fn mpp(&self) -> f64 {
        self.mpp
    }

    /// The two classification runs, the shapes of the first already
    /// geographic where `geographic` says so.
    pub(super) fn pair(
        (a, mpp_a): (&'a [Shape], f64),
        (b, mpp_b): (&'a [Shape], f64),
        geographic: &'a [bool],
    ) -> (Self, Self) {
        // Shapes pair by index only when both runs drew the same number.
        let paired = a.len() == b.len();
        (
            Self {
                shapes: a,
                skip: geographic,
                mpp: mpp_a,
            },
            Self {
                shapes: b,
                skip: if paired { geographic } else { &[] },
                mpp: mpp_b,
            },
        )
    }
}

/// Adds the edges both runs stroke whole as geographic lines, consecutive
/// edges with one stroke joined into one part. Each part's id, with the
/// edges it draws.
pub(super) fn add(
    ctx: &mut Ctx<'_>,
    plane: &LocalPlane<'_>,
    anchors: &[GeoPoint],
    xy: &[Xy],
    (a, b): &(Run<'_>, Run<'_>),
) -> Vec<(PartId, Vec<(usize, Edge)>)> {
    let mut runs: Vec<(Vec<(usize, Edge)>, Stroke)> = Vec::new();
    for (i, edge, stroke) in edges(xy, a, b, ctx.palette.line.color) {
        match runs.last_mut() {
            Some((run, s)) if *s == stroke && run.last().map(|e| e.0 + 1) == Some(i) => {
                run.push((i, edge));
            }
            _ => runs.push((vec![(i, edge)], stroke)),
        }
    }
    runs.into_iter()
        .map(|(run, stroke)| {
            let mut points: Vec<GeoPoint> = Vec::new();
            for &(i, edge) in &run {
                points.extend(
                    ground(plane, anchors, xy, (i, edge), FLATNESS_PX * b.mpp)
                        .into_iter()
                        .skip(usize::from(!points.is_empty())),
                );
            }
            let id = ctx.add_part(
                PartRole::Line,
                GeoGeometry::Line(points),
                Some(stroke),
                Fill::None,
            );
            (id, run)
        })
        .collect()
}

/// Points on the ground along edge `i`: its control points, with the
/// spline's samples between them.
fn ground(
    plane: &LocalPlane<'_>,
    anchors: &[GeoPoint],
    xy: &[Xy],
    (i, edge): (usize, Edge),
    flatness: f64,
) -> Vec<GeoPoint> {
    let ends = (
        anchors.get(i).copied(),
        anchors.get((i + 1) % anchors.len().max(1)).copied(),
    );
    let (Some(start), Some(end)) = ends else {
        return Vec::new();
    };
    let inner = match edge {
        Edge::Straight => Vec::new(),
        Edge::Spline => path(xy, i, edge, flatness).unwrap_or_default(),
    };
    let count = inner.len();
    std::iter::once(start)
        .chain(
            inner
                .into_iter()
                .skip(1)
                .take(count.saturating_sub(2))
                .map(|p| plane.to_geo(p)),
        )
        .chain(std::iter::once(end))
        .collect()
}

/// The edges between consecutive control points `xy`, the closing edge
/// included, that both runs stroke whole, by index of their first point,
/// with the path they follow and the stroke of the shape drawing most of
/// each. An edge is taken as straight when the runs stroke it so, else as
/// the spline of smooth lines.
pub(super) fn edges(
    xy: &[Xy],
    a: &Run<'_>,
    b: &Run<'_>,
    fallback: Rgba,
) -> Vec<(usize, Edge, Stroke)> {
    let count = match xy.len() {
        0 | 1 => 0,
        2 => 1,
        n => n,
    };
    let mut found: Vec<(usize, Edge, Stroke)> = Vec::new();
    for kind in [Edge::Straight, Edge::Spline] {
        let paths: Vec<(usize, Vec<Xy>)> = (0..count)
            .filter(|i| found.iter().all(|f| f.0 != *i))
            .filter_map(|i| Some((i, path(xy, i, kind, FLATNESS_PX * b.mpp)?)))
            .collect();
        let (ca, cb) = (coverage(&paths, kind, a), coverage(&paths, kind, b));
        // A run that strokes nothing along any edge has dropped the line at
        // its size (wire is not drawn below a minimum length) and says
        // nothing.
        // The smaller size samples a curve every few percent of its length,
        // cut to whole pixels, so its line does not follow the curve.
        let a_silent = kind == Edge::Spline || ca.iter().all(|c| c.0 <= 0.0);
        for (((i, _), (cov_a, _)), (cov_b, shape)) in paths.iter().zip(&ca).zip(&cb) {
            let Some(shape) = shape.and_then(|s| b.shapes.get(s)) else {
                continue;
            };
            let gap_px = (1.0 - cov_b) * paths_len(&paths, *i, xy, kind) / b.mpp;
            let whole_b = *cov_b >= COVERED && (kind == Edge::Straight || gap_px <= SPLINE_GAP_PX);
            if (a_silent || *cov_a >= COVERED) && whole_b {
                found.push((*i, kind, shape_stroke(shape, fallback)));
            }
        }
    }
    found.sort_by_key(|f| f.0);
    found
}

/// Ground length of edge `i`'s path.
fn paths_len(paths: &[(usize, Vec<Xy>)], i: usize, _xy: &[Xy], _kind: Edge) -> f64 {
    paths
        .iter()
        .find(|p| p.0 == i)
        .map(|(_, p)| {
            p.windows(2)
                .map(|w| match w {
                    &[a, b] => b.sub(a).len(),
                    _ => 0.0,
                })
                .sum()
        })
        .unwrap_or(0.0)
}

/// For each path: the share stroked segments of `run` cover, and the shape
/// covering most of it.
fn coverage(paths: &[(usize, Vec<Xy>)], kind: Edge, run: &Run<'_>) -> Vec<(f64, Option<usize>)> {
    let tolerance = kind.tolerance_px() * run.mpp;
    let index = Paths::new(paths.iter().map(|(_, p)| (p.clone(), tolerance)).collect());
    let mut spans: Vec<Vec<(usize, f64, f64)>> = vec![Vec::new(); paths.len()];
    for (s, shape) in run.shapes.iter().enumerate() {
        if run.skip.get(s).copied().unwrap_or(false) || !stroked(shape) {
            continue;
        }
        for line in shape.polylines() {
            let pts: Vec<Xy> = line.into_iter().map(|v| from_px(v, run.mpp)).collect();
            for w in pts.windows(2) {
                let &[u, v] = w else { continue };
                for (k, lo, hi) in index.spans(u, v) {
                    if let Some(at) = spans.get_mut(k) {
                        at.push((s, lo, hi));
                    }
                }
            }
        }
    }
    spans
        .into_iter()
        .enumerate()
        .map(|(k, on)| {
            let len = index.length(k);
            // The share drawn in one stroke: a line whose style changes
            // partway along an edge (solid, then dashed) is not one line.
            let mut best = (0.0, None);
            let mut shapes: Vec<usize> = on.iter().map(|o| o.0).collect();
            shapes.dedup();
            for &s in &shapes {
                let style = run.shapes.get(s).map(|x| shape_stroke(x, Rgba::BLACK));
                let alike = |o: &&(usize, f64, f64)| {
                    run.shapes.get(o.0).map(|x| shape_stroke(x, Rgba::BLACK)) == style
                };
                let mine = covered(on.iter().filter(alike).map(|o| (o.1, o.2)).collect());
                if mine > best.0 {
                    best = (mine, Some(s));
                }
            }
            (if len > 0.0 { best.0 / len } else { 0.0 }, best.1)
        })
        .collect()
}

/// Whether a shape is a stroked line, not a fill or a filled outline.
pub(crate) fn stroked(shape: &Shape) -> bool {
    shape.shape_type != shape_type::FILL && shape_fill(shape) == Fill::None
}

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
