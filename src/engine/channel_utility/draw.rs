//! `DrawChannel` and `DrawChannel2` of clsChannelUtility.java.

use super::good_channel::{ChannelDraw, draw_segments};
use super::lc::{draw_lc_single_line_segments, get_lc_pixels};
use super::partitions::get_partitions;
use crate::engine::base::{At, EngineError, Shape};
use crate::engine::channels::ChannelExternals;
use crate::engine::channels::axad::CATKBYFIRE_SHIFT;
use crate::engine::channels::scaled_size::get_scaled_size;
use crate::engine::lineutility::relative::adjust_catkbyfire_control_point;
use crate::engine::settings::{SHIFT_LINES, Settings};
use crate::engine::tactical_lines as lt;
use crate::engine::tg::{CAP_BUTT, Tg};
use crate::engine::tg_utility::channel_width::channel_width;
use crate::engine::tg_utility::points::reorder_pixels;
use crate::engine::tg_utility::segments::{get_lc_partitions, get_segments};

/// Upstream `DrawChannel`: draws the channel line type `line_type` along the
/// graphic's pixels into `shapes`. A counterattack by fire first has its
/// control point pulled back in `tg.pixels`, as upstream does in place.
/// The graphic's line cap becomes butt for the wire and fence types.
pub(crate) fn draw_channel<E: ChannelExternals>(
    tg: &mut Tg,
    line_type: i32,
    settings: &Settings,
    ext: &E,
    shapes: &mut Vec<Shape>,
) -> Result<(), EngineError> {
    let pixels = match get_lc_pixels(tg)? {
        Some(short_line) => short_line,
        None => {
            adjust_catkbyfire_control_point(line_type, &mut tg.pixels, CATKBYFIRE_SHIFT)?;
            tg.pixels.clone()
        }
    };
    let flat: Vec<f64> = pixels.iter().flat_map(|p| [p.x, p.y]).collect();
    draw_channel2(flat, line_type, tg, settings, ext, shapes)
}

/// The channel width in pixels for the line type, and the points the
/// channel is drawn along, or `None` when there is nothing to draw.
struct ChannelSetup {
    channel_width: i32,
    pixels: Vec<f64>,
    distance_to_channel_point: f64,
}

fn channel_setup(
    pixels: Vec<f64>,
    line_type: i32,
    tg: &mut Tg,
) -> Result<Option<ChannelSetup>, EngineError> {
    let scaled = |size: f64| get_scaled_size(size, f64::from(tg.line_thickness), tg.pattern_scale);
    let mut distance = scaled(20.0);
    let (width, points) = match line_type {
        lt::MAIN
        | lt::CATK
        | lt::CATKBYFIRE
        | lt::AIRAOA
        | lt::AAAAA
        | lt::SPT
        | lt::FRONTAL_ATTACK
        | lt::TURNING_MOVEMENT
        | lt::MOVEMENT_TO_CONTACT => {
            let mut pixels = pixels;
            reorder_pixels(&mut pixels)?;
            if pixels.len() / 2 < 3 {
                return Ok(None);
            }
            let width = channel_width(&pixels, &mut distance)? / 2;
            pixels.truncate(pixels.len() - 2);
            (width, pixels)
        }
        lt::LC => (scaled(40.0) as i32, pixels),
        lt::UNSP
        | lt::DFENCE
        | lt::SFENCE
        | lt::DOUBLEA
        | lt::LWFENCE
        | lt::HWFENCE
        | lt::SINGLEC
        | lt::DOUBLEC
        | lt::TRIPLE => {
            let width = if SHIFT_LINES {
                scaled(60.0)
            } else {
                scaled(30.0)
            } as i32;
            tg.line_cap = CAP_BUTT;
            (width, pixels)
        }
        lt::BBS_LINE => (8 * tg.pixels.at(0)?.style, pixels),
        _ => return Err(EngineError::LineType(line_type)),
    };
    if points.len() / 2 < 2 {
        return Ok(None);
    }
    Ok(Some(ChannelSetup {
        channel_width: width,
        pixels: points,
        distance_to_channel_point: distance,
    }))
}

/// Upstream `DrawChannel2`: partitions the line at double-backed segments
/// and draws each partition. A line of contact is partitioned at sharp
/// turns instead, with the tight angles drawn as a plain FLOT.
fn draw_channel2<E: ChannelExternals>(
    pixels: Vec<f64>,
    line_type: i32,
    tg: &mut Tg,
    settings: &Settings,
    ext: &E,
    shapes: &mut Vec<Shape>,
) -> Result<(), EngineError> {
    let Some(setup) = channel_setup(pixels, line_type, tg)? else {
        return Ok(());
    };
    let tg: &Tg = tg;
    let draw = ChannelDraw {
        tg,
        settings,
        ext,
        pixels: &setup.pixels,
        channel_width: setup.channel_width,
        distance_to_channel_point: setup.distance_to_channel_point,
    };
    if line_type == lt::LC {
        let lc_width = get_scaled_size(40.0, f64::from(tg.line_thickness), tg.pattern_scale);
        let (partitions, single_line) = get_lc_partitions(&setup.pixels, lc_width)?;
        draw_segments(&draw, &partitions, shapes);
        if !single_line.is_empty() {
            draw_lc_single_line_segments(tg, ext, &setup.pixels, &single_line, shapes);
        }
    } else {
        let segments = get_segments(&setup.pixels, 3.0)?;
        let partitions = get_partitions(&segments)?;
        draw_segments(&draw, &partitions, shapes);
    }
    Ok(())
}
