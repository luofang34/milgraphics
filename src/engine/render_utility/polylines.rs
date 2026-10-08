//! Ports of `clsUtilityGE.setSplineLinetype` and `SetShapeInfosPolylines`
//! (with `createRenderablesFromShape`, `createSimpleFillShape`,
//! `createSimplePatternFillShape` and `createDashedPolylines`): the last
//! step that turns shapes into plain polylines.

use crate::engine::base::{PathOp, Pt, Shape, Stroke, shape_type};
use crate::engine::line_type::classes::is_change1_area;
use crate::engine::line_type::is_weather;
use crate::engine::lineutility::extend::extend_along_line_double2;
use crate::engine::tactical_lines::*;
use crate::engine::tg::Tg;
use crate::engine::tg_utility::line_classes::is_closed_polygon;

/// Spline line types and the twins the GE client draws instead.
const SPLINE_TWINS: [(i32, i32); 28] = [
    (BRDGHD, BRDGHD_GE),
    (HOLD, HOLD_GE),
    (ICE_OPENINGS_FROZEN, ICE_OPENINGS_FROZEN_GE),
    (ICE_OPENINGS_LEAD, ICE_OPENINGS_LEAD_GE),
    (ICE_EDGE_RADAR, ICE_EDGE_RADAR_GE),
    (CRACKS_SPECIFIC_LOCATION, CRACKS_SPECIFIC_LOCATION_GE),
    (JET, JET_GE),
    (STREAM, STREAM_GE),
    (FLOOD_TIDE, FLOOD_TIDE_GE),
    (EBB_TIDE, EBB_TIDE_GE),
    (SEAWALL, SEAWALL_GE),
    (JETTY_BELOW_WATER, JETTY_BELOW_WATER_GE),
    (JETTY_ABOVE_WATER, JETTY_ABOVE_WATER_GE),
    (RAMP_BELOW_WATER, RAMP_BELOW_WATER_GE),
    (RAMP_ABOVE_WATER, RAMP_ABOVE_WATER_GE),
    (PIER, PIER_GE),
    (COASTLINE, COASTLINE_GE),
    (DEPTH_CONTOUR, DEPTH_CONTOUR_GE),
    (DEPTH_CURVE, DEPTH_CURVE_GE),
    (CRACKS, CRACKS_GE),
    (ESTIMATED_ICE_EDGE, ESTIMATED_ICE_EDGE_GE),
    (ICE_EDGE, ICE_EDGE_GE),
    (ISOTHERM, ISOTHERM_GE),
    (UPPER_AIR, UPPER_AIR_GE),
    (ISOBAR, ISOBAR_GE),
    (ISODROSOTHERM, ISODROSOTHERM_GE),
    (ISOTACH, ISOTACH_GE),
    (ISOPLETHS, ISOPLETHS_GE),
];

/// Upstream `setSplineLinetype`: METOC spline types become their GE twins
/// before drawing.
pub(crate) fn set_spline_linetype(tg: &mut Tg) {
    if let Some((_, twin)) = SPLINE_TWINS.iter().find(|(from, _)| *from == tg.line_type) {
        tg.line_type = *twin;
    }
}

type Polyline = Vec<(f64, f64)>;

/// Upstream `createRenderablesFromShape`: the path cut into polylines at
/// each move. Fill shapes get their first point appended when open. A
/// polyline of one point is dropped.
fn renderables(shape: &Shape) -> Vec<Polyline> {
    let close = shape.shape_type == shape_type::FILL;
    let mut out: Vec<Polyline> = Vec::new();
    let mut current: Polyline = Vec::new();
    let flush = |current: &mut Polyline, out: &mut Vec<Polyline>| {
        if close {
            if let (Some(&first), Some(&last)) = (current.first(), current.last()) {
                if first != last {
                    current.push(first);
                }
            }
        }
        if current.len() > 1 {
            out.push(std::mem::take(current));
        }
    };
    for op in &shape.path {
        match *op {
            PathOp::MoveTo(x, y) => {
                if !current.is_empty() {
                    flush(&mut current, &mut out);
                }
                current = vec![(x, y)];
            }
            PathOp::LineTo(x, y) => current.push((x, y)),
        }
    }
    if current.len() > 1 {
        flush(&mut current, &mut out);
    }
    out
}

fn path_of(lines: &[Polyline]) -> Vec<PathOp> {
    let mut path = Vec::new();
    for line in lines {
        for (i, &(x, y)) in line.iter().enumerate() {
            path.push(if i == 0 {
                PathOp::MoveTo(x, y)
            } else {
                PathOp::LineTo(x, y)
            });
        }
    }
    path
}

fn has_dash(shape: &Shape) -> bool {
    shape.stroke.dash.as_ref().is_some_and(|d| d.len() >= 2)
}

fn dashed_closed_area(tg: &Tg, shape: &Shape) -> bool {
    (is_closed_polygon(tg.line_type) || is_change1_area(tg.line_type)) && has_dash(shape)
}

/// Upstream `createSimpleFillShape` / `createSimplePatternFillShape`: a
/// dashed outline cannot carry the area fill, so the fill (or hatch) becomes
/// a shape of its own and the outline loses it.
fn split_off_fill(tg: &Tg, shape: &mut Shape, lines: &[Polyline]) -> Option<Shape> {
    if !dashed_closed_area(tg, shape) {
        return None;
    }
    let mut fill = Shape::new(shape_type::FILL);
    fill.stroke = Stroke {
        width: 0.0,
        dash: None,
    };
    fill.path = path_of(lines);
    if let Some(color) = shape.fill_color {
        fill.fill_color = Some(color);
        shape.fill_color = None;
    } else {
        fill.pattern_fill = Some(shape.pattern_fill?);
        shape.pattern_fill = None;
    }
    shape.shape_type = shape_type::POLYLINE;
    Some(fill)
}

/// Upstream `allowFillForThese`: the types that keep a fill on their
/// outline shapes even next to a separate fill shape.
fn allow_fill(tg: &Tg) -> bool {
    if let Some(info) = symbol_set_entity(&tg.symbol_id) {
        if is_weather(info.0, info.1).is_some() {
            return true;
        }
    }
    matches!(
        tg.line_type,
        BBS_AREA
            | BBS_RECTANGLE
            | CATK
            | CATKBYFIRE
            | AIRAOA
            | AAAAA
            | MAIN
            | SPT
            | FRONTAL_ATTACK
            | TURNING_MOVEMENT
            | MOVEMENT_TO_CONTACT
            | SARA
            | RANGE_FAN_SECTOR
            | RADAR_SEARCH
            | RANGE_FAN
            | MNFLDFIX
            | TURN_REVD
            | TURN
            | MNFLDDIS
            | EASY
            | ATDITCHC
            | ATDITCHM
            | FERRY
            | BYDIF
            | BYIMP
            | DEPTH_AREA
    )
}

/// Symbol set and entity code of a symbol code of 16 or more digits.
fn symbol_set_entity(code: &str) -> Option<(u8, u32)> {
    if code.len() <= 15 {
        return None;
    }
    let set = code.get(4..6)?.parse().ok()?;
    let entity = code.get(10..16)?.parse().ok()?;
    Some((set, entity))
}

/// Upstream `SetShapeInfosPolylines` (no clipping): rewrites every shape's
/// path as the polylines `createRenderablesFromShape` makes (so a one-point
/// piece is gone and fill rings are closed), and moves the fill of a dashed
/// area to a shape of its own at the front. Dashes stay on the stroke when
/// `tg.use_dash_array` is set (upstream's default); otherwise the polylines
/// are cut into the dashes.
pub(crate) fn shapes_to_polylines(tg: &Tg, shapes: &mut Vec<Shape>) {
    let mut simple_fill: Option<Shape> = None;
    for shape in shapes.iter_mut() {
        let mut lines = renderables(shape);
        if simple_fill.is_none() {
            simple_fill = split_off_fill(tg, shape, &lines);
        }
        if simple_fill.is_some() && !allow_fill(tg) {
            shape.fill_color = None;
        }
        if !tg.use_dash_array {
            lines = dashed(shape, lines);
        }
        shape.path = path_of(&lines);
    }
    if let Some(fill) = simple_fill {
        shapes.insert(0, fill);
    }
}

/// Upstream `createDashedPolylines`: each polyline cut into its dashes.
fn dashed(shape: &Shape, lines: Vec<Polyline>) -> Vec<Polyline> {
    let Some(dash) = shape.stroke.dash.as_ref().filter(|d| d.len() >= 2) else {
        return lines;
    };
    if shape.line_color.is_none() {
        return lines;
    }
    let mut out: Vec<Polyline> = Vec::new();
    for line in &lines {
        let mut index = 0usize;
        let mut remaining = dash.first().copied().unwrap_or(0.0);
        for pair in line.windows(2) {
            let (Some(&a), Some(&b)) = (pair.first(), pair.get(1)) else {
                continue;
            };
            let end = Pt::new(b.0, b.1);
            let mut start = Pt::new(a.0, a.1);
            loop {
                let len = ((end.x - start.x).powi(2) + (end.y - start.y).powi(2)).sqrt();
                if len <= 0.0 {
                    break;
                }
                if len < remaining {
                    if index % 2 == 0 {
                        out.push(vec![(start.x, start.y), (end.x, end.y)]);
                    }
                    remaining -= len;
                    break;
                }
                let flip = extend_along_line_double2(start, end, remaining);
                if index % 2 == 0 {
                    out.push(vec![(start.x, start.y), (flip.x, flip.y)]);
                }
                index = (index + 1) % dash.len();
                remaining = dash.get(index).copied().unwrap_or(0.0);
                start = flip;
            }
        }
    }
    out
}
