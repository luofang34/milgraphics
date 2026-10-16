//! Leaving out of the screen tier the stroked segments that lie on edges
//! the construction drew as geographic lines (`family::ported::baseline`).

use crate::family::ported_baseline::{ON_EDGE_PX, on_edge};
use crate::plane::Xy;
use crate::render::ScreenPoint;

/// Start, unit direction and length, in frame pixels, of the edges listed in
/// `baseline` between the control points at `pixels`.
pub(super) fn edges(pixels: &[ScreenPoint], baseline: &[usize]) -> Vec<(Xy, Xy, f64)> {
    let n = pixels.len();
    baseline
        .iter()
        .filter_map(|&i| {
            let (p, q) = (pixels.get(i)?, pixels.get((i + 1) % n.max(1))?);
            let (p, q) = (Xy::new(p.x, p.y), Xy::new(q.x, q.y));
            Some((p, q.sub(p).unit()?, q.sub(p).len()))
        })
        .collect()
}

#[cfg(test)]
mod tests;

/// `lines` split where their segments lie on an edge, without those
/// segments. Cut ends from clipping lie on the same edges, so a stretch of
/// the line drawn for a close view strips like the whole line.
pub(super) fn strip(lines: Vec<Vec<(f64, f64)>>, edges: &[(Xy, Xy, f64)]) -> Vec<Vec<(f64, f64)>> {
    if edges.is_empty() {
        return lines;
    }
    let on_baseline = |u: (f64, f64), v: (f64, f64)| {
        edges.iter().any(|&(p, along, len)| {
            let (u, v) = (Xy::new(u.0, u.1).sub(p), Xy::new(v.0, v.1).sub(p));
            on_edge(u, v, along, len, ON_EDGE_PX).is_some()
        })
    };
    let mut out = Vec::new();
    for line in lines {
        let mut piece: Vec<(f64, f64)> = Vec::new();
        for w in line.windows(2) {
            let &[u, v] = w else { continue };
            if on_baseline(u, v) {
                if piece.len() > 1 {
                    out.push(std::mem::take(&mut piece));
                }
                piece.clear();
            } else {
                if piece.is_empty() {
                    piece.push(u);
                }
                piece.push(v);
            }
        }
        if piece.len() > 1 {
            out.push(piece);
        }
    }
    out
}
