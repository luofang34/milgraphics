//! The small private helpers of DISMSupport.java that several builders
//! share: pixel limits, glyph size tiers, arc approximation, end pieces and
//! angle utilities, plus the point-array access idioms.

use crate::engine::base::{At, EngineError, Pt};
use std::f64::consts::PI;

/// Upstream `maxLength`: the largest decoration size in pixels at 96 DPI.
pub(crate) const MAX_LENGTH: f64 = 100.0;
/// Upstream `minLength`: the smallest decoration size in pixels at 96 DPI.
pub(crate) const MIN_LENGTH: f64 = 2.5;

/// Writes `p` at slot `i`, growing the array with default points as the
/// pre-sized upstream array already had them.
pub(crate) fn put(points: &mut Vec<Pt>, i: usize, p: Pt) {
    if i >= points.len() {
        points.resize(i + 1, Pt::default());
    }
    if let Some(slot) = points.get_mut(i) {
        *slot = p;
    }
}

/// Sets the style of the point at slot `i`.
pub(crate) fn set_style(points: &mut [Pt], i: usize, style: i32) -> Result<(), EngineError> {
    points.at_mut(i)?.style = style;
    Ok(())
}

/// Clamps a decoration size to `[MIN_LENGTH, MAX_LENGTH]` scaled by the DPI
/// factor, in upstream's order (upper bound first).
pub(crate) fn clamp_size(value: f64, dpi_scale: f64) -> f64 {
    let mut v = value;
    if v > MAX_LENGTH * dpi_scale {
        v = MAX_LENGTH * dpi_scale;
    }
    if v < MIN_LENGTH * dpi_scale {
        v = MIN_LENGTH * dpi_scale;
    }
    v
}

/// Upstream `GetTGFontSize`: the glyph size tier for a length in pixels.
pub(crate) fn get_tg_font_size(length: f64) -> f64 {
    if length < 20.0 {
        0.0
    } else if length < 50.0 {
        1.0
    } else if length > 250.0 {
        3.0
    } else {
        2.0
    }
}

fn angle_of(dx: f64, dy: f64, on_axis_positive: bool, zero: f64) -> f64 {
    if dy == 0.0 {
        if dx > 0.0 { 0.0 } else { PI }
    } else if dx == 0.0 {
        if on_axis_positive {
            PI * 0.5
        } else {
            PI * -0.5
        }
    } else {
        zero
    }
}

/// Upstream `ArcApproximationDouble`: 17 points on the ellipse inscribed in
/// the box, from the angle of `(startx, starty)` to the angle of
/// `(endx, endy)`, counterclockwise on screen.
pub(crate) fn arc_approximation_double(
    bounds: (f64, f64, f64, f64),
    start: (f64, f64),
    end: (f64, f64),
) -> [Pt; 17] {
    let (mut left, mut top, mut right, mut bottom) = bounds;
    if left > right {
        std::mem::swap(&mut left, &mut right);
    }
    if top > bottom {
        std::mem::swap(&mut top, &mut bottom);
    }
    let a = (right - left) / 2.0;
    let b = (bottom - top) / 2.0;
    let ctr_x = left + a;
    let ctr_y = top + b;
    let x1 = start.0 - ctr_x;
    let x2 = end.0 - ctr_x;
    let y1 = ctr_y - start.1;
    let y2 = ctr_y - end.1;
    let start_angle = angle_of(x1, y1, y1 > 0.0, y1.atan2(x1));
    let mut end_angle = angle_of(x2, y2, y2 > 0.0, y2.atan2(x2));
    if end_angle <= start_angle {
        end_angle += 2.0 * PI;
    }
    let increment = (end_angle - start_angle) / 16.0;
    let mut out = [Pt::default(); 17];
    let mut t = start_angle;
    for p in &mut out {
        p.x = ctr_x + a * t.cos();
        p.y = ctr_y - b * t.sin();
        t += increment;
    }
    out
}

/// Upstream `DrawOpenRectangleDouble`: the open rectangle through
/// `points[1]`, the far corners and `points[0]`, with styles 0,0,0,5.
pub(crate) fn draw_open_rectangle_double(points: &[Pt]) -> Result<[Pt; 4], EngineError> {
    let (p0, p1, p2) = (points.at(0)?, points.at(1)?, points.at(2)?);
    let mid_x = (p0.x + p1.x) / 2.0;
    let mid_y = (p0.y + p1.y) / 2.0;
    let corner0 = Pt::new(p0.x - mid_x + p2.x, p0.y - mid_y + p2.y);
    let corner1 = Pt::new(p1.x - mid_x + p2.x, p1.y - mid_y + p2.y);
    let mut out = [p1, corner1, corner0, p0];
    for p in &mut out {
        p.style = 0;
    }
    out[3].style = 5;
    Ok(out)
}

/// Upstream `DetermineDirectionDouble`: 1 when the figure opens to the right.
pub(crate) fn determine_direction_double(points: &[Pt]) -> Result<i32, EngineError> {
    let (p0, p1, p2) = (points.at(0)?, points.at(1)?, points.at(2)?);
    if p0.x == p1.x {
        return Ok(i32::from(p2.x < p0.x));
    }
    let slope = (p0.y - p1.y) / (p0.x - p1.x);
    let b = p0.y - slope * p0.x;
    Ok(i32::from((p2.y - b) / slope > p2.x))
}

/// Upstream `CalcEndpieceDeltasDouble`: the (dx, dy) of an end-piece barb
/// rotated `angle_delta` from the figure's axis.
pub(crate) fn calc_endpiece_deltas_double(
    points: &[Pt],
    angle_delta: f64,
    dpi_scale: f64,
) -> Result<(f64, f64), EngineError> {
    let (p0, p1, p2) = (points.at(0)?, points.at(1)?, points.at(2)?);
    let mid_x = (p0.x + p1.x) / 2.0;
    let mid_y = (p0.y + p1.y) / 2.0;
    let height = ((p1.x - p0.x) * (p1.x - p0.x) + (p1.y - p0.y) * (p1.y - p0.y)).sqrt();
    let length = ((p2.x - mid_x) * (p2.x - mid_x) + (p2.y - mid_y) * (p2.y - mid_y)).sqrt();
    let mut diag = (height + length) / 20.0;
    if diag > MAX_LENGTH / 5.0 * dpi_scale {
        diag = MAX_LENGTH / 5.0 * dpi_scale;
    }
    if diag < MIN_LENGTH * dpi_scale {
        diag = MIN_LENGTH * dpi_scale;
    }
    let angle = (p2.y - mid_y).atan2(p2.x - mid_x) + angle_delta;
    Ok((diag * angle.cos(), diag * angle.sin()))
}

/// Upstream `DrawEndpieceDeltasDouble`: two barbs from `point`, each a
/// move (style 0) and a line end (style 5).
pub(crate) fn draw_endpiece_deltas_double(
    point: Pt,
    d1: f64,
    d2: f64,
    d3: f64,
    d4: f64,
) -> [Pt; 4] {
    let mut first = point;
    first.style = 0;
    let mut second = Pt::new(point.x + d1, point.y + d2);
    second.style = 5;
    let mut third = point;
    third.style = 0;
    let mut fourth = Pt::new(point.x + d3, point.y + d4);
    fourth.style = 5;
    [first, second, third, fourth]
}

/// Upstream `normalizeAngle`: an angle in `[0, 360)` computed in `float`.
pub(crate) fn normalize_angle(angle: f32) -> f32 {
    (3_600_000.0_f32 + angle) % 360.0
}

fn is_in_range(min: f32, max: f32, target: f32) -> bool {
    let target = normalize_angle(target);
    let min = normalize_angle(min);
    let max = normalize_angle(max);
    if min < max {
        return min <= target && target <= max;
    }
    min <= target || target <= max
}

fn get_angle_between_points(x1: f64, y1: f64, x2: f64, y2: f64) -> f32 {
    (y2 - y1).atan2(x2 - x1).to_degrees() as f32
}

/// Upstream `ReverseDelayArc`: whether the semicircle at the end of the
/// withdraw, delay and retire arrows bends to the other side.
pub(crate) fn reverse_delay_arc(points: &[Pt]) -> Result<bool, EngineError> {
    let (p1, p2, p3) = (points.at(0)?, points.at(1)?, points.at(2)?);
    let line_angle = get_angle_between_points(p1.x, p1.y, p2.x, p2.y);
    let curve_angle = get_angle_between_points(p2.x, p2.y, p3.x, p3.y);
    let upper = curve_angle + 180.0;
    Ok(!is_in_range(curve_angle, upper, line_angle))
}

/// Upstream `side`: 0 when the point is left of the line, 1 when right,
/// 2 when collinear.
pub(crate) fn side(x1: f64, y1: f64, x2: f64, y2: f64, px: f64, py: f64) -> i32 {
    let o = (x2 - x1) * (py - y1) - (y2 - y1) * (px - x1);
    if o > 0.0 {
        0
    } else if o < 0.0 {
        1
    } else {
        2
    }
}
