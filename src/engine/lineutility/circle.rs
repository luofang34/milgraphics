//! Port of `CalcCircleDouble` and `CalcCircleShape` from lineutility.java.

use crate::engine::base::{At, EngineError, Pt, Shape, shape_type};
use std::f64::consts::PI;

/// Upstream `CalcCircleDouble`: fills `circle_points[0..numpts]` with a closed
/// circle of `numpts` points (the last repeats the first). Every point has
/// style `styl`, except the closing point: 0 stays 0, 9 becomes 10, 11 becomes
/// 12 and anything else becomes 5.
pub(crate) fn calc_circle_double(
    center: Pt,
    radius: f64,
    numpts: i32,
    circle_points: &mut [Pt],
    styl: i32,
) -> Result<(), EngineError> {
    let segment_angle = 2.0 * PI / f64::from(numpts - 1);
    for j in 0..numpts - 1 {
        let x = center.x + (radius * (f64::from(j) * segment_angle).cos());
        let y = center.y + (radius * (f64::from(j) * segment_angle).sin());
        *circle_points.at_mut(index(j))? = Pt::styled(x, y, styl);
    }
    let mut last = circle_points.at(0)?;
    last.style = match styl {
        0 => 0,
        9 => 10,
        11 => 12,
        _ => 5,
    };
    *circle_points.at_mut(index(numpts - 1))? = last;
    Ok(())
}

fn index(i: i32) -> usize {
    usize::try_from(i).unwrap_or(usize::MAX)
}

/// Upstream `CalcCircleShape`: a fill (style 9) or polyline shape tracing the
/// circle built by [`calc_circle_double`].
pub(crate) fn calc_circle_shape(
    center: Pt,
    radius: f64,
    numpts: i32,
    circle_points: &mut [Pt],
    styl: i32,
) -> Result<Shape, EngineError> {
    let mut shape = Shape::new(if styl == 9 {
        shape_type::FILL
    } else {
        shape_type::POLYLINE
    });
    shape.style = styl;
    calc_circle_double(center, radius, numpts, circle_points, styl)?;
    shape.move_to(circle_points.at(0)?);
    for j in 1..numpts {
        shape.line_to(circle_points.at(index(j))?);
    }
    Ok(shape)
}
