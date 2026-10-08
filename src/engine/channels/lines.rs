//! The channel edge lines of Channels.java: `CoordIL2Double`,
//! `GetChannelArray2Double`, `GetChannel2Double`, `GetLowerChannelLineDouble`,
//! `GetUpperChannelLineDouble`, `FenceType` and `GetTripleCountDouble`.

use super::connect::connect_array_true_double;
use super::point_index::{at_i, new_pts};
use crate::engine::base::{EngineError, Pt};
use crate::engine::lineutility::basics::calc_distance_double;
use crate::engine::tactical_lines as lt;

/// Upstream `FenceType`: 1 for the wire and fence types that carry repeated
/// features, otherwise 0.
pub(crate) fn fence_type(line_type: i32) -> i32 {
    match line_type {
        lt::TRIPLE
        | lt::DOUBLEC
        | lt::SINGLEC
        | lt::HWFENCE
        | lt::LWFENCE
        | lt::UNSP
        | lt::DOUBLEA
        | lt::SFENCE
        | lt::DFENCE => 1,
        _ => 0,
    }
}

/// Upstream `GetTripleCountDouble`: the point capacity needed for the
/// concertina wire and fence channels over the first `counter` points.
pub(crate) fn get_triple_count_double(
    pts: &[Pt],
    counter: i32,
    line_type: i32,
) -> Result<i32, EngineError> {
    let mut total: i32 = 0;
    for j in 0..counter - 1 {
        let d = calc_distance_double(at_i(pts, j)?, at_i(pts, j + 1)?);
        let this_segment = if d <= 10.0 {
            0
        } else {
            ((d - 10.0) / 10.0) as i32
        };
        total = total.wrapping_add(this_segment);
    }
    Ok(match line_type {
        lt::SINGLEC | lt::DOUBLEC | lt::TRIPLE => 6i32
            .wrapping_mul(counter)
            .wrapping_add(37i32.wrapping_mul(total)),
        lt::HWFENCE | lt::LWFENCE | lt::UNSP | lt::DOUBLEA | lt::SFENCE | lt::DFENCE => 4i32
            .wrapping_mul(counter)
            .wrapping_add(4i32.wrapping_mul(total)),
        lt::BBS_LINE => 2 * counter + 1,
        _ => 2 * counter,
    })
}

/// Upstream `CoordIL2Double`: the upper (`upper_lower` 1) or lower (0)
/// channel edge of the first `counter` points, `channel_width` apart (the
/// width is halved first except for LC). Any other `upper_lower` yields
/// zero points.
pub(crate) fn coord_il2_double(
    n_printer: i32,
    pts: &[Pt],
    upper_lower: i32,
    counter: i32,
    line_type: i32,
    channel_width: i32,
) -> Result<Vec<Pt>, EngineError> {
    let mut result = new_pts(counter)?;
    let mut width = channel_width;
    if line_type != lt::LC {
        width /= 2;
    }
    let mut copy = Vec::new();
    for j in 0..counter {
        copy.push(at_i(pts, j)?);
    }
    // GetChannel2Double: `(int) nChannelWidth / 2` of `width * nPrinter`.
    let edge_width = (width * n_printer) / 2;
    let channel_points = connect_array_true_double(edge_width, counter - 1, &copy)?;
    for (slot, cp) in result.iter_mut().zip(channel_points.iter()) {
        if upper_lower == 1 {
            *slot = cp.line2;
        } else if upper_lower == 0 {
            *slot = cp.line1;
        }
    }
    Ok(result)
}

/// Upstream `GetChannelArray2Double`: the channel edge for channel line
/// types; any other type returns `pts` unchanged.
pub(crate) fn get_channel_array2_double(
    n_printer: i32,
    pts: &[Pt],
    upper_lower: i32,
    counter: i32,
    draw_this: i32,
    channel_width: i32,
) -> Result<Vec<Pt>, EngineError> {
    match draw_this {
        lt::LC
        | lt::AIRAOA
        | lt::AAAAA
        | lt::CATK
        | lt::CATKBYFIRE
        | lt::MAIN
        | lt::MAIN_STRAIGHT
        | lt::SPT
        | lt::SPT_STRAIGHT
        | lt::FRONTAL_ATTACK
        | lt::TURNING_MOVEMENT
        | lt::MOVEMENT_TO_CONTACT
        | lt::TRIPLE
        | lt::DOUBLEC
        | lt::SINGLEC
        | lt::HWFENCE
        | lt::BBS_LINE
        | lt::LWFENCE
        | lt::DOUBLEA
        | lt::UNSP
        | lt::SFENCE
        | lt::DFENCE
        | lt::CHANNEL
        | lt::CHANNEL_FLARED
        | lt::CHANNEL_DASHED => coord_il2_double(
            n_printer,
            pts,
            upper_lower,
            counter,
            draw_this,
            channel_width,
        ),
        _ => Ok(pts.to_vec()),
    }
}
