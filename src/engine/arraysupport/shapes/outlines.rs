//! The outline switch: the shapes drawn from the finished point array.

use super::flot_shapes;
use super::{add_polyline, segment};
use crate::engine::arraysupport::shape_path::{line_to, move_to};
use crate::engine::arraysupport::work::{Work, get};
use crate::engine::base::{EngineError, Pt, Shape, shape_type};
use crate::engine::tactical_lines as lt;

/// Appends the outline shapes of `w.line_type`.
pub(crate) fn build(w: &mut Work<'_>, shapes: &mut Vec<Shape>) -> Result<(), EngineError> {
    match w.line_type {
        lt::PDF => pdf(w, shapes),
        lt::BBS_AREA | lt::BBS_RECTANGLE => bbs(w, shapes),
        lt::DIRATKGND => dir_atk_gnd(w, shapes),
        lt::DEPTH_AREA => super::depth_area::build(w, shapes),
        lt::TRAINING_AREA => training_area(w, shapes),
        lt::ITD | lt::SFY | lt::SFG | lt::USF | lt::SF | lt::WFG | lt::CFG | lt::PIPE => {
            flot_shapes::build(w, shapes)
        }
        lt::ATDITCHM => flot_shapes::atditchm(w, shapes),
        lt::FOLLA => folla(w, shapes),
        lt::ESR1 => esr1(w, shapes),
        lt::FORDIF => fordif(w, shapes),
        lt::FENCED => fenced(w, shapes),
        lt::AIRFIELD => airfield(w, shapes),
        lt::STRIKWARN => strikwarn(w, shapes),
        lt::DIRATKAIR | lt::DIRATKSPT | lt::EXPLOIT => tail_polylines(w, shapes),
        _ => add_polyline(&w.p, w.ac, shapes),
    }
}

/// `p[from..from + count]` as the `secondPoly` arrays upstream copies.
fn sub(p: &[Pt], from: i32, count: i32) -> Result<Vec<Pt>, EngineError> {
    (0..count).map(|i| get(p, from + i)).collect()
}

fn pdf(w: &Work<'_>, shapes: &mut Vec<Shape>) -> Result<(), EngineError> {
    add_polyline(&w.p, 3, shapes)?;
    let mut s = Shape::new(shape_type::POLYLINE);
    move_to(&mut s, get(&w.p, 3)?);
    for k in 4..8 {
        line_to(&mut s, get(&w.p, k)?)?;
    }
    shapes.push(s);
    add_polyline(&sub(&w.p, 8, 6)?, 6, shapes)
}

fn bbs(w: &Work<'_>, shapes: &mut Vec<Shape>) -> Result<(), EngineError> {
    let mut s = Shape::new(shape_type::FILL);
    move_to(&mut s, get(&w.p, 0)?);
    for j in 0..w.save {
        line_to(&mut s, get(&w.p, j)?)?;
    }
    shapes.push(s);
    let mut s = Shape::new(shape_type::POLYLINE);
    move_to(&mut s, get(&w.orig, 0)?);
    for j in 1..w.save {
        line_to(&mut s, get(&w.orig, j)?)?;
    }
    shapes.push(s);
    Ok(())
}

/// The line, then the arrow, which the renderer strokes thinner.
fn dir_atk_gnd(w: &Work<'_>, shapes: &mut Vec<Shape>) -> Result<(), EngineError> {
    let ac = w.ac;
    let mut s = Shape::new(shape_type::POLYLINE);
    move_to(&mut s, get(&w.p, 0)?);
    for j in 0..ac - 10 {
        line_to(&mut s, get(&w.p, j)?)?;
    }
    shapes.push(s);
    let mut s = Shape::new(shape_type::POLYLINE);
    move_to(&mut s, get(&w.p, ac - 10)?);
    for j in (1..=9).rev() {
        if get(&w.p, ac - j - 1)?.style == 5 {
            move_to(&mut s, get(&w.p, ac - j)?);
        } else {
            line_to(&mut s, get(&w.p, ac - j)?)?;
        }
    }
    shapes.push(s);
    Ok(())
}

/// The red outline, then the blue symbol drawn from the added points.
fn training_area(w: &Work<'_>, shapes: &mut Vec<Shape>) -> Result<(), EngineError> {
    let mut red = Shape::new(shape_type::POLYLINE);
    red.style = 1;
    let mut blue = Shape::new(shape_type::POLYLINE);
    blue.style = 0;
    move_to(&mut red, get(&w.p, 0)?);
    for k in 1..w.save {
        line_to(&mut red, get(&w.p, k)?)?;
    }
    let mut begin = true;
    for k in w.save..w.ac {
        let pk = get(&w.p, k)?;
        if pk.style == 0 {
            if begin {
                move_to(&mut blue, pk);
                begin = false;
            } else {
                line_to(&mut blue, pk)?;
            }
        }
        if pk.style == 5 {
            line_to(&mut blue, pk)?;
            begin = true;
        }
    }
    shapes.push(red);
    shapes.push(blue);
    Ok(())
}

fn folla(w: &Work<'_>, shapes: &mut Vec<Shape>) -> Result<(), EngineError> {
    shapes.push(segment(1, get(&w.p, 0)?, get(&w.p, 1)?)?);
    let mut s = Shape::new(shape_type::POLYLINE);
    s.style = 0;
    for j in 2..w.vbl {
        if get(&w.p, j - 1)?.style != 5 {
            line_to(&mut s, get(&w.p, j)?)?;
        } else {
            move_to(&mut s, get(&w.p, j)?);
        }
    }
    shapes.push(s);
    Ok(())
}

fn esr1(w: &Work<'_>, shapes: &mut Vec<Shape>) -> Result<(), EngineError> {
    shapes.push(segment(get(&w.p, 0)?.style, get(&w.p, 0)?, get(&w.p, 1)?)?);
    shapes.push(segment(get(&w.p, 2)?.style, get(&w.p, 2)?, get(&w.p, 3)?)?);
    Ok(())
}

fn fordif(w: &Work<'_>, shapes: &mut Vec<Shape>) -> Result<(), EngineError> {
    let mut s = Shape::new(shape_type::POLYLINE);
    s.style = get(&w.p, 0)?.style;
    move_to(&mut s, get(&w.p, 0)?);
    line_to(&mut s, get(&w.p, 1)?)?;
    move_to(&mut s, get(&w.p, 2)?);
    line_to(&mut s, get(&w.p, 3)?)?;
    shapes.push(s);
    let mut s = Shape::new(shape_type::POLYLINE);
    s.style = get(&w.p, 4)?.style;
    move_to(&mut s, get(&w.p, 4)?);
    for k in 5..w.ac {
        if get(&w.p, k - 1)?.style != 5 {
            line_to(&mut s, get(&w.p, k)?)?;
        }
    }
    shapes.push(s);
    Ok(())
}

/// The control points as one shape, then the X marks (`w.points` past the
/// control points).
fn fenced(w: &Work<'_>, shapes: &mut Vec<Shape>) -> Result<(), EngineError> {
    let pts = &w.points;
    let mut s = Shape::new(shape_type::POLYLINE);
    s.style = get(pts, 0)?.style;
    move_to(&mut s, get(pts, 0)?);
    for k in 1..w.vbl {
        line_to(&mut s, get(pts, k)?)?;
    }
    shapes.push(s);
    let mut s = Shape::new(shape_type::POLYLINE);
    let mut begin = true;
    let total = i32::try_from(pts.len()).map_err(|_| EngineError::Degenerate("too many points"))?;
    for k in w.vbl..total {
        let pk = get(pts, k)?;
        if begin {
            if k > 0 && pk.style == 5 && get(pts, k - 1)?.style == 5 {
                line_to(&mut s, pk)?;
            }
            move_to(&mut s, pk);
            begin = false;
        } else {
            line_to(&mut s, pk)?;
            if pk.style == 5 || pk.style == 10 {
                begin = true;
            }
        }
        if k == total - 1 {
            shapes.push(s.clone());
        }
    }
    Ok(())
}

fn airfield(w: &Work<'_>, shapes: &mut Vec<Shape>) -> Result<(), EngineError> {
    let ac = w.ac;
    let mut s = Shape::new(shape_type::POLYLINE);
    move_to(&mut s, get(&w.p, 0)?);
    for k in 1..ac - 5 {
        line_to(&mut s, get(&w.p, k)?)?;
    }
    shapes.push(s);
    let mut s = Shape::new(shape_type::POLYLINE);
    move_to(&mut s, get(&w.p, ac - 4)?);
    line_to(&mut s, get(&w.p, ac - 3)?)?;
    move_to(&mut s, get(&w.p, ac - 2)?);
    line_to(&mut s, get(&w.p, ac - 1)?)?;
    shapes.push(s);
    Ok(())
}

fn strikwarn(w: &Work<'_>, shapes: &mut Vec<Shape>) -> Result<(), EngineError> {
    let half = w.ac / 2;
    add_polyline(&w.p, half, shapes)?;
    add_polyline(&sub(&w.p, half, half)?, half, shapes)
}

/// Line types whose arrow or tail sits in the last points of the array.
fn tail_polylines(w: &Work<'_>, shapes: &mut Vec<Shape>) -> Result<(), EngineError> {
    let len = i32::try_from(w.p.len()).map_err(|_| EngineError::Degenerate("too many points"))?;
    match w.line_type {
        lt::DIRATKAIR => {
            add_polyline(&sub(&w.p, len - 4, 4)?, 4, shapes)?;
            add_polyline(&w.p, w.ac - 13, shapes)?;
            add_polyline(&sub(&w.p, len - 13, 9)?, 9, shapes)
        }
        lt::DIRATKSPT => {
            add_polyline(&w.p, w.ac - 3, shapes)?;
            add_polyline(&sub(&w.p, len - 3, 3)?, 3, shapes)
        }
        _ => {
            add_polyline(&w.p, 2, shapes)?;
            add_polyline(&sub(&w.p, 2, 3)?, 3, shapes)?;
            add_polyline(&sub(&w.p, 5, 3)?, 3, shapes)
        }
    }
}
