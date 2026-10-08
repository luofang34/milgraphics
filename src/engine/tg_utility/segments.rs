//! Ports of `GetSegments` and `GetLCPartitions` from mil-sym-java
//! JavaTacticalRenderer/clsUtility.java: the partitioning of a channel's
//! client line so double-backed segments do not draw channels off the
//! display.

use crate::engine::base::{At, EngineError, Pt};
use crate::engine::lineutility::basics::calc_distance_double;
use crate::engine::lineutility::extend::extend_along_line_double2;
use crate::engine::lineutility::slope::calc_true_slope_double_for_routes;
use crate::engine::partition::Partition;
use crate::engine::tg_utility::points::{in_x_order, in_y_order};
use std::f64::consts::PI;

fn point_at(pixels: &[f64], k: usize) -> Result<Pt, EngineError> {
    Ok(Pt::new(pixels.at(2 * k)?, pixels.at(2 * k + 1)?))
}

/// Upstream `GetSegments`: one flag per segment of the x,y pairs in
/// `pixels`; a segment is false when it doubles back on the previous one
/// (slopes within `1 / factor` radians against the x order, or both steeper
/// than `factor` against the y order). The first segment is always true.
pub(crate) fn get_segments(pixels: &[f64], factor: f64) -> Result<Vec<bool>, EngineError> {
    let count = pixels.len() / 2;
    let mut segments = vec![false; count.saturating_sub(1)];
    *segments.at_mut(0)? = true;
    for j in 0..count.saturating_sub(2) {
        let pt0 = point_at(pixels, j)?;
        let pt1 = point_at(pixels, j + 1)?;
        let pt2 = point_at(pixels, j + 2)?;
        let (vertical1, m1) = calc_true_slope_double_for_routes(pt0, pt1);
        let (vertical2, m2) = calc_true_slope_double_for_routes(pt1, pt2);
        let mut good = true;
        if vertical1
            && vertical2
            && (m1.atan() - m2.atan()).abs() < 1.0 / factor
            && !in_x_order(pt0, pt1, pt2)
        {
            good = false;
        }
        if (!vertical1 || m1.abs() > factor)
            && (!vertical2 || m2.abs() > factor)
            && !in_y_order(pt0, pt1, pt2)
        {
            good = false;
        }
        *segments.at_mut(j + 1)? = good;
    }
    Ok(segments)
}

/// Upstream `GetLCPartitions`: splits a line of contact at sharp turns.
/// Returns the partitions that get a channel and the "single line"
/// partitions whose angle is too small to fit one (drawn as a plain FLOT).
pub(crate) fn get_lc_partitions(
    pixels: &[f64],
    lc_channel_width: f64,
) -> Result<(Vec<Partition>, Vec<Partition>), EngineError> {
    let count = i32::try_from(pixels.len() / 2).unwrap_or(i32::MAX);
    let mut partitions = Vec::new();
    let mut single_line = Vec::new();
    let mut next = Partition { start: 0, end: 0 };
    let mut i: i32 = 0;
    while i < count - 2 {
        let k = usize::try_from(i).unwrap_or(usize::MAX);
        let pt0 = point_at(pixels, k)?;
        let pt1 = point_at(pixels, k + 1)?;
        let pt2 = point_at(pixels, k + 2)?;
        let angle1 = (pt1.y - pt0.y).atan2(pt1.x - pt0.x);
        let angle2 = (pt1.y - pt2.y).atan2(pt1.x - pt2.x);
        let angle = angle1 - angle2;
        let mut degrees = angle * 180.0 / PI;
        if angle < 0.0 {
            degrees += 360.0;
        }
        if degrees > 270.0 {
            if angle_too_small(pt0, pt1, pt2, lc_channel_width) {
                next.end = i - 1;
                partitions.push(next);
                single_line.push(Partition {
                    start: i,
                    end: i + 2,
                });
                i += 1;
                next = Partition {
                    start: i + 1,
                    end: 0,
                };
            }
        } else if degrees < 90.0 {
            next.end = i;
            partitions.push(next);
            next = Partition {
                start: i + 1,
                end: 0,
            };
        }
        i += 1;
    }
    next.end = count - 2;
    partitions.push(next);
    Ok((partitions, single_line))
}

/// Whether the acute angle at `pt1` leaves too little room for the channel:
/// the shorter arm, laid along the longer one, ends closer to its own end
/// than the channel width.
fn angle_too_small(pt0: Pt, pt1: Pt, pt2: Pt, channel_width: f64) -> bool {
    if calc_distance_double(pt0, pt1) < calc_distance_double(pt1, pt2) {
        let new_pt = extend_along_line_double2(pt1, pt2, calc_distance_double(pt1, pt0));
        calc_distance_double(pt0, new_pt) < channel_width
    } else {
        let new_pt = extend_along_line_double2(pt1, pt0, calc_distance_double(pt1, pt2));
        calc_distance_double(pt2, new_pt) < channel_width
    }
}

#[cfg(test)]
mod tests;
