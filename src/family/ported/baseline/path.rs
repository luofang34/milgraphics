//! The paths the renderer draws between consecutive control points, and an
//! index for finding which of them a stroked segment lies on.

use std::collections::BTreeMap;

use crate::plane::Xy;

/// How the renderer joins two consecutive control points.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Edge {
    /// A straight segment.
    Straight,
    /// The smooth curve of METOC and hydrographic lines: the four
    /// quadratics upstream samples in place of the cubic through the
    /// control points (`DrawSplines`, `drawCubicBezier2`). Its path keeps
    /// only the samples, each cut to whole pixels, so the drawn line lies
    /// within a pixel and a half of the curve.
    Spline,
}

impl Edge {
    /// Distance, in pixels, within which a stroked segment lies on the
    /// edge.
    pub(crate) fn tolerance_px(self) -> f64 {
        match self {
            Self::Straight => 0.5,
            Self::Spline => 1.5,
        }
    }
}

/// Most chords each quadratic of a spline is sampled into.
const MAX_CHORDS: f64 = 64.0;
/// Share of the neighbouring chord upstream's splines put their control
/// points at (tension 0.33, `0.33 / 0.5 * 0.175`).
const CONTROL_SCALE: f64 = 0.33 / 0.5 * 0.175;

/// Path of the edge from control point `i` to the next, through `points`,
/// with chords within `flatness` of a curve; a spline has no closing edge.
pub(crate) fn path(points: &[Xy], i: usize, edge: Edge, flatness: f64) -> Option<Vec<Xy>> {
    let n = points.len();
    let p = *points.get(i)?;
    let q = *points.get((i + 1) % n)?;
    match edge {
        Edge::Straight => Some(vec![p, q]),
        Edge::Spline => {
            if i + 1 >= n {
                return None;
            }
            let before = if i == 0 { p } else { *points.get(i - 1)? };
            let after2 = *points.get(i + 2).unwrap_or(&q);
            let c1 = p.add(q.sub(before).scale(CONTROL_SCALE));
            let c2 = q.sub(after2.sub(p).scale(CONTROL_SCALE));
            let mut out = vec![p];
            for (a, b, c) in quadratics(p, c1, c2, q) {
                // A quadratic strays from its chord by half its control
                // point's offset, and k chords cut that by k².
                let bulge = b.sub(a.add(c).scale(0.5)).len() / 2.0;
                let k = (bulge / flatness).sqrt().ceil().clamp(1.0, MAX_CHORDS);
                let steps = k as u32;
                out.extend((1..=steps).map(|j| quadratic(a, b, c, f64::from(j) / k)));
            }
            Some(out)
        }
    }
}

/// The quadratics `drawCubicBezier2` samples in place of the cubic
/// `p0..p3`, each as start, control and end.
fn quadratics(p0: Xy, p1: Xy, p2: Xy, p3: Xy) -> [(Xy, Xy, Xy); 4] {
    let on = |a: Xy, b: Xy, r: f64| a.add(b.sub(a).scale(r));
    let (pa, pb) = (on(p0, p1, 0.75), on(p3, p2, 0.75));
    let d = p3.sub(p0).scale(1.0 / 16.0);
    let c1 = on(p0, p1, 0.375);
    let c2 = on(pa, pb, 0.375).sub(d);
    let c3 = on(pb, pa, 0.375).add(d);
    let c4 = on(p3, p2, 0.375);
    let (m1, m2, m3) = (on(c1, c2, 0.5), on(pa, pb, 0.5), on(c3, c4, 0.5));
    [(p0, c1, m1), (m1, c2, m2), (m2, c3, m3), (m3, c4, p3)]
}

fn quadratic(a: Xy, b: Xy, c: Xy, t: f64) -> Xy {
    let s = 1.0 - t;
    a.scale(s * s).add(b.scale(2.0 * s * t)).add(c.scale(t * t))
}

/// Paths, with the cells of a square grid each one passes near.
pub(crate) struct Paths {
    /// Each path's points, cumulative lengths and tolerance.
    paths: Vec<(Vec<Xy>, Vec<f64>, f64)>,
    /// The largest tolerance, by which each path's cells are grown.
    tolerance: f64,
    cell: f64,
    grid: BTreeMap<(i64, i64), Vec<usize>>,
}

/// Most grid cells along each side of the paths' extent, which bounds the
/// cells a path is entered in.
const MAX_CELLS: f64 = 1024.0;

impl Paths {
    /// Index of `paths` for segments lying on them, each within its own
    /// tolerance.
    pub(crate) fn new(paths: Vec<(Vec<Xy>, f64)>) -> Self {
        let tolerance = paths.iter().map(|p| p.1).fold(0.0, f64::max);
        let (mut lo, mut hi) = (Xy::new(f64::MAX, f64::MAX), Xy::new(f64::MIN, f64::MIN));
        let mut total = 0.0;
        for w in paths.iter().flat_map(|p| p.0.windows(2)) {
            if let &[a, b] = w {
                total += b.sub(a).len();
                lo = Xy::new(lo.x.min(a.x).min(b.x), lo.y.min(a.y).min(b.y));
                hi = Xy::new(hi.x.max(a.x).max(b.x), hi.y.max(a.y).max(b.y));
            }
        }
        let span = (hi.x - lo.x).max(hi.y - lo.y);
        let per_path = total / paths.len().max(1) as f64;
        let cell = (per_path / 2.0).max(span / MAX_CELLS).max(4.0 * tolerance);
        let mut index = Self {
            paths: Vec::with_capacity(paths.len()),
            tolerance,
            cell: if cell.is_finite() && cell > 0.0 {
                cell
            } else {
                1.0
            },
            grid: BTreeMap::new(),
        };
        for (k, (points, own)) in paths.into_iter().enumerate() {
            for w in points.windows(2) {
                if let &[a, b] = w {
                    let (c0, c1) = index.cells(a, b);
                    for x in c0.0..=c1.0 {
                        for y in c0.1..=c1.1 {
                            let at = index.grid.entry((x, y)).or_default();
                            if at.last() != Some(&k) {
                                at.push(k);
                            }
                        }
                    }
                }
            }
            let mut cum = vec![0.0];
            for w in points.windows(2) {
                if let (&[a, b], Some(&last)) = (w, cum.last()) {
                    cum.push(last + b.sub(a).len());
                }
            }
            index.paths.push((points, cum, own));
        }
        index
    }

    /// Length of path `k`.
    pub(crate) fn length(&self, k: usize) -> f64 {
        self.paths
            .get(k)
            .and_then(|(_, cum, _)| cum.last().copied())
            .unwrap_or(0.0)
    }

    /// Each path the segment `u`–`v` lies on, with the span along it the
    /// segment covers.
    pub(crate) fn spans(&self, u: Xy, v: Xy) -> Vec<(usize, f64, f64)> {
        self.meets(u, v)
            .into_iter()
            .filter_map(|(k, lo, hi, on)| on.then_some((k, lo, hi)))
            .collect()
    }

    /// Each path the segment `u`–`v` starts and ends on, with the span
    /// between its ends and whether its middle lies on the path too: a
    /// chord cutting across a bend does not.
    pub(crate) fn meets(&self, u: Xy, v: Xy) -> Vec<(usize, f64, f64, bool)> {
        let (c0, c1) = self.cells(u, u);
        let mut near: Vec<usize> = (c0.0..=c1.0)
            .flat_map(|x| (c0.1..=c1.1).map(move |y| (x, y)))
            .filter_map(|c| self.grid.get(&c))
            .flatten()
            .copied()
            .collect();
        near.sort_unstable();
        near.dedup();
        let mid = u.add(v).scale(0.5);
        near.into_iter()
            .filter_map(|k| {
                let (points, cum, tolerance) = self.paths.get(k)?;
                let su = locate(points, cum, u, *tolerance)?;
                let sv = locate(points, cum, v, *tolerance)?;
                let on = locate(points, cum, mid, *tolerance).is_some();
                let (lo, hi) = (su.min(sv), su.max(sv));
                // A chord between two far parts of a curved path is not on it.
                (hi > lo && hi - lo <= v.sub(u).len() + 2.0 * tolerance).then_some((k, lo, hi, on))
            })
            .collect()
    }

    /// Grid cells of the box around `a` and `b`, grown by the tolerance.
    fn cells(&self, a: Xy, b: Xy) -> ((i64, i64), (i64, i64)) {
        let cell = |v: f64| (v / self.cell).floor() as i64;
        let t = self.tolerance;
        (
            (cell(a.x.min(b.x) - t), cell(a.y.min(b.y) - t)),
            (cell(a.x.max(b.x) + t), cell(a.y.max(b.y) + t)),
        )
    }
}

/// Distance along `points` (with cumulative lengths `cum`) of the point
/// nearest `p`, when it is within `tolerance`.
fn locate(points: &[Xy], cum: &[f64], p: Xy, tolerance: f64) -> Option<f64> {
    let mut best: Option<(f64, f64)> = None;
    for (w, &start) in points.windows(2).zip(cum) {
        let &[a, b] = w else { continue };
        let d = b.sub(a);
        let len2 = d.dot(d);
        let t = if len2 > 0.0 {
            (p.sub(a).dot(d) / len2).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let off = p.sub(a.add(d.scale(t))).len();
        if best.is_none_or(|(o, _)| off < o) {
            best = Some((off, start + t * len2.sqrt()));
        }
    }
    best.filter(|&(off, _)| off <= tolerance).map(|(_, s)| s)
}
