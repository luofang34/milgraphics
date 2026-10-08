//! Port of `ChannelWidth` from mil-sym-java JavaTacticalRenderer/clsUtility.java.

use crate::engine::base::{At, EngineError, Pt};
use crate::engine::lineutility::basics::calc_distance_double;
use crate::engine::lineutility::slope::calc_true_slope_double2;
use crate::engine::tg_utility::points::calc_intersect_pt;

/// Upstream `ChannelWidth`: the channel width in pixels for an axis of
/// advance, from the distance of the control point (the last pair of
/// `pixels`) to the last segment. `distance_to_channel_point` receives the
/// distance from the arrow tip to the back of the arrowhead; it is left
/// unchanged when there are fewer than three points.
pub(crate) fn channel_width(
    pixels: &[f64],
    distance_to_channel_point: &mut f64,
) -> Result<i32, EngineError> {
    let count = pixels.len() / 2;
    if count < 3 {
        return Ok(0);
    }
    let at = |k: usize| -> Result<f64, EngineError> { pixels.at(k) };
    let last_segment_pt1 = Pt::new(at(2 * count - 6)?, at(2 * count - 5)?);
    let last_segment_pt2 = Pt::new(at(2 * count - 4)?, at(2 * count - 3)?);
    let channel_width_pt = Pt::new(at(2 * count - 2)?, at(2 * count - 1)?);

    let (non_vertical, m) = calc_true_slope_double2(last_segment_pt1, last_segment_pt2);
    let mut distance = 0.0;
    if non_vertical && m != 0.0 {
        let mut intersect = Pt::new(0.0, 0.0);
        calc_intersect_pt(
            channel_width_pt,
            -1.0 / m,
            last_segment_pt2,
            m,
            &mut intersect,
        );
        distance = calc_distance_double(channel_width_pt, intersect);
    }
    if non_vertical && m == 0.0 {
        distance = (channel_width_pt.y - last_segment_pt1.y).abs();
    }
    if !non_vertical {
        distance = (channel_width_pt.x - last_segment_pt1.x).abs();
        *distance_to_channel_point = distance;
        return Ok((distance as i32).wrapping_mul(4));
    }

    let width = ((distance as i32).wrapping_mul(8)).max(2);
    let hypotenuse = calc_distance_double(last_segment_pt2, channel_width_pt);
    *distance_to_channel_point = (hypotenuse * hypotenuse - distance * distance).sqrt();
    Ok(width)
}

#[cfg(test)]
mod tests;
