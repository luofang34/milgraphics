//! Port of `clsUtilityCPOF.Change1PixelsToShapes`: cuts `tg.pixels` into
//! shapes at the points whose style marks the end of a polyline.

use crate::engine::base::{EngineError, Shape, shape_type};
use crate::engine::tactical_lines::*;
use crate::engine::tg::Tg;

/// Builds the outline shape (or, with `fill`, one fill shape that goes to the
/// front of `shapes`) from `tg.pixels`. A point of style 5 or 10 ends a
/// polyline; the range-fan fill, route, track and cake types start a new
/// shape there.
pub(super) fn change1_pixels_to_shapes(
    tg: &Tg,
    shapes: &mut Vec<Shape>,
    fill: bool,
) -> Result<(), EngineError> {
    let kind = if fill {
        shape_type::FILL
    } else {
        shape_type::POLYLINE
    };
    let mut shape: Option<Shape> = None;
    let mut begin_line = true;
    let n = tg.pixels.len();
    let splits = matches!(tg.line_type, RANGE_FAN_FILL | BS_ROUTE | BS_TRACK | BS_CAKE);
    for (k, current) in tg.pixels.iter().enumerate() {
        let cur = shape.get_or_insert_with(|| Shape::new(kind));
        if begin_line {
            if k == 0 {
                cur.style = current.style;
            }
            if k > 0 {
                let last = tg.pixels.get(k - 1).copied().unwrap_or_default();
                // Doubled points with style 5.
                if current.style == 5 && last.style == 5 {
                    if cur.path.is_empty() {
                        return Err(EngineError::Degenerate("path has no initial moveto"));
                    }
                    cur.line_to(*current);
                }
            }
            cur.move_to(*current);
            begin_line = false;
        } else {
            cur.line_to(*current);
            if current.style == 5 || current.style == 10 {
                begin_line = true;
                if splits && k + 1 < n {
                    if let Some(done) = shape.take() {
                        shapes.push(done);
                    }
                    shape = Some(Shape::new(shape_type::POLYLINE));
                }
            }
        }
        if k + 1 == n {
            if let Some(done) = shape.take() {
                if done.shape_type == shape_type::FILL {
                    shapes.insert(0, done);
                } else {
                    shapes.push(done);
                }
            }
        }
    }
    Ok(())
}
