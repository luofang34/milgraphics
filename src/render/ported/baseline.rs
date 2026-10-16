//! Leaving out of the screen tier the stroked segments that lie on edges
//! the construction drew as geographic lines (`family::ported::baseline`).

use crate::family::ported_baseline::{Edge, Paths, path};

/// Largest distance, in pixels, of a spline's chords from the curve.
const FLATNESS_PX: f64 = 0.1;
use crate::plane::Xy;
use crate::render::ScreenPoint;

/// The paths, in frame pixels, of the edges listed in `baseline` between
/// the control points at `pixels`. The frame is a similarity of the plane
/// the edges were found in, so a spline maps to the same spline.
pub(super) fn paths(pixels: &[ScreenPoint], baseline: &[(usize, Edge)]) -> Option<Paths> {
    if baseline.is_empty() {
        return None;
    }
    let points: Vec<Xy> = pixels.iter().map(|p| Xy::new(p.x, p.y)).collect();
    let paths = baseline
        .iter()
        .filter_map(|&(i, edge)| Some((path(&points, i, edge, FLATNESS_PX)?, edge.tolerance_px())))
        .collect();
    Some(Paths::new(paths))
}

/// `lines` split where their segments start and end on a path, without
/// those segments: the renderer draws a curve as chords between samples a
/// fixed number of pixels apart, which cut across its bends where the view
/// shows it small. Cut ends from clipping lie on the same paths, so a
/// stretch of the line drawn for a close view strips like the whole line.
pub(super) fn strip(lines: Vec<Vec<(f64, f64)>>, paths: Option<&Paths>) -> Vec<Vec<(f64, f64)>> {
    let Some(paths) = paths else {
        return lines;
    };
    let on_baseline = |u: (f64, f64), v: (f64, f64)| {
        !paths.meets(Xy::new(u.0, u.1), Xy::new(v.0, v.1)).is_empty()
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

#[cfg(test)]
mod tests;
