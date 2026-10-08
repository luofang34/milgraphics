//! Port of the ICE_OPENINGS_* cases of clsMETOC.GetMeTOCShape and
//! ExtrapolatePointFromCurve: leads and frozen openings drawn as a pair of
//! splines along each partition of the points.

use super::MetocSupport;
use super::parallel_lines::parallel_lines2;
use super::path::GeneralPath;
use super::scaled_size;
use super::splines::{SEGMENT_END_STYLE, draw_splines};
use crate::engine::base::{At, EngineError, Pt, Shape, idx};
use crate::engine::tactical_lines::*;
use crate::engine::tg::Tg;

/// `ExtrapolatePointFromCurve`: the point on `spline` level with `pt` along
/// the direction `theta`, or none when the spline never straddles it.
pub(crate) fn extrapolate_point_from_curve(spline: &[Pt], pt: Pt, theta: f64) -> Option<Pt> {
    let (cos, sin) = (theta.cos(), theta.sin());
    let xp = pt.x * cos + pt.y * sin;
    for pair in spline.windows(2) {
        let (a, b) = (pair.first()?, pair.get(1)?);
        let x1p = a.x * cos + a.y * sin;
        let x2p = b.x * cos + b.y * sin;
        if (x1p <= xp && x2p >= xp) || (x1p >= xp && x2p <= xp) {
            let y1p = -a.x * sin + a.y * cos;
            let y2p = -b.x * sin + b.y * cos;
            let mp = (y2p - y1p) / (x2p - x1p);
            let yp = y1p + (xp - x1p) * mp;
            return Some(Pt::new(xp * cos - yp * sin, xp * sin + yp * cos));
        }
    }
    None
}

/// Splits sampled points back into one run per spline segment, at the
/// points marked with [`SEGMENT_END_STYLE`]. `current` carries the points
/// after the last marker into the next call, as upstream's shared list does.
fn split_runs(points: &[Pt], current: &mut Vec<Pt>) -> Vec<Vec<Pt>> {
    let mut runs = Vec::new();
    for p in points {
        if p.style == SEGMENT_END_STYLE {
            runs.push(std::mem::take(current));
        } else {
            current.push(*p);
        }
    }
    runs
}

fn tangent_angle(run: &[Pt], k: usize) -> Result<f64, EngineError> {
    let last = run.len().saturating_sub(1);
    let (a, b) = if k == 0 {
        (run.at(0)?, run.at(1)?)
    } else if k == last {
        (run.at(k - 1)?, run.at(k)?)
    } else {
        (run.at(k - 1)?, run.at(k + 1)?)
    };
    Ok((b.y - a.y).atan2(b.x - a.x))
}

/// The cross lines of a frozen opening: from each point of one edge to
/// where the other edge lies level with it.
fn frozen_connectors(lower: &[Pt], upper: &[Pt]) -> Result<GeneralPath, EngineError> {
    let mut carried = Vec::new();
    let runs_lower = split_runs(lower, &mut carried);
    let runs_upper = split_runs(upper, &mut carried);
    let mut path = GeneralPath::new();
    for j in 0..runs_lower.len() {
        if runs_upper.len() <= j {
            break;
        }
        // The cross lines start from the edge with fewer runs.
        let (run, other) = if runs_lower.len() >= runs_upper.len() {
            (run_at(&runs_lower, j)?, run_at(&runs_upper, j)?)
        } else {
            (run_at(&runs_upper, j)?, run_at(&runs_lower, j)?)
        };
        if run.len() == 1 {
            continue;
        }
        for k in 0..run.len() {
            let pt = run.at(k)?;
            let theta = tangent_angle(run, k)?;
            if let Some(pt2) = extrapolate_point_from_curve(other, pt, theta) {
                path.move_to(pt.x, pt.y);
                path.line_to(pt2.x, pt2.y)?;
            }
        }
    }
    Ok(path)
}

fn run_at(runs: &[Vec<Pt>], j: usize) -> Result<&[Pt], EngineError> {
    runs.get(j).map(Vec::as_slice).ok_or(EngineError::Index {
        index: i64::try_from(j).unwrap_or(i64::MAX),
        len: runs.len(),
    })
}

/// Draws one edge into `shapes`. With `own_list` the edge collects its
/// samples separately before adding them to `samples`; with `closes` the
/// path ends at the edge's last point.
fn draw_edge(
    tg: &Tg,
    shapes: &mut Vec<Shape>,
    samples: &mut Vec<Pt>,
    (own_list, closes): (bool, bool),
) -> Result<(), EngineError> {
    let mut path = if own_list {
        let mut own = Vec::new();
        let path = draw_splines(tg, &mut own);
        samples.extend_from_slice(&own);
        path
    } else {
        draw_splines(tg, samples)
    };
    if closes {
        let last = tg.pixels.at(tg.pixels.len().wrapping_sub(1))?;
        path.line_to(last.x, last.y)?;
    }
    shapes.push(path.into_polyline(0));
    Ok(())
}

/// The ICE_OPENINGS_LEAD, ICE_OPENINGS_LEAD_GE, ICE_OPENINGS_FROZEN and
/// ICE_OPENINGS_FROZEN_GE cases. `tg.pixels` is left as the last edge, as
/// upstream leaves it.
pub(crate) fn ice_openings(
    tg: &mut Tg,
    shapes: &mut Vec<Shape>,
    support: &dyn MetocSupport,
) -> Result<(), EngineError> {
    let lt = tg.line_type;
    let frozen = matches!(lt, ICE_OPENINGS_FROZEN | ICE_OPENINGS_FROZEN_GE);
    let mode = (
        lt == ICE_OPENINGS_FROZEN_GE,
        matches!(lt, ICE_OPENINGS_LEAD_GE | ICE_OPENINGS_FROZEN_GE),
    );
    let original = tg.pixels.clone();
    let partitions = support.partitions(tg)?;
    let mut spline_points: Vec<Pt> = Vec::new();
    let mut spline_points2: Vec<Pt> = Vec::new();
    for part in partitions {
        tg.pixels = original.clone();
        let start = idx(part.start, original.len())?;
        let stop = idx(part.end.wrapping_add(1), original.len())?;
        let mut pixels = Vec::new();
        for k in start..=stop {
            pixels.push(original.at(k)?);
        }
        if pixels.is_empty() {
            continue;
        }
        let width = scaled_size(20.0, f64::from(tg.line_thickness), tg.pattern_scale) as i32;
        let two = parallel_lines2(&pixels, width, support);
        let (upper, lower) = two.split_at(two.len() / 2);
        tg.pixels = lower.to_vec();
        draw_edge(tg, shapes, &mut spline_points, mode)?;
        tg.pixels = upper.to_vec();
        if lt == ICE_OPENINGS_LEAD_GE {
            // The upper edge of a Google Earth lead starts a fresh list,
            // which the next partition's lower edge then extends.
            spline_points = Vec::new();
            draw_edge(tg, shapes, &mut spline_points, mode)?;
        } else {
            draw_edge(tg, shapes, &mut spline_points2, mode)?;
        }
        if frozen {
            let connectors = frozen_connectors(&spline_points, &spline_points2)?;
            shapes.push(connectors.into_polyline(0));
        }
    }
    Ok(())
}
