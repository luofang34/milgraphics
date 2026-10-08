//! Port of clsMETOC.DrawSplines and DrawArrow: the spline through a
//! graphic's points, with the arrowheads, feathers and ticks some types add
//! along it.

use super::bezier::draw_cubic_bezier2;
use super::path::GeneralPath;
use super::scaled_size;
use crate::engine::base::{At, EngineError, Pt};
use crate::engine::lineutility::extend::{
    extend_along_line_double, extend_angled_line, extend_directed_line, extend_line_double,
};
use crate::engine::tactical_lines::*;
use crate::engine::tg::Tg;

/// Style marker upstream puts on the last sampled point of each segment so
/// the frozen-opening code can split the samples back into segments.
pub(crate) const SEGMENT_END_STYLE: i32 = 47;

/// `DrawArrow`: an arrowhead with its tip at `tip`, opening away from
/// `base`, `size` pixels deep.
fn draw_arrow(tip: Pt, base: Pt, size: f64, path: &mut GeneralPath) -> Result<(), EngineError> {
    let pt_base = extend_along_line_double(base, tip, size);
    let left = extend_directed_line(tip, pt_base, pt_base, 2, size);
    let right = extend_directed_line(tip, pt_base, pt_base, 3, size);
    path.move_to(left.x, left.y);
    path.line_to(base.x, base.y)?;
    path.line_to(right.x, right.y)
}

/// Appends the polyline through `points`, truncated to whole pixels as the
/// Google Earth variants do.
fn line_through_truncated(path: &mut GeneralPath, points: &[Pt]) -> Result<(), EngineError> {
    let first = points.at(0)?;
    path.move_to(f64::from(first.x as i32), f64::from(first.y as i32));
    for p in points.iter().skip(1) {
        path.line_to(f64::from(p.x as i32), f64::from(p.y as i32))?;
    }
    Ok(())
}

fn last_pixel(tg: &Tg) -> Result<Pt, EngineError> {
    tg.pixels.at(tg.pixels.len().wrapping_sub(1))
}

fn second_to_last(points: &[Pt]) -> Option<Pt> {
    let n = points.len();
    if n >= 2 {
        points.get(n - 2).copied()
    } else {
        None
    }
}

/// The two feather strokes at the start of a flood tide.
fn flood_tide_feathers(path: &mut GeneralPath, samples: &[Pt], d: f64) -> Result<(), EngineError> {
    let pt0 = samples.at(0)?;
    let pt1 = samples.at(1)?;
    let pt2 = extend_line_double(pt0, pt1, d);
    let pt3 = extend_line_double(pt0, pt1, d * 2.0);
    let pt4 = extend_line_double(pt0, pt1, d * 3.0);
    let pt5 = extend_directed_line(pt3, pt2, pt2, 3, d);
    let pt6 = extend_directed_line(pt4, pt3, pt3, 3, d);
    path.move_to(pt3.x, pt3.y);
    path.line_to(pt5.x, pt5.y)?;
    path.move_to(pt4.x, pt4.y);
    path.line_to(pt6.x, pt6.y)
}

/// The short strokes either side of the sampled points of radar ice edges.
fn radar_ticks(path: &mut GeneralPath, samples: &[Pt], d: f64) -> Result<(), EngineError> {
    for j in 0..samples.len().saturating_sub(1) {
        let here = samples.at(j)?;
        let next = samples.at(j + 1)?;
        for alpha in [45.0, 135.0] {
            let a = extend_angled_line(here, next, here, alpha, d);
            let b = extend_angled_line(here, next, here, -alpha, d);
            path.move_to(here.x, here.y);
            path.line_to(a.x, a.y)?;
            path.move_to(here.x, here.y);
            path.line_to(b.x, b.y)?;
        }
    }
    Ok(())
}

/// The perpendicular strokes across a crack at a specific location.
fn crack_ticks(path: &mut GeneralPath, samples: &[Pt], d: f64) -> Result<(), EngineError> {
    for j in 0..samples.len().saturating_sub(1) {
        let here = samples.at(j)?;
        let next = samples.at(j + 1)?;
        let a = extend_directed_line(here, next, next, 2, d);
        path.move_to(a.x, a.y);
        let b = extend_directed_line(here, next, next, 3, d);
        path.line_to(b.x, b.y)?;
    }
    Ok(())
}

/// Types whose path is replaced by the sampled polyline on the last segment.
fn replaces_path_with_samples(line_type: i32) -> bool {
    matches!(
        line_type,
        ICE_OPENINGS_FROZEN_GE
            | ICE_OPENINGS_LEAD_GE
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
            | ISODROSOTHERM_GE
            | ISOTACH_GE
            | ISOTHERM_GE
            | UPPER_AIR_GE
            | ISOBAR_GE
    )
}

/// The per-segment features of the switch at the end of `DrawSplines`'s
/// loop body. `samples` are this segment's points, `all` every point
/// sampled so far; `last` is true on the final segment.
fn segment_features(
    tg: &Tg,
    path: &mut GeneralPath,
    (samples, all): (&[Pt], &[Pt]),
    (i, last): (usize, bool),
) -> Result<(), EngineError> {
    let scaled = |size: f64| scaled_size(size, f64::from(tg.line_thickness), tg.pattern_scale);
    let tail_arrow = |path: &mut GeneralPath, d: f64| -> Result<(), EngineError> {
        match second_to_last(samples) {
            Some(tip) => draw_arrow(tip, last_pixel(tg)?, d, path),
            None => Ok(()),
        }
    };
    let walk_arrow = |path: &mut GeneralPath| -> Result<(), EngineError> {
        if samples.len() > i + 1 {
            draw_arrow(samples.at(i + 1)?, samples.at(i)?, scaled(10.0), path)?;
        }
        Ok(())
    };
    match tg.line_type {
        EBB_TIDE if last => tail_arrow(path, scaled(10.0)),
        FLOOD_TIDE | FLOOD_TIDE_GE => {
            let d = scaled(10.0);
            if i == 0 && samples.len() > 1 {
                flood_tide_feathers(path, samples, d)?;
            }
            if last {
                if tg.line_type == FLOOD_TIDE_GE {
                    line_through_truncated(path, all)?;
                }
                tail_arrow(path, d)?;
            }
            Ok(())
        }
        STREAM | JET => walk_arrow(path),
        EBB_TIDE_GE if last => {
            *path = GeneralPath::new();
            line_through_truncated(path, all)?;
            tail_arrow(path, scaled(10.0))
        }
        JET_GE | STREAM_GE => {
            walk_arrow(path)?;
            if last {
                line_through_truncated(path, all)
            } else {
                Ok(())
            }
        }
        ICE_EDGE_RADAR | ICE_EDGE_RADAR_GE => {
            radar_ticks(path, samples, scaled(5.0))?;
            if last && tg.line_type == ICE_EDGE_RADAR_GE {
                line_through_truncated(path, all)?;
            }
            Ok(())
        }
        CRACKS_SPECIFIC_LOCATION | CRACKS_SPECIFIC_LOCATION_GE => {
            crack_ticks(path, samples, scaled(5.0))?;
            if last && tg.line_type == CRACKS_SPECIFIC_LOCATION_GE {
                line_through_truncated(path, all)?;
            }
            Ok(())
        }
        lt if replaces_path_with_samples(lt) => {
            if !all.is_empty() {
                *path = GeneralPath::new();
                if last {
                    line_through_truncated(path, all)?;
                }
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

/// The loop of `DrawSplines`; errors end it, leaving `path` and
/// `spline_points` as they were.
fn trace_splines(
    tg: &Tg,
    path: &mut GeneralPath,
    spline_points: &mut Vec<Pt>,
) -> Result<(), EngineError> {
    let array = &tg.pixels;
    let n = array.len();
    // tension / 0.5 * 0.175 with tension 0.33
    let control_scale = 0.33 / 0.5 * 0.175;
    for i in 0..n.saturating_sub(1) {
        let pt = array.at(i)?;
        let pt_before = if i == 0 {
            path.move_to(pt.x, pt.y);
            pt
        } else {
            array.at(i - 1)?
        };
        let pt2 = array.at(i + 1)?;
        let tail = array.at(n - 1)?;
        let pt_after = if i + 2 < n { array.at(i + 1)? } else { tail };
        let pt_after2 = if i + 2 < n { array.at(i + 2)? } else { tail };
        let p2 = Pt::new(
            pt.x + control_scale * (pt_after.x - pt_before.x),
            pt.y + control_scale * (pt_after.y - pt_before.y),
        );
        let p3 = Pt::new(
            pt_after.x - control_scale * (pt_after2.x - pt.x),
            pt_after.y - control_scale * (pt_after2.y - pt.y),
        );
        let mut samples = draw_cubic_bezier2(tg, path, pt, p2, p3, pt2)?;
        if matches!(tg.line_type, ICE_OPENINGS_FROZEN | ICE_OPENINGS_FROZEN_GE) {
            if let Some(end) = samples.last_mut() {
                end.style = SEGMENT_END_STYLE;
            }
        }
        spline_points.extend_from_slice(&samples);
        segment_features(tg, path, (&samples, spline_points), (i, i + 2 == n))?;
    }
    Ok(())
}

/// `DrawSplines`: the path of the spline through `tg.pixels`. The points
/// sampled along it are appended to `spline_points`. Like upstream, a
/// failure partway keeps what was drawn so far.
pub(crate) fn draw_splines(tg: &Tg, spline_points: &mut Vec<Pt>) -> GeneralPath {
    let mut path = GeneralPath::new();
    trace_splines(tg, &mut path, spline_points).ok();
    path
}
