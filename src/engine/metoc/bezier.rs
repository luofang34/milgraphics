//! Port of clsMETOC.getPointOnSegment and drawCubicBezier2: the cubic
//! segment between two spline points and the points sampled along it.

use super::path::GeneralPath;
use super::scaled_size;
use crate::engine::base::{EngineError, Pt};
use crate::engine::lineutility::basics::{calc_distance_double, mid_point_double};
use crate::engine::tactical_lines::*;
use crate::engine::tg::Tg;

/// Most points one quadratic piece may sample. Upstream is unbounded; this
/// keeps a degenerate scale from allocating without limit.
const MAX_SAMPLES_PER_PIECE: i32 = 100_000;

/// `getPointOnSegment`: the point at `ratio` of the way from `p0` to `p1`.
pub(crate) fn point_on_segment(p0: Pt, p1: Pt, ratio: f64) -> Pt {
    Pt::new(p0.x + (p1.x - p0.x) * ratio, p0.y + (p1.y - p0.y) * ratio)
}

/// Types whose spline is drawn as one true cubic and nothing else.
fn is_solid_curve(line_type: i32) -> bool {
    matches!(
        line_type,
        ISOBAR
            | UPPER_AIR
            | ISODROSOTHERM
            | ICE_EDGE
            | CRACKS
            | DEPTH_CURVE
            | DEPTH_CONTOUR
            | COASTLINE
            | PIER
            | RAMP_ABOVE_WATER
            | JETTY_ABOVE_WATER
            | SEAWALL
            | ICE_OPENINGS_LEAD
            | ISOTACH
            | ISOTHERM
            | ISOPLETHS
            | ESTIMATED_ICE_EDGE
            | RAMP_BELOW_WATER
            | JETTY_BELOW_WATER
    )
}

/// Types that draw the cubic and still need the sampled points for their
/// other features.
fn is_curve_with_samples(line_type: i32) -> bool {
    matches!(
        line_type,
        ICE_OPENINGS_LEAD_GE
            | SEAWALL_GE
            | JETTY_BELOW_WATER_GE
            | JETTY_ABOVE_WATER_GE
            | RAMP_ABOVE_WATER_GE
            | RAMP_BELOW_WATER_GE
            | PIER_GE
            | COASTLINE_GE
            | DEPTH_CONTOUR_GE
            | DEPTH_CURVE_GE
            | CRACKS_GE
            | ESTIMATED_ICE_EDGE_GE
            | ICE_EDGE_GE
            | ISOPLETHS_GE
            | ISOTACH_GE
            | ISOTHERM_GE
            | ISOBAR_GE
            | UPPER_AIR_GE
            | ISODROSOTHERM_GE
            | ICE_OPENINGS_FROZEN
            | ICE_OPENINGS_FROZEN_GE
            | ICE_EDGE_RADAR
            | ICE_EDGE_RADAR_GE
            | CRACKS_SPECIFIC_LOCATION
            | CRACKS_SPECIFIC_LOCATION_GE
            | EBB_TIDE
            | FLOOD_TIDE
            | EBB_TIDE_GE
            | FLOOD_TIDE_GE
            | JET
            | STREAM
            | JET_GE
            | STREAM_GE
    )
}

/// Spacing of the sampled points along the curve.
fn sample_increment(tg: &Tg) -> f64 {
    let size = match tg.line_type {
        ICE_EDGE_RADAR | ICE_EDGE_RADAR_GE => 20.0,
        ICE_OPENINGS_FROZEN
        | ICE_OPENINGS_FROZEN_GE
        | CRACKS_SPECIFIC_LOCATION
        | CRACKS_SPECIFIC_LOCATION_GE => 7.0,
        _ => 10.0,
    };
    scaled_size(size, f64::from(tg.line_thickness), tg.pattern_scale)
}

/// One quadratic piece of the cubic approximation, as upstream samples it:
/// `n` points from `t = 0` in steps of `increment / distance`.
fn sample_quadratic(
    out: &mut Vec<Pt>,
    (p0, p1, p2): (Pt, Pt, Pt),
    n: i32,
    increment: f64,
    distance: f64,
) -> Result<(), EngineError> {
    if n > MAX_SAMPLES_PER_PIECE {
        return Err(EngineError::Degenerate(
            "spline sampling exceeds its budget",
        ));
    }
    for j in 0..n {
        let t = f64::from(j) * (increment / distance);
        let x = (1.0 - t) * (1.0 - t) * p0.x + 2.0 * (1.0 - t) * t * p1.x + t * t * p2.x;
        let y = (1.0 - t) * (1.0 - t) * p0.y + 2.0 * (1.0 - t) * t * p1.y + t * t * p2.y;
        out.push(Pt::new(x, y));
    }
    Ok(())
}

/// `drawCubicBezier2`: draws the cubic `p0..p3` into `path` for the types
/// that draw it, and returns the points sampled along the curve (none for
/// the types that only draw the cubic).
pub(crate) fn draw_cubic_bezier2(
    tg: &Tg,
    path: &mut GeneralPath,
    p0: Pt,
    p1: Pt,
    p2: Pt,
    p3: Pt,
) -> Result<Vec<Pt>, EngineError> {
    let pa = point_on_segment(p0, p1, 0.75);
    let pb = point_on_segment(p3, p2, 0.75);
    let dx = (p3.x - p0.x) / 16.0;
    let dy = (p3.y - p0.y) / 16.0;
    let pc_1 = point_on_segment(p0, p1, 0.375);
    let mut pc_2 = point_on_segment(pa, pb, 0.375);
    pc_2.x -= dx;
    pc_2.y -= dy;
    let mut pc_3 = point_on_segment(pb, pa, 0.375);
    pc_3.x += dx;
    pc_3.y += dy;
    let pc_4 = point_on_segment(p3, p2, 0.375);
    let pa_1 = mid_point_double(pc_1, pc_2, 0);
    let pa_2 = mid_point_double(pa, pb, 0);
    let pa_3 = mid_point_double(pc_3, pc_4, 0);

    let mut array = Vec::new();
    if is_solid_curve(tg.line_type) {
        path.move_to(p0.x, p0.y);
        path.curve_to(p1, p2, p3)?;
        return Ok(array);
    }
    if is_curve_with_samples(tg.line_type) {
        path.move_to(p0.x, p0.y);
        path.curve_to(p1, p2, p3)?;
    }

    let increment = sample_increment(tg);
    // The first piece never samples fewer than one step.
    let mut distance = calc_distance_double(p0, pa_1);
    if distance < increment {
        distance = increment;
    }
    let n = (distance / increment) as i32;
    sample_quadratic(&mut array, (p0, pc_1, pa_1), n, increment, distance)?;

    let pieces = [(pa_1, pc_2, pa_2), (pa_2, pc_3, pa_3), (pa_3, pc_4, p3)];
    for piece in pieces {
        let distance = calc_distance_double(piece.0, piece.2);
        let n = (distance / increment) as i32;
        sample_quadratic(&mut array, piece, n, increment, distance)?;
    }
    Ok(array)
}
