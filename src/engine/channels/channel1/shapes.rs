//! The last stage of `GetChannel1Double`: splits the point list at its
//! pen-up styles into shapes.

use super::Chan;
use crate::engine::base::{EngineError, Shape, shape_type};
use crate::engine::channels::point_index::at_i;
use crate::engine::tactical_lines as lt;
use crate::style::Rgba;

/// Line style marking a point that ends a polyline.
const PEN_UP: i32 = 5;

/// Appends the shapes for `ch.points` to `shapes`.
pub(super) fn points_to_shapes(ch: &Chan<'_>, shapes: &mut Vec<Shape>) -> Result<(), EngineError> {
    match ch.draw {
        lt::LC => line_of_contact_shapes(ch, shapes),
        lt::UNSP | lt::SFENCE | lt::DFENCE | lt::LWFENCE | lt::HWFENCE => fence_shapes(ch, shapes),
        lt::MOVEMENT_TO_CONTACT => contact_shapes(ch, shapes),
        _ => polyline_shapes(ch, shapes),
    }
}

/// One polyline per flot: a new shape starts after every pen-up, and the
/// enemy-side flot is drawn red.
fn line_of_contact_shapes(ch: &Chan<'_>, shapes: &mut Vec<Shape>) -> Result<(), EngineError> {
    let mut shape = Shape::new(shape_type::POLYLINE);
    let mut begin_path = true;
    for k in 0..ch.counter {
        let p = at_i(&ch.points, k)?;
        if begin_path {
            shape = Shape::new(shape_type::POLYLINE);
            shape.move_to(p);
            shape.style = p.style;
            if p.style == 25 {
                shape.line_color = Some(Rgba::RED);
            }
            begin_path = false;
        } else if k > 0 {
            shape.line_to(p);
            if p.style == PEN_UP {
                if !shape.path.is_empty() {
                    shapes.push(shape.clone());
                }
                begin_path = true;
            }
        } else {
            shape.move_to(p);
        }
    }
    Ok(())
}

/// Whether the point at `k` starts a new subpath after a doubled pen-up.
fn skips_doubled_pen_up(ch: &Chan<'_>, k: i32) -> Result<bool, EngineError> {
    if k == 0 {
        return Ok(false);
    }
    Ok(at_i(&ch.points, k)?.style == PEN_UP
        && at_i(&ch.points, k - 1)?.style == PEN_UP
        && k != ch.counter - 1)
}

fn push_if_drawn(shape: &Shape, shapes: &mut Vec<Shape>) {
    if !shape.path.is_empty() {
        shapes.push(shape.clone());
    }
}

/// Arrow types and plain channels: one shape with a subpath per pen-up run.
/// The arrow types skip doubled pen-ups; counterattacks are drawn with line
/// style 1.
fn polyline_shapes(ch: &Chan<'_>, shapes: &mut Vec<Shape>) -> Result<(), EngineError> {
    let doubled_pen_ups = matches!(
        ch.draw,
        lt::CATK
            | lt::CATKBYFIRE
            | lt::AAAAA
            | lt::SPT
            | lt::SPT_STRAIGHT
            | lt::AIRAOA
            | lt::FRONTAL_ATTACK
            | lt::TURNING_MOVEMENT
    );
    let mut shape = Shape::new(shape_type::POLYLINE);
    let mut begin_line = true;
    for k in 0..ch.counter {
        if matches!(ch.draw, lt::CATK | lt::CATKBYFIRE) {
            shape.style = 1;
        }
        let p = at_i(&ch.points, k)?;
        if begin_line {
            if doubled_pen_ups && skips_doubled_pen_up(ch, k)? {
                continue;
            }
            if !doubled_pen_ups && k == 0 {
                shape.style = p.style;
            }
            shape.move_to(p);
            begin_line = false;
        } else {
            shape.line_to(p);
            if p.style == PEN_UP {
                begin_line = true;
            }
        }
        if k == ch.counter - 1 {
            push_if_drawn(&shape, shapes);
        }
    }
    Ok(())
}

/// Movement to contact: style 9 starts the filled cover glyph, style 10 ends
/// a polyline that is drawn beneath the others.
fn contact_shapes(ch: &Chan<'_>, shapes: &mut Vec<Shape>) -> Result<(), EngineError> {
    let mut shape = Shape::new(shape_type::POLYLINE);
    let mut begin_line = true;
    for k in 0..ch.counter {
        let p = at_i(&ch.points, k)?;
        if begin_line {
            if skips_doubled_pen_up(ch, k)? {
                continue;
            }
            if p.style == 9 && at_i(&ch.points, k - 1)?.style != 9 {
                if !shape.path.is_empty() {
                    shapes.push(shape.clone());
                }
                shape = Shape::new(shape_type::FILL);
                shape.style = p.style;
                shape.fill_color = ch.req.tg.line_color;
                shape.fill_style = 1;
            }
            shape.move_to(p);
            begin_line = false;
        } else {
            shape.line_to(p);
            if p.style == PEN_UP {
                begin_line = true;
            } else if p.style == 10 {
                if !shape.path.is_empty() {
                    shapes.insert(0, shape.clone());
                    shape = Shape::new(shape_type::POLYLINE);
                }
                begin_line = true;
            }
        }
        if k == ch.counter - 1 {
            push_if_drawn(&shape, shapes);
        }
    }
    Ok(())
}

/// Fence types: a single shape whose subpaths follow the pen-up styles.
fn fence_shapes(ch: &Chan<'_>, shapes: &mut Vec<Shape>) -> Result<(), EngineError> {
    let mut shape = Shape::new(shape_type::POLYLINE);
    for k in 0..ch.counter {
        let p = at_i(&ch.points, k)?;
        if k == 0 {
            shape.move_to(p);
            if p.style == PEN_UP {
                continue;
            }
        }
        if k > 0 && k < ch.counter - 1 {
            let previous = at_i(&ch.points, k - 1)?;
            if previous.style == PEN_UP {
                shape.move_to(p);
            } else if previous.style == 0 {
                shape.line_to(p);
            }
            if p.style == PEN_UP {
                shape.move_to(p);
            }
            if k == ch.counter - 2 && p.style == 0 {
                shape.move_to(p);
                shape.line_to(at_i(&ch.points, k + 1)?);
            }
        }
        if k == ch.counter - 1 {
            push_if_drawn(&shape, shapes);
        }
    }
    Ok(())
}
