//! Port of clsMETOC.GetMeTOCShape: builds the shapes of one METOC type.

use super::ice_openings::ice_openings;
use super::path::GeneralPath;
use super::properties::set_metoc_properties;
use super::shape_properties::set_shape_properties;
use super::splines::draw_splines;
use super::{MetocSupport, PatternFill};
use crate::engine::base::{At, EngineError, Pt, Shape};
use crate::engine::settings::Settings;
use crate::engine::tactical_lines::*;
use crate::engine::tg::Tg;

/// Types the general line builder draws.
fn uses_line_array(lt: i32) -> bool {
    matches!(
        lt,
        SF | USF
            | SFG
            | SFY
            | WFY
            | WFG
            | WF
            | UWF
            | UCF
            | CF
            | CFG
            | CFY
            | OCCLUDED
            | UOF
            | OFY
            | TROUGH
            | UPPER_TROUGH
            | CABLE
            | INSTABILITY
            | SHEAR
            | RIDGE
            | SQUALL
            | ITC
            | CONVERGENCE
            | ITD
            | IFR
            | MVFR
            | TURBULENCE
            | ICING
            | NON_CONVECTIVE
            | CONVECTIVE
            | FROZEN
            | THUNDERSTORMS
            | FOG
            | SAND
            | FREEFORM
            | OPERATOR_FREEFORM
            | LVO
            | UNDERCAST
            | LRO
    ) || uses_line_array_areas(lt)
        || uses_line_array_bottom(lt)
}

fn uses_line_array_areas(lt: i32) -> bool {
    matches!(
        lt,
        DEPTH_AREA
            | ISLAND
            | BEACH
            | WATER
            | FISH_TRAPS
            | SWEPT_AREA
            | OIL_RIG_FIELD
            | FOUL_GROUND
            | KELP
            | BEACH_SLOPE_MODERATE
            | BEACH_SLOPE_STEEP
            | ANCHORAGE_AREA
            | ANCHORAGE_LINE
            | PIPE
            | TRAINING_AREA
            | RESTRICTED_AREA
            | REEF
            | FORESHORE_AREA
            | FORESHORE_LINE
            | DRYDOCK
            | LOADING_FACILITY_LINE
            | LOADING_FACILITY_AREA
            | PERCHES
            | UNDERWATER_HAZARD
            | BREAKERS
            | DISCOLORED_WATER
            | BEACH_SLOPE_FLAT
            | BEACH_SLOPE_GENTLE
            | MARITIME_LIMIT
            | MARITIME_AREA
            | OPERATOR_DEFINED
            | SUBMERGED_CRIB
            | CANAL
    )
}

fn uses_line_array_bottom(lt: i32) -> bool {
    matches!(
        lt,
        VDR_LEVEL_12
            | VDR_LEVEL_23
            | VDR_LEVEL_34
            | VDR_LEVEL_45
            | VDR_LEVEL_56
            | VDR_LEVEL_67
            | VDR_LEVEL_78
            | VDR_LEVEL_89
            | VDR_LEVEL_910
            | SOLID_ROCK
            | CLAY
            | VERY_COARSE_SAND
            | COARSE_SAND
            | MEDIUM_SAND
            | FINE_SAND
            | VERY_FINE_SAND
            | VERY_FINE_SILT
            | FINE_SILT
            | MEDIUM_SILT
            | COARSE_SILT
            | BOULDERS
            | OYSTER_SHELLS
            | PEBBLES
            | SAND_AND_SHELLS
            | BOTTOM_SEDIMENTS_LAND
            | BOTTOM_SEDIMENTS_NO_DATA
            | BOTTOM_ROUGHNESS_SMOOTH
            | BOTTOM_ROUGHNESS_MODERATE
            | BOTTOM_ROUGHNESS_ROUGH
            | CLUTTER_LOW
            | CLUTTER_MEDIUM
            | CLUTTER_HIGH
            | IMPACT_BURIAL_0
            | IMPACT_BURIAL_10
            | IMPACT_BURIAL_20
            | IMPACT_BURIAL_75
            | IMPACT_BURIAL_100
            | BOTTOM_CATEGORY_A
            | BOTTOM_CATEGORY_B
            | BOTTOM_CATEGORY_C
            | BOTTOM_TYPE_A1
            | BOTTOM_TYPE_A2
            | BOTTOM_TYPE_A3
            | BOTTOM_TYPE_B1
            | BOTTOM_TYPE_B2
            | BOTTOM_TYPE_B3
            | BOTTOM_TYPE_C1
            | BOTTOM_TYPE_C2
            | BOTTOM_TYPE_C3
    )
}

/// Splines that run on to the last control point.
fn is_spline_to_end(lt: i32) -> bool {
    matches!(
        lt,
        ISOBAR
            | ISOBAR_GE
            | UPPER_AIR
            | UPPER_AIR_GE
            | ISOTHERM
            | ISOTHERM_GE
            | ISOTACH
            | ISOTACH_GE
            | ISODROSOTHERM
            | ISODROSOTHERM_GE
            | ISOPLETHS
            | ISOPLETHS_GE
            | ICE_EDGE
            | ICE_EDGE_GE
            | ESTIMATED_ICE_EDGE
            | ESTIMATED_ICE_EDGE_GE
            | CRACKS
            | CRACKS_GE
            | DEPTH_CURVE
            | DEPTH_CURVE_GE
            | DEPTH_CONTOUR
            | DEPTH_CONTOUR_GE
            | COASTLINE
            | COASTLINE_GE
            | PIER
            | PIER_GE
            | RAMP_ABOVE_WATER
            | RAMP_ABOVE_WATER_GE
            | RAMP_BELOW_WATER
            | RAMP_BELOW_WATER_GE
            | JETTY_ABOVE_WATER
            | JETTY_ABOVE_WATER_GE
            | JETTY_BELOW_WATER
            | JETTY_BELOW_WATER_GE
            | SEAWALL
            | SEAWALL_GE
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

/// LEADING_LINE: a solid first half and a dashed second half along the
/// spline.
fn leading_line(tg: &Tg, shapes: &mut Vec<Shape>, spline: &[Pt]) -> Result<bool, EngineError> {
    let Some(first) = spline.first() else {
        // No spline: the points as a dashed polyline, and no properties.
        let mut path = GeneralPath::new();
        let start = tg.pixels.at(0)?;
        path.move_to(start.x, start.y);
        for p in &tg.pixels {
            path.line_to(p.x, p.y)?;
        }
        shapes.push(path.into_polyline(1));
        return Ok(false);
    };
    let n = spline.len() / 2;
    let mut path = GeneralPath::new();
    path.move_to(first.x, first.y);
    for j in 1..=n {
        let p = spline.at(j)?;
        path.line_to(p.x, p.y)?;
    }
    shapes.push(path.into_polyline(0));

    let mut dashed = GeneralPath::new();
    let mid = spline.at(n)?;
    dashed.move_to(mid.x, mid.y);
    for p in spline.iter().skip(n + 1) {
        dashed.line_to(p.x, p.y)?;
    }
    shapes.push(dashed.into_polyline(1));
    Ok(true)
}

/// The shapes of every type but the line-array ones, the ice openings and
/// the leading line. Returns the sampled spline points.
fn spline_shapes(tg: &Tg, shapes: &mut Vec<Shape>, pt_last: Pt) -> Result<Vec<Pt>, EngineError> {
    let mut spline_points = Vec::new();
    let mut path = draw_splines(tg, &mut spline_points);
    if is_spline_to_end(tg.line_type) {
        path.line_to(pt_last.x, pt_last.y)?;
    }
    shapes.push(path.into_polyline(0));
    Ok(spline_points)
}

/// What the dispatch left to do.
enum Dispatched {
    /// Continue with the closing segment and the shape properties.
    Splines(Vec<Pt>),
    /// Skip the closing segment, keep the properties.
    NoClosing,
    /// Skip both.
    Finished,
}

fn dispatch(
    tg: &mut Tg,
    shapes: &mut Vec<Shape>,
    pt_last: Pt,
    support: &dyn MetocSupport,
) -> Result<Dispatched, EngineError> {
    let lt = tg.line_type;
    if lt == ITCZ_LADDER {
        let scale = f64::from(tg.line_thickness) / 3.0;
        shapes.extend(super::itcz::shapes(&tg.pixels, scale));
        return Ok(Dispatched::NoClosing);
    }
    if uses_line_array(lt) {
        support.line_array(tg, shapes)?;
        return Ok(Dispatched::Splines(Vec::new()));
    }
    match lt {
        CRACKS_SPECIFIC_LOCATION
        | CRACKS_SPECIFIC_LOCATION_GE
        | ICE_EDGE_RADAR
        | ICE_EDGE_RADAR_GE => {
            let mut spline_points = Vec::new();
            let path = draw_splines(tg, &mut spline_points);
            shapes.push(path.into_polyline(0));
            if lt == ICE_EDGE_RADAR {
                return Ok(Dispatched::NoClosing);
            }
            Ok(Dispatched::Splines(spline_points))
        }
        ICE_OPENINGS_LEAD | ICE_OPENINGS_LEAD_GE | ICE_OPENINGS_FROZEN | ICE_OPENINGS_FROZEN_GE => {
            ice_openings(tg, shapes, support)?;
            Ok(Dispatched::NoClosing)
        }
        LEADING_LINE => {
            let mut spline_points = Vec::new();
            draw_splines(tg, &mut spline_points);
            if leading_line(tg, shapes, &spline_points)? {
                Ok(Dispatched::Splines(spline_points))
            } else {
                Ok(Dispatched::Finished)
            }
        }
        _ if is_spline_to_end(lt) => Ok(Dispatched::Splines(spline_shapes(tg, shapes, pt_last)?)),
        _ => Ok(Dispatched::Splines(Vec::new())),
    }
}

/// `GetMeTOCShape`: appends the shapes of `tg.line_type` to `shapes`. The
/// graphic's colours, style and width are set for the type first, so `tg`
/// changes as it does upstream (including `tg.pixels` for the ice
/// openings). On an error the shapes added so far stay, as upstream catches
/// and returns with them. The result is the pattern fill the host must
/// apply, for the types that use one.
pub(crate) fn get_me_toc_shape(
    tg: &mut Tg,
    shapes: &mut Vec<Shape>,
    settings: &Settings,
    support: &dyn MetocSupport,
) -> Result<Option<PatternFill>, EngineError> {
    let pt_last = tg.pixels.at(tg.pixels.len().wrapping_sub(1))?;
    set_metoc_properties(tg, settings);
    let spline_points = match dispatch(tg, shapes, pt_last, support)? {
        Dispatched::Finished => return Ok(None),
        Dispatched::NoClosing => Vec::new(),
        Dispatched::Splines(points) => points,
    };
    if let Some(last) = spline_points.last() {
        let mut path = GeneralPath::new();
        path.move_to(last.x, last.y);
        path.line_to(pt_last.x, pt_last.y)?;
        shapes.push(path.into_polyline(0));
    }
    Ok(set_shape_properties(tg, shapes))
}
