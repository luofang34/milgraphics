//! The shape-building switches of `arraysupport.GetLineArray2Double`: the
//! outline shapes of each line type, then the separate fill shapes of the
//! arrowhead types. Shapes are never dropped for being empty.

mod fills;
mod flot_shapes;
mod outlines;

use super::shape_path::{line_to, move_to};
use super::work::{Work, get};
use crate::engine::base::{EngineError, Pt, Shape, shape_type};

/// Builds the shapes of the line type from the finished points.
pub(crate) fn build(w: &mut Work<'_>, shapes: &mut Vec<Shape>) -> Result<(), EngineError> {
    outlines::build(w, shapes)?;
    fills::build(w, shapes)
}

/// Upstream `addPolyline`: one polyline shape through `pts[..count]`,
/// starting a new run after each style-5 or style-10 point and skipping
/// doubled end points.
pub(crate) fn add_polyline(
    pts: &[Pt],
    count: i32,
    shapes: &mut Vec<Shape>,
) -> Result<(), EngineError> {
    let mut shape: Option<Shape> = None;
    let mut begin_line = true;
    for k in 0..count {
        let cur = get(pts, k)?;
        let sh = shape.get_or_insert_with(|| Shape::new(shape_type::POLYLINE));
        if begin_line {
            if k == 0 {
                sh.style = cur.style;
            }
            if k > 0 {
                let prev = get(pts, k - 1)?;
                let doubled = prev.style == 5 && k < count - 1;
                if cur.style == 5 && (doubled || prev.style == 10) {
                    continue;
                }
            }
            if k == 0 && pts.len() > 1 && cur.style == 5 && get(pts, 1)?.style == 5 {
                continue;
            }
            move_to(sh, cur);
            begin_line = false;
        } else {
            line_to(sh, cur)?;
            if cur.style == 5 || cur.style == 10 {
                begin_line = true;
            }
        }
        if k == count - 1 {
            if let Some(done) = shape.take() {
                shapes.push(done);
            }
        }
    }
    Ok(())
}

/// A polyline of `moveTo`/`lineTo` pairs, the common two-point shape.
pub(crate) fn segment(style: i32, a: Pt, b: Pt) -> Result<Shape, EngineError> {
    let mut s = Shape::new(shape_type::POLYLINE);
    s.style = style;
    move_to(&mut s, a);
    line_to(&mut s, b)?;
    Ok(s)
}

#[cfg(test)]
mod tests;
