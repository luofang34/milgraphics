//! Port of mil-sym-java JavaTacticalRenderer/clsUtility.ResolveModifierShape and
//! SetLCColor: per-type overrides of a shape's style and colours.

#[cfg(test)]
mod tests;

use crate::engine::base::{EngineError, Shape, shape_type};
use crate::engine::line_type::classes::is_change1_area;
use crate::engine::line_type::is_weather;
use crate::engine::tactical_lines::*;
use crate::engine::tg::{CAP_BUTT, Tg};
use crate::engine::tg_utility::line_classes::{is_closed_polygon, lines_with_fill};
use crate::style::Rgba;

/// Upstream `SetLCColor`: the red segments of a hostile or friendly LC keep
/// their colour only on the side the standard assigns to red.
pub(crate) fn set_lc_color(tg: &Tg, shape: &mut Shape) {
    let is_red = shape.line_color == Some(Rgba::RED);
    let keep_red = if tg.is_hostile() { !is_red } else { is_red };
    shape.line_color = if keep_red {
        Some(Rgba::RED)
    } else {
        tg.line_color
    };
}

/// The METOC line type of the graphic's symbol code, as `IsWeather` reads it.
fn symbol_weather_line_type(symbol_id: &str) -> Option<i32> {
    if symbol_id.len() < 20 {
        return None;
    }
    let set = symbol_id.get(4..6)?.parse::<u8>().ok()?;
    let entity = symbol_id.get(10..16)?.parse::<u32>().ok()?;
    is_weather(set, entity)
}

/// Upstream `ResolveModifierShape`: sets `shape`'s style, line colour and
/// fill from the graphic's properties, with per-type exceptions. METOC
/// shapes keep what the METOC code gave them.
pub(crate) fn resolve_modifier_shape(tg: &mut Tg, shape: &mut Shape) -> Result<(), EngineError> {
    if symbol_weather_line_type(&tg.symbol_id).is_some_and(|t| t > 0) {
        return Ok(());
    }
    let polyline = shape.shape_type == shape_type::POLYLINE;
    let fill = shape.shape_type == shape_type::FILL;
    match tg.line_type {
        NFA | NFA_CIRCULAR | NFA_RECTANGULAR | BIO | BIOT | NUC | CHEM | CHEMT | RAD | RADT
        | WFZ_REVD | WFZ => {
            hatched_outline(tg, shape, if tg.use_hatch_fill { 0 } else { 3 });
        }
        OBSAREA => hatched_outline(tg, shape, 0),
        LAA => hatched_outline(tg, shape, if tg.use_hatch_fill { 0 } else { 2 }),
        DIRATKAIR | ATDITCHC | ATDITCHM | SARA | FOLSP | FERRY | MNFLDFIX | TURN_REVD | TURN
        | MNFLDDIS | EASY | BYDIF | BYIMP | MOBILE_DEFENSE => {
            tg.line_cap = CAP_BUTT;
            if fill {
                shape.fill_style = 1;
                shape.fill_color = tg.line_color;
            }
            if polyline {
                shape.style = tg.line_style;
                shape.line_color = tg.line_color;
            }
        }
        DECEIVE | CLUSTER | CATK | CATKBYFIRE | PLD | PLANNED | CFL | FORDSITE | ACOUSTIC_AMB => {
            if polyline {
                shape.style = 1;
                shape.line_color = tg.line_color;
            }
        }
        PNO => {
            if polyline {
                shape.style = 1;
                shape.line_color = tg.line_color;
                shape.fill_color = tg.fill_color;
                shape.fill_style = tg.fill_style;
            }
        }
        FOLLA | ESR1 | FORDIF => {
            if polyline {
                shape.line_color = tg.line_color;
                if shape.style != tg.line_style && shape.style != 1 {
                    shape.style = tg.line_style;
                }
            }
        }
        AREA_DEFENSE => area_defense(tg, shape)?,
        MOVEMENT_TO_CONTACT => movement_to_contact(tg, shape)?,
        EXPLOIT => {
            if fill {
                shape.fill_style = tg.fill_style;
                shape.fill_color = tg.fill_color;
            }
            if polyline {
                shape.line_color = tg.line_color;
                if shape.style != 1 {
                    shape.style = tg.line_style;
                }
            }
        }
        _ => default_shape(tg, shape),
    }
    Ok(())
}

/// Outline shapes of hatched areas take the graphic's line and fill.
fn hatched_outline(tg: &Tg, shape: &mut Shape, fill_style: i32) {
    if shape.shape_type == shape_type::POLYLINE {
        shape.style = tg.line_style;
        shape.line_color = tg.line_color;
        shape.fill_style = fill_style;
        shape.fill_color = tg.fill_color;
    }
}

/// Whether the shape's first and last points coincide and it has `count`
/// points, which marks an arrowhead drawn as a closed fill.
fn is_closed_head(shape: &Shape, count: usize) -> Result<bool, EngineError> {
    let points = shape.points();
    let first = *points
        .first()
        .ok_or(EngineError::Index { index: 0, len: 0 })?;
    let last = *points
        .last()
        .ok_or(EngineError::Index { index: 0, len: 0 })?;
    Ok(points.len() == count && first.x == last.x && first.y == last.y)
}

fn area_defense(tg: &Tg, shape: &mut Shape) -> Result<(), EngineError> {
    if shape.shape_type == shape_type::FILL {
        shape.fill_style = tg.fill_style;
        shape.fill_color = tg.fill_color;
        // A closed five-point shape is the triangle, filled with the line colour.
        if is_closed_head(shape, 5)? {
            shape.fill_style = 1;
            shape.fill_color = tg.line_color;
        }
    }
    if shape.shape_type == shape_type::POLYLINE {
        shape.line_color = tg.line_color;
        shape.style = tg.line_style;
        if lines_with_fill(tg.line_type)
            || is_closed_polygon(tg.line_type)
            || is_change1_area(tg.line_type)
        {
            shape.fill_style = tg.fill_style;
            shape.fill_color = tg.fill_color;
        }
    }
    Ok(())
}

fn movement_to_contact(tg: &Tg, shape: &mut Shape) -> Result<(), EngineError> {
    if shape.shape_type == shape_type::FILL {
        shape.fill_style = tg.fill_style;
        shape.fill_color = tg.fill_color;
        // A closed four-point shape is the arrow at the end of the jagged
        // line, filled with the line colour.
        if is_closed_head(shape, 4)? {
            shape.fill_color = tg.line_color;
        }
    }
    if shape.shape_type == shape_type::POLYLINE {
        shape.line_color = tg.line_color;
        shape.style = tg.line_style;
    }
    Ok(())
}

fn default_shape(tg: &Tg, shape: &mut Shape) {
    let lt = tg.line_type;
    if shape.shape_type == shape_type::FILL {
        shape.fill_style = tg.fill_style;
        shape.fill_color = tg.fill_color;
    }
    if shape.shape_type != shape_type::POLYLINE {
        return;
    }
    if lt == LC {
        set_lc_color(tg, shape);
    } else {
        shape.line_color = tg.line_color;
    }
    shape.style = tg.line_style;
    if lines_with_fill(lt) || is_closed_polygon(lt) || is_change1_area(lt) {
        match lt {
            RANGE_FAN | RANGE_FAN_SECTOR | RADAR_SEARCH | BBS_AREA | BBS_RECTANGLE => {
                shape.fill_color = None;
            }
            _ => {
                shape.fill_style = tg.fill_style;
                shape.fill_color = tg.fill_color;
            }
        }
    }
    match lt {
        BS_ELLIPSE | BS_RECTANGLE => {
            shape.fill_style = tg.fill_style;
            shape.fill_color = tg.fill_color;
        }
        BBS_RECTANGLE | PBS_RECTANGLE | PBS_SQUARE => shape.fill_color = None,
        _ => {}
    }
}
