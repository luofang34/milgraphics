//! Port of the line-extension helpers of lineutility.java: points along,
//! beyond, above, below and beside a line.

use super::basics::calc_distance_double;
use super::slope::{calc_true_slope_double, reverse_direction};
use super::{EXTEND_ABOVE, EXTEND_BELOW, EXTEND_LEFT, EXTEND_RIGHT};
use crate::engine::base::Pt;

/// Upstream `ExtendLineDouble`: the point `dist` pixels beyond `pt2` on the
/// line from `pt1`; returns `pt2` when either length is zero.
pub(crate) fn extend_line_double(pt1: Pt, pt2: Pt, dist: f64) -> Pt {
    let d = calc_distance_double(pt1, pt2);
    if d == 0.0 || dist == 0.0 {
        return pt2;
    }
    Pt::new(
        (d + dist) / d * (pt2.x - pt1.x) + pt1.x,
        (d + dist) / d * (pt2.y - pt1.y) + pt1.y,
    )
}

/// Upstream `ExtendAlongLineDouble`: the point `dist` pixels from `pt1`
/// toward `pt2`; returns `pt2` when either length is zero.
pub(crate) fn extend_along_line_double(pt1: Pt, pt2: Pt, dist: f64) -> Pt {
    let d = calc_distance_double(pt1, pt2);
    if d == 0.0 || dist == 0.0 {
        return pt2;
    }
    Pt::new(
        (dist / d) * (pt2.x - pt1.x) + pt1.x,
        (dist / d) * (pt2.y - pt1.y) + pt1.y,
    )
}

/// Upstream `ExtendAlongLineDouble2`: as [`extend_along_line_double`] but a
/// zero length returns `pt1`.
pub(crate) fn extend_along_line_double2(pt1: Pt, pt2: Pt, dist: f64) -> Pt {
    let d = calc_distance_double(pt1, pt2);
    if d == 0.0 || dist == 0.0 {
        return pt1;
    }
    Pt::new(
        dist / d * (pt2.x - pt1.x) + pt1.x,
        dist / d * (pt2.y - pt1.y) + pt1.y,
    )
}

/// Upstream `ExtendAlongLineDouble(pt1, pt2, dist, styl)`: the extended point
/// carries `styl`; a zero length returns `pt2` unchanged.
pub(crate) fn extend_along_line_double_style(pt1: Pt, pt2: Pt, dist: f64, styl: i32) -> Pt {
    let d = calc_distance_double(pt1, pt2);
    if d == 0.0 || dist == 0.0 {
        return pt2;
    }
    Pt::styled(
        dist / d * (pt2.x - pt1.x) + pt1.x,
        dist / d * (pt2.y - pt1.y) + pt1.y,
        styl,
    )
}

/// Upstream `ExtendLineAbove`: `(status, x, y)` of the point `d` pixels
/// above (`direction` 0) or below the line at `pt3`. Status 0 for a vertical
/// line, with x and y zero.
pub(crate) fn extend_line_above(
    pt1: Pt,
    pt2: Pt,
    pt3: Pt,
    d: f64,
    direction: i32,
) -> (i32, f64, f64) {
    let (vertical, m) = calc_true_slope_double(pt1, pt2);
    if vertical == 0 {
        return (0, 0.0, 0.0);
    }
    if m == 0.0 {
        let y = if direction == 0 {
            pt3.y - d.abs()
        } else {
            pt3.y + d.abs()
        };
        return (1, pt3.x, y);
    }
    let mag = (d / (m * (1.0 + 1.0 / (m * m)).sqrt())).abs();
    let dy = if direction == 0 { -mag } else { mag };
    let dx = -m * dy;
    (1, pt3.x + dx, pt3.y + dy)
}

/// Upstream `ExtendLineLeft`: `(status, x, y)` of the point `d` pixels left
/// (`direction` 0) or right of the line at `pt3`. Status 0 for a horizontal
/// line, with x and y zero.
pub(crate) fn extend_line_left(
    pt1: Pt,
    pt2: Pt,
    pt3: Pt,
    d: f64,
    direction: i32,
) -> (i32, f64, f64) {
    let (vertical, m) = calc_true_slope_double(pt1, pt2);
    if vertical != 0 && m == 0.0 {
        return (0, 0.0, 0.0);
    }
    if vertical == 0 {
        let x = if direction == 0 {
            pt3.x - d.abs()
        } else {
            pt3.x + d.abs()
        };
        return (1, x, pt3.y);
    }
    let mag = (d / (1.0 + 1.0 / (m * m)).sqrt()).abs();
    let dx = if direction == 0 { -mag } else { mag };
    let dy = -(1.0 / m) * dx;
    (1, pt3.x + dx, pt3.y + dy)
}

/// The extension point for a direction code, or `None` for a code that is
/// not 0-3 (where upstream fails and keeps the original point).
fn extend_by_direction(pt1: Pt, pt2: Pt, pt0: Pt, direction: i32, d: f64) -> Option<(f64, f64)> {
    let (_, x, y) = match direction {
        EXTEND_LEFT => extend_line_left(pt1, pt2, pt0, d, 0),
        EXTEND_RIGHT => extend_line_left(pt1, pt2, pt0, d, 1),
        EXTEND_ABOVE => extend_line_above(pt1, pt2, pt0, d, 0),
        EXTEND_BELOW => extend_line_above(pt1, pt2, pt0, d, 1),
        _ => return None,
    };
    Some((x, y))
}

/// Upstream `ExtendDirectedLine(pt1, pt2, pt0, direction, d)`: the point `d`
/// pixels from `pt0` perpendicular to the line (0 left, 1 right, 2 above,
/// 3 below). Other codes return `pt0`.
pub(crate) fn extend_directed_line(pt1: Pt, pt2: Pt, pt0: Pt, direction: i32, d: f64) -> Pt {
    let mut r = pt0;
    if let Some((x, y)) = extend_by_direction(pt1, pt2, pt0, direction, d) {
        r.x = x;
        r.y = y;
    }
    r
}

/// Upstream `ExtendDirectedLineText`: like [`extend_directed_line`] for text
/// offsets. A negative distance flips the direction, and left/right become
/// above/below on horizontal segments (above/below become left/right on
/// vertical ones).
pub(crate) fn extend_directed_line_text(pt1: Pt, pt2: Pt, pt0: Pt, direction: i32, d: f64) -> Pt {
    let (mut direction, mut d) = (direction, d);
    if d < 0.0 {
        direction = reverse_direction(direction);
        d = d.abs();
    }
    if pt1.y == pt2.y {
        direction = match direction {
            0 => EXTEND_ABOVE,
            1 => EXTEND_BELOW,
            o => o,
        };
    }
    if pt1.x == pt2.x {
        direction = match direction {
            2 => EXTEND_LEFT,
            3 => EXTEND_RIGHT,
            o => o,
        };
    }
    extend_directed_line(pt1, pt2, pt0, direction, d)
}

/// Upstream `ExtendDirectedLine(pt1, pt2, pt0, direction, d, style)`: as
/// [`extend_directed_line`] with axis-aligned segments mapped to the
/// perpendicular direction and `style` set on the result. Codes other than
/// 0-3 return `pt0` unchanged, including its style.
pub(crate) fn extend_directed_line_style(
    pt1: Pt,
    pt2: Pt,
    pt0: Pt,
    direction: i32,
    d: f64,
    style: i32,
) -> Pt {
    let mut direction = direction;
    if pt1.x == pt2.x {
        direction = match direction {
            2 => 0,
            3 => 1,
            o => o,
        };
    }
    if pt1.y == pt2.y {
        direction = match direction {
            0 => 2,
            1 => 3,
            o => o,
        };
    }
    let mut r = pt0;
    if let Some((x, y)) = extend_by_direction(pt1, pt2, pt0, direction, d) {
        r.x = x;
        r.y = y;
        r.style = style;
    }
    r
}

/// Upstream `ExtendLine2Double`: the point `dist` pixels beyond `pt2` on the
/// line from `pt1`, carrying `styl`; `pt2` itself when the line has no length.
pub(crate) fn extend_line2_double(pt1: Pt, pt2: Pt, dist: f64, styl: i32) -> Pt {
    let d = calc_distance_double(pt1, pt2);
    let mut r = Pt::new(pt2.x, pt2.y);
    if d > 0.0 {
        r.x = (d + dist) / d * (pt2.x - pt1.x) + pt1.x;
        r.y = (d + dist) / d * (pt2.y - pt1.y) + pt1.y;
        r.style = styl;
    }
    r
}

/// Upstream `ExtendAngledLine`: the point `d` pixels from `pt2` at `alpha`
/// degrees from the pt0-pt1 direction.
pub(crate) fn extend_angled_line(pt0: Pt, pt1: Pt, pt2: Pt, alpha: f64, d: f64) -> Pt {
    let psi = ((pt1.y - pt0.y) / (pt1.x - pt0.x)).atan();
    let theta = psi + std::f64::consts::PI * alpha / 180.0;
    Pt::new(pt2.x + d * theta.cos(), pt2.y + d * theta.sin())
}

/// Upstream `ExtendTrueLinePerpDouble`: the point `d` pixels from `pt2` on
/// the line through `pt2` perpendicular to pt0-pt1, carrying `styl`.
pub(crate) fn extend_true_line_perp_double(
    pt0: Pt,
    pt1: Pt,
    pt2: Pt,
    d: f64,
    styl: i32,
) -> Result<Pt, crate::engine::base::EngineError> {
    use super::basics::get_pixels_min;
    let mut result = pt0;
    let mut y_intercept = pt0;
    let (mut offset_x, _) = get_pixels_min(&[pt0, pt1, pt2], 3)?;
    if offset_x <= 0.0 {
        offset_x -= 100.0;
    } else {
        offset_x = 0.0;
    }
    let (n_temp, m) = calc_true_slope_double(pt0, pt1);
    if n_temp == 0 {
        result.x = if pt0.y < pt1.y { pt2.x - d } else { pt2.x + d };
        result.y = pt2.y;
    } else if m == 0.0 {
        result.x = pt2.x;
        result.y = pt2.y + d;
    } else {
        let b = pt2.y + (1.0 / m) * pt2.x;
        let b1 = (-1.0 / m) * offset_x + b;
        y_intercept.x = offset_x;
        y_intercept.y = b1;
        result = extend_line_double(y_intercept, pt2, d);
    }
    result.style = styl;
    Ok(result)
}
