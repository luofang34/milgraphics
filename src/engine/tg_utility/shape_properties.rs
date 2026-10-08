//! Port of the shape-property rules of mil-sym-java
//! JavaTacticalRenderer/clsUtility.java: `getLineStroke` and
//! `SetShapeProperties`.

use crate::engine::base::{Shape, Stroke, shape_type};
use crate::engine::tactical_lines::*;
use crate::engine::tg::Tg;
use crate::engine::tg_utility::shape_resolution::resolve_modifier_shape;

#[cfg(test)]
mod tests;

/// Upstream `getLineStroke`: the stroke for a line style (0 solid, 1 dashed,
/// 2 dotted, 3 dash-dot, 4 dash-dot-dot) with dash lengths scaled by the
/// width. Cap and join are not modelled; the drawn output only depends on
/// width and dash array.
pub(crate) fn get_line_stroke(width: i32, style: i32) -> Stroke {
    let w = f64::from(width);
    let dash_length = 2.0 * w;
    let dot_length = 1.0;
    let dot_space = 2.0 * w;
    let dash = match style {
        1 => Some(vec![dash_length, dash_length]),
        2 => Some(vec![dot_length, dot_space]),
        3 => Some(vec![2.0 * dash_length, dot_space, dot_length, dot_space]),
        4 => Some(vec![
            dash_length,
            dot_space,
            dot_length,
            dot_space,
            dot_length,
            dot_space,
        ]),
        _ => None,
    };
    Stroke { width: w, dash }
}

/// Upstream `SetShapeProperties`: drops the shapes a type should not show
/// without a fill, then resolves each shape's colours, style and stroke from
/// the graphic's properties. Skipped for METOC and MSR by the caller.
pub(crate) fn set_shape_properties(tg: &mut Tg, shapes: &mut Vec<Shape>) {
    // Upstream's whole body sits in one try block that swallows exceptions,
    // so a failure stops processing and keeps what was done so far.
    drop_unfilled_shapes(tg, shapes);
    let mut thickness = tg.line_thickness;
    for (j, shape) in shapes.iter_mut().enumerate() {
        if shape.path.is_empty() {
            continue;
        }
        if shape.shape_type == shape_type::FILL && tg.line_type != DEPTH_AREA {
            shape.fill_color = tg.fill_color;
        }
        // A failure inside is swallowed upstream as well.
        resolve_modifier_shape(tg, shape).ok();
        if tg.line_type == AIRFIELD && j == 1 {
            shape.fill_color = None;
        }
        if tg.line_type == BBS_POINT && j == 0 {
            shape.line_color = None;
        }
        if thickness == 0 {
            thickness = 1;
        }
        shape.stroke = if shape.shape_type == shape_type::FILL {
            Stroke {
                width: f64::from(thickness),
                dash: None,
            }
        } else {
            get_line_stroke(thickness, shape.style)
        };
    }
    solidify_arrowheads(tg, shapes, thickness);
}

/// Without a fill colour, some types keep only their last shape and some
/// lose their fill shapes.
fn drop_unfilled_shapes(tg: &Tg, shapes: &mut Vec<Shape>) {
    if tg.fill_color.is_some() {
        return;
    }
    match tg.line_type {
        AC | SAAFR | MRR | SL | TC | SC | LLTR => {
            if let Some(last) = shapes.pop() {
                shapes.clear();
                shapes.push(last);
            }
        }
        CATK | AIRAOA | AAAAA | SPT | FRONTAL_ATTACK | TURNING_MOVEMENT | MOVEMENT_TO_CONTACT
        | MAIN | CATKBYFIRE => {
            shapes.retain(|s| s.shape_type != shape_type::FILL);
        }
        _ => {}
    }
}

/// Arrowheads of some types are drawn solid even when the line is dashed.
fn solidify_arrowheads(tg: &Tg, shapes: &mut [Shape], thickness: i32) {
    let solid = get_line_stroke(thickness, 0);
    match tg.line_type {
        DIRATKAIR => {
            for shape in shapes.iter_mut().skip(2) {
                shape.style = 0;
                shape.stroke = solid.clone();
            }
        }
        DIRATKGND | DIRATKSPT | EXPLOIT | INFILTRATION => {
            if let Some(shape) = shapes.get_mut(1) {
                shape.style = 0;
                shape.stroke = solid;
            }
        }
        PDF => {
            if let Some(shape) = shapes.get_mut(1) {
                shape.style = 0;
                shape.stroke = solid;
                shape.fill_color = shape.line_color;
            }
        }
        _ => {}
    }
}
