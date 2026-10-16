//! The segments of the renderer's lines that are proportional to the
//! control points although their shape is not.
//!
//! A mission task draws its arc, bar or arrow body in proportion to its
//! control points and its arrowheads or ticks in pixels, often in one
//! polyline. Where a polyline has the same vertices at both pixel sizes,
//! a segment whose two ends land on the same ground both times is a
//! geographic line, and only the others are drawn per view.

use crate::construction::{GeoGeometry, PartId, PartRole};
use crate::family::Ctx;
use crate::geo::GeoPoint;
use crate::plane::LocalPlane;
use crate::style::Fill;

use super::baseline::{Run, stroked};
use super::classify::SAME_FRACTION;
use super::{from_px, shape_stroke};

/// Adds the segments both runs draw at the same place on the ground as
/// geographic lines, each chain of them one part, from the larger run. The
/// first part's id, with each chain's points.
pub(super) fn add(
    ctx: &mut Ctx<'_>,
    plane: &LocalPlane<'_>,
    (a, b): &(Run<'_>, Run<'_>),
    extent: f64,
) -> (Option<PartId>, Vec<Vec<GeoPoint>>) {
    let mut first = None;
    let mut found = Vec::new();
    if a.shapes().len() != b.shapes().len() {
        return (first, found);
    }
    let tolerance = SAME_FRACTION * extent;
    for (s, (sa, sb)) in a.shapes().iter().zip(b.shapes()).enumerate() {
        if a.skips(s) || !stroked(sa) || !stroked(sb) {
            continue;
        }
        let stroke = shape_stroke(sb, ctx.palette.line.color);
        for (la, lb) in sa.polylines().iter().zip(sb.polylines()) {
            let Some(segments) = same(la, a.mpp(), &lb, b.mpp(), tolerance) else {
                continue;
            };
            for chain in chains(&segments) {
                let Some(px) = lb.get(chain) else { continue };
                let points: Vec<GeoPoint> = px
                    .iter()
                    .map(|&p| plane.to_geo(from_px(p, b.mpp())))
                    .collect();
                let id = ctx.add_part(
                    PartRole::Line,
                    GeoGeometry::Line(points.clone()),
                    Some(stroke),
                    Fill::None,
                );
                first.get_or_insert(id);
                found.push(points);
            }
        }
    }
    (first, found)
}

/// For two drawings of one polyline with the same vertex count, whether each
/// segment's ends land within `tolerance` of each other on the ground.
fn same(
    a: &[(f64, f64)],
    mpp_a: f64,
    b: &[(f64, f64)],
    mpp_b: f64,
    tolerance: f64,
) -> Option<Vec<bool>> {
    if a.len() != b.len() || a.len() < 2 {
        return None;
    }
    let at =
        |p: (f64, f64), q: (f64, f64)| from_px(p, mpp_a).sub(from_px(q, mpp_b)).len() <= tolerance;
    let still: Vec<bool> = a.iter().zip(b).map(|(&p, &q)| at(p, q)).collect();
    // A repeated vertex is no segment to draw.
    let long = |w: &[(f64, f64)]| match w {
        &[p, q] => from_px(p, mpp_b).sub(from_px(q, mpp_b)).len() > tolerance,
        _ => false,
    };
    Some(
        still
            .windows(2)
            .zip(b.windows(2))
            .map(|(s, w)| s == [true, true] && long(w))
            .collect(),
    )
}

/// Vertex ranges of the runs of consecutive geographic segments.
fn chains(segments: &[bool]) -> Vec<core::ops::Range<usize>> {
    let mut out = Vec::new();
    let mut start = None;
    for (j, &on) in segments.iter().chain([&false]).enumerate() {
        match (on, start) {
            (true, None) => start = Some(j),
            (false, Some(s)) => {
                out.push(s..j + 1);
                start = None;
            }
            _ => {}
        }
    }
    out
}

#[cfg(test)]
mod tests;
