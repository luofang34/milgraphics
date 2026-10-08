//! Port of clsMETOC.SetShapeProperties: gives the shapes built for a METOC
//! type its colours, line style and stroke.

use super::PatternFill;
use crate::engine::base::{At, EngineError, Shape, Stroke, shape_type};
use crate::engine::lineutility::basics::calc_distance_double;
use crate::engine::tactical_lines::*;
use crate::engine::tg::Tg;
use crate::engine::tg_utility::line_classes::is_closed_polygon;
use crate::engine::tg_utility::shape_properties::get_line_stroke;

/// Types whose fill is a raster pattern upstream.
fn has_pattern_fill(line_type: i32) -> bool {
    matches!(
        line_type,
        FISH_TRAPS
            | SWEPT_AREA
            | OIL_RIG_FIELD
            | FOUL_GROUND
            | KELP
            | BEACH_SLOPE_MODERATE
            | BEACH_SLOPE_STEEP
    )
}

fn is_surface_front(line_type: i32) -> bool {
    matches!(line_type, SF | USF | SFG | SFY)
}

/// Areas that take the graphic's fill colour rather than a pattern.
fn takes_solid_fill(line_type: i32) -> bool {
    matches!(
        line_type,
        FORESHORE_AREA
            | WATER
            | BEACH
            | ISLAND
            | DRYDOCK
            | LOADING_FACILITY_AREA
            | PERCHES
            | UNDERWATER_HAZARD
            | DISCOLORED_WATER
            | SUBMERGED_CRIB
            | FREEFORM
    ) || is_bottom_or_vdr(line_type)
}

fn is_bottom_or_vdr(line_type: i32) -> bool {
    matches!(
        line_type,
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
            | FINE_SAND
            | MEDIUM_SAND
            | COARSE_SAND
            | VERY_COARSE_SAND
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
            | BOTTOM_ROUGHNESS_MODERATE
            | BOTTOM_ROUGHNESS_ROUGH
            | BOTTOM_ROUGHNESS_SMOOTH
            | CLUTTER_HIGH
            | CLUTTER_MEDIUM
            | CLUTTER_LOW
            | IMPACT_BURIAL_0
            | IMPACT_BURIAL_10
            | IMPACT_BURIAL_100
            | IMPACT_BURIAL_20
            | IMPACT_BURIAL_75
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

/// Length of the first six segments of an arc, which sizes the dashes of
/// the types drawn as a chain of arcs.
fn arc_length(shape: &Shape) -> Result<f32, EngineError> {
    let points = shape.points();
    let mut length = 0.0_f32;
    for i in 0..6 {
        length += calc_distance_double(points.at(i)?, points.at(i + 1)?) as f32;
    }
    Ok(length)
}

/// Instability and shear: dots sit on the peak of each arc. Float
/// arithmetic follows upstream.
fn peak_dotted_stroke(
    shape: &Shape,
    line_type: i32,
    thickness: i32,
) -> Result<Stroke, EngineError> {
    let dot_length = 1.0_f32;
    let arc = arc_length(shape)?;
    let spacing = (thickness as f32 * 2.0).min(arc / 5.0);
    let dash: Vec<f32> = if line_type == INSTABILITY {
        let dash_length = (arc - (dot_length * 2.0 + spacing * 3.0)) / 2.0;
        vec![
            dash_length,
            spacing,
            dot_length,
            spacing,
            dot_length,
            spacing,
            dash_length,
            0.0,
        ]
    } else {
        let dash_length = (arc - (dot_length + spacing * 2.0)) / 2.0;
        vec![dash_length, spacing, dot_length, spacing, dash_length, 0.0]
    };
    stroke_with(thickness, &dash)
}

/// A stroke with a custom dash array; `BasicStroke` rejects negative
/// lengths and arrays without a positive one, and so does this.
fn stroke_with(thickness: i32, dash: &[f32]) -> Result<Stroke, EngineError> {
    if dash.iter().any(|d| *d < 0.0) || !dash.iter().any(|d| *d > 0.0) {
        return Err(EngineError::Degenerate("dash array is not drawable"));
    }
    Ok(Stroke {
        width: f64::from(thickness),
        dash: Some(dash.iter().map(|d| f64::from(*d)).collect()),
    })
}

/// Trough: dashes no longer than a quarter of the arc.
fn trough_stroke(shape: &Shape, thickness: i32) -> Result<Stroke, EngineError> {
    let arc = arc_length(shape)?;
    let dash_length = (2.0 * thickness as f32).min(arc / 4.0);
    stroke_with(thickness, &[dash_length, dash_length])
}

fn stroke_for(shape: &Shape, tg: &Tg) -> Result<Stroke, EngineError> {
    match tg.line_type {
        INSTABILITY | SHEAR => peak_dotted_stroke(shape, tg.line_type, tg.line_thickness),
        TROUGH => trough_stroke(shape, tg.line_thickness),
        _ => Ok(get_line_stroke(tg.line_thickness, shape.style)),
    }
}

/// The colours a shape takes from the graphic before its stroke is set.
fn apply_colours(shape: &mut Shape, tg: &Tg, closed: bool) {
    if shape.shape_type == shape_type::FILL {
        shape.fill_color = tg.fill_color;
    }
    match tg.line_type {
        SF | USF | SFG | SFY | ITD => {}
        LEADING_LINE | TRAINING_AREA => shape.line_color = tg.line_color,
        _ => {
            shape.line_color = tg.line_color;
            shape.style = tg.line_style;
        }
    }
    if (closed || shape.shape_type == shape_type::FILL) && takes_solid_fill(tg.line_type) {
        shape.fill_color = tg.fill_color;
    }
}

fn shape_loop(tg: &Tg, shapes: &mut [Shape]) -> Result<(), EngineError> {
    let closed = is_closed_polygon(tg.line_type);
    for shape in shapes.iter_mut().filter(|s| !s.path.is_empty()) {
        apply_colours(shape, tg, closed);
        shape.stroke = stroke_for(shape, tg)?;
    }
    Ok(())
}

fn surface_front_strokes(tg: &Tg, shapes: &mut [Shape]) {
    for shape in shapes.iter_mut().filter(|s| !s.path.is_empty()) {
        shape.style = tg.line_style;
        shape.stroke = get_line_stroke(tg.line_thickness, shape.style);
    }
}

/// `SetShapeProperties`. Upstream swallows a failure and keeps the shapes
/// as far as they were set; so does this. The pattern fill, if the type has
/// one, is returned for the host to apply.
pub(crate) fn set_shape_properties(tg: &Tg, shapes: &mut [Shape]) -> Option<PatternFill> {
    if tg.line_type == DEPTH_AREA {
        return None;
    }
    let mut pattern = None;
    if has_pattern_fill(tg.line_type) {
        let first = shapes.first_mut()?;
        first.line_color = tg.line_color;
        pattern = Some(PatternFill {
            shape_index: 0,
            line_type: tg.line_type,
        });
    } else if is_surface_front(tg.line_type) {
        surface_front_strokes(tg, shapes);
        return None;
    }
    shape_loop(tg, shapes).ok();
    pattern
}
