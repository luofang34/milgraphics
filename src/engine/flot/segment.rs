//! Per-segment flot points (`GetFlotSegment`, `GetFlotSegment2`) and the
//! orientation state that carries a flot's side from one segment to the
//! next.

use super::FlotStyle;
use super::angle::calc_angle_points;
use crate::engine::base::{At, EngineError, idx};
use crate::engine::lineutility::intersect::calc_distance2;
use crate::engine::tactical_lines as tl;

/// Upstream's `bFlip`, `lDirection` and `lLastDirection` `ref<int[]>`
/// values, kept between the segments of one polyline.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct FlipState {
    pub(crate) flip: i32,
    pub(crate) direction: i32,
    pub(crate) last_direction: i32,
}

impl FlipState {
    /// All three unset (-1), as the callers that assign `-1` start.
    pub(crate) fn unset() -> Self {
        Self {
            flip: -1,
            direction: -1,
            last_direction: -1,
        }
    }

    /// All three zero, as a freshly allocated `new int[1]`.
    pub(crate) fn zeroed() -> Self {
        Self {
            flip: 0,
            direction: 0,
            last_direction: 0,
        }
    }
}

/// The four integer coordinates of segment `segment` of `vb`.
pub(crate) fn segment_coords(vb: &[i32], segment: usize) -> Result<[i32; 4], EngineError> {
    let i = segment.saturating_mul(2);
    Ok([
        vb.at(i)?,
        vb.at(i.saturating_add(1))?,
        vb.at(i.saturating_add(2))?,
        vb.at(i.saturating_add(3))?,
    ])
}

/// The segment's heading in degrees as the flot code derives it from the
/// arctangent and the quadrant (0 up, clockwise, y down).
pub(crate) fn heading_degrees(c: [i32; 4]) -> f64 {
    let [x1, y1, x2, y2] = c;
    let num = f64::from(y2) - f64::from(y1);
    let den = f64::from(x2) - f64::from(x1);
    let mut angle = if den == 0.0 {
        std::f64::consts::FRAC_PI_2
    } else {
        (num / den).atan().abs()
    };
    angle *= 180.0 / std::f64::consts::PI;
    if x1 <= x2 && y1 >= y2 {
        90.0 - angle
    } else if x1 <= x2 && y1 <= y2 {
        angle + 90.0
    } else if x1 >= x2 && y1 <= y2 {
        270.0 - angle
    } else {
        270.0 + angle
    }
}

/// Turns the heading into the angle the flot's arcs are centred on, and
/// toggles the side whenever the segment's x direction reverses.
pub(crate) fn oriented_angle(
    c: [i32; 4],
    segment: usize,
    heading: f64,
    state: &mut FlipState,
) -> f64 {
    let [x1, _, x2, _] = c;
    let mut angle = heading;
    if x1 >= x2 {
        angle += 90.0;
        state.direction = 1;
    } else {
        angle -= 90.0;
        state.direction = 0;
    }
    if segment > 0 && state.direction != state.last_direction {
        state.flip = if state.flip == 1 { 0 } else { 1 };
    }
    if state.flip == 1 {
        angle += 180.0;
    }
    angle
}

/// Copies one flot's 30 values into `points` at `offset`.
pub(crate) fn write_arc(
    points: &mut [i32],
    offset: usize,
    arc: &[i32; 30],
) -> Result<(), EngineError> {
    let len = points.len();
    let end = offset.saturating_add(30);
    let dst = points.get_mut(offset..end).ok_or(EngineError::Index {
        index: i64::try_from(end).unwrap_or(i64::MAX),
        len,
    })?;
    dst.copy_from_slice(arc);
    Ok(())
}

/// The location of flot `m` along the segment, truncated to whole pixels.
pub(crate) fn flot_center(c: [i32; 4], m: i32, step: f64, distance: f64) -> (i32, i32) {
    let [x1, y1, x2, y2] = c;
    let t = (f64::from(m) + 0.5) * (f64::from(x2) - f64::from(x1)) * step / distance;
    let u = (f64::from(m) + 0.5) * (f64::from(y2) - f64::from(y1)) * step / distance;
    ((f64::from(x1) + t) as i32, (f64::from(y1) + u) as i32)
}

/// Upstream `GetFlotSegment`: the points of the flots along segment
/// `segment` of `vb`, written to `points` (30 values per flot) when given.
/// Returns the number of points (10 per flot). With `points` `None` only
/// the orientation state advances.
pub(crate) fn get_flot_segment(
    vb: &[i32],
    segment: usize,
    mut points: Option<&mut [i32]>,
    flot_diameter: f64,
    state: &mut FlipState,
) -> Result<i32, EngineError> {
    let c = segment_coords(vb, segment)?;
    if segment == 0 {
        let (x1, x2) = (vb.at(0)?, vb.at(2)?);
        state.flip = i32::from(x1 >= x2);
    }
    let heading = heading_degrees(c);
    let mut distance = calc_distance2(
        i64::from(c[0]),
        i64::from(c[1]),
        i64::from(c[2]),
        i64::from(c[3]),
    );
    let num_segs = (distance / flot_diameter) as i32;
    distance += f64::from(num_segs) * flot_diameter - distance;
    let angle = oriented_angle(c, segment, heading, state);
    let mut written = 0_usize;
    for m in 0..num_segs {
        let (lx, ly) = flot_center(c, m, flot_diameter, distance);
        let arc = calc_angle_points(
            lx,
            ly,
            angle,
            distance / f64::from(num_segs.wrapping_mul(2)),
        );
        if let Some(points) = points.as_deref_mut() {
            write_arc(points, written, &arc)?;
            written += 30;
        }
    }
    state.last_direction = state.direction;
    Ok(num_segs.wrapping_mul(10))
}

/// The flot pitch of `GetFlotSegment2` for each front type.
fn segment2_increment(style: &FlotStyle) -> f64 {
    match style.line_type {
        tl::WF | tl::UWF => style.scaled(40.0),
        tl::WFG | tl::WFY => style.scaled(60.0),
        tl::OCCLUDED | tl::UOF => style.scaled(50.0),
        tl::SF | tl::USF | tl::SFG | tl::SFY | tl::OFY => style.scaled(80.0),
        _ => style.scaled(20.0),
    }
}

/// Upstream `GetFlotSegment2`: as [`get_flot_segment`] for the front
/// types, with the pitch from the line type and arcs of radius 10 scaled.
/// Stationary fronts start on the opposite side.
pub(crate) fn get_flot_segment2(
    style: &FlotStyle,
    vb: &[i32],
    segment: usize,
    points: &mut [i32],
    state: &mut FlipState,
) -> Result<i32, EngineError> {
    let c = segment_coords(vb, segment)?;
    let increment = segment2_increment(style);
    if segment == 0 {
        let stationary = matches!(style.line_type, tl::SF | tl::USF | tl::SFG | tl::SFY);
        let (x1, x2) = (vb.at(0)?, vb.at(2)?);
        let leftward = x1 >= x2;
        state.flip = i32::from(leftward != stationary);
    }
    let heading = heading_degrees(c);
    let mut distance = calc_distance2(
        i64::from(c[0]),
        i64::from(c[1]),
        i64::from(c[2]),
        i64::from(c[3]),
    );
    let num_segs = (distance / increment) as i32;
    distance += f64::from(num_segs) * increment - distance;
    let angle = oriented_angle(c, segment, heading, state);
    for m in 0..num_segs {
        let (lx, ly) = flot_center(c, m, increment, distance);
        let arc = calc_angle_points(lx, ly, angle, style.scaled(10.0));
        write_arc(points, idx(m.wrapping_mul(30), 0)?, &arc)?;
    }
    state.last_direction = state.direction;
    Ok(num_segs.wrapping_mul(10))
}
