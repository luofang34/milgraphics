//! `DrawGoodChannel2` and `DrawSegments` of clsChannelUtility.java: draw the
//! partitions of a channel through `GetChannel1Double`.

use crate::engine::base::{At, EngineError, Shape};
use crate::engine::channels::ChannelExternals;
use crate::engine::channels::channel1::{ChannelRequest, get_channel1_double};
use crate::engine::partition::Partition;
use crate::engine::settings::Settings;
use crate::engine::tactical_lines as lt;
use crate::engine::tg::Tg;

/// What every partition of one channel shares.
#[derive(Debug)]
pub(super) struct ChannelDraw<'a, E> {
    pub(super) tg: &'a Tg,
    pub(super) settings: &'a Settings,
    pub(super) ext: &'a E,
    /// The client points as x,y pairs.
    pub(super) pixels: &'a [f64],
    /// The full channel width in pixels.
    pub(super) channel_width: i32,
    /// Distance in pixels from the arrow tip to the back of the arrowhead.
    pub(super) distance_to_channel_point: f64,
}

/// The line type a partition is drawn with: the first segment of the arrow
/// types is flared, later ones are plain channels, and the last partition
/// of an arrow after the first keeps a straight shaft.
fn partition_line_type(line_type: i32, from_segment: i32, last_segment: bool) -> i32 {
    if last_segment {
        if from_segment == 0 {
            return line_type;
        }
        return match line_type {
            lt::SPT | lt::FRONTAL_ATTACK | lt::TURNING_MOVEMENT | lt::MOVEMENT_TO_CONTACT => {
                lt::SPT_STRAIGHT
            }
            lt::MAIN => lt::MAIN_STRAIGHT,
            _ => line_type,
        };
    }
    match line_type {
        lt::LC
        | lt::UNSP
        | lt::DFENCE
        | lt::SFENCE
        | lt::DOUBLEA
        | lt::LWFENCE
        | lt::HWFENCE
        | lt::BBS_LINE
        | lt::SINGLEC
        | lt::DOUBLEC
        | lt::TRIPLE => line_type,
        lt::SPT
        | lt::FRONTAL_ATTACK
        | lt::TURNING_MOVEMENT
        | lt::MOVEMENT_TO_CONTACT
        | lt::MAIN => {
            if from_segment == 0 {
                lt::CHANNEL_FLARED
            } else {
                lt::CHANNEL
            }
        }
        lt::CATK | lt::CATKBYFIRE => lt::CHANNEL_DASHED,
        _ => lt::CHANNEL,
    }
}

/// Upstream `DrawGoodChannel2`: draws the channel along the segments
/// `partition.start..=partition.end`. Failures stop only this partition, as
/// upstream's catch does.
pub(super) fn draw_good_channel2<E: ChannelExternals>(
    draw: &ChannelDraw<'_, E>,
    partition: Partition,
    last_segment: bool,
    shapes: &mut Vec<Shape>,
) {
    if good_channel(draw, partition, last_segment, shapes).is_err() {
        // The remaining partitions are still drawn.
    }
}

fn good_channel<E: ChannelExternals>(
    draw: &ChannelDraw<'_, E>,
    partition: Partition,
    last_segment: bool,
    shapes: &mut Vec<Shape>,
) -> Result<(), EngineError> {
    let (from, to) = (partition.start, partition.end);
    if from < 0 || to < 0 || to < from {
        return Ok(());
    }
    let line_type = partition_line_type(draw.tg.line_type, from, last_segment);
    let num_points = to - from + 2;
    let mut good = Vec::new();
    for j in from..to + 2 {
        let k = usize::try_from(2 * j).unwrap_or(usize::MAX);
        good.push(draw.pixels.at(k)?);
        good.push(draw.pixels.at(k + 1)?);
    }
    let request = ChannelRequest {
        tg: draw.tg,
        settings: draw.settings,
        line_type,
        upper: &good,
        lower: &good,
        upper_counter: num_points,
        lower_counter: num_points,
        channel_width: draw.channel_width / 2,
        useptr: draw.distance_to_channel_point as i32,
    };
    get_channel1_double(&request, draw.ext, shapes)
}

/// Upstream `DrawSegments`: every partition but the last with the
/// per-partition line type, then the last partition as the line's own type.
pub(super) fn draw_segments<E: ChannelExternals>(
    draw: &ChannelDraw<'_, E>,
    partitions: &[Partition],
    shapes: &mut Vec<Shape>,
) {
    let Some((last, rest)) = partitions.split_last() else {
        return;
    };
    for partition in rest {
        draw_good_channel2(draw, *partition, false, shapes);
    }
    draw_good_channel2(draw, *last, true, shapes);
}
