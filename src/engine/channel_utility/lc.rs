//! The line of contact special cases of clsChannelUtility.java:
//! `getLCPixels` and `DrawLCSingleLineSegments`.

use crate::engine::base::{At, EngineError, Pt, Shape};
use crate::engine::channels::ChannelExternals;
use crate::engine::channels::scaled_size::get_scaled_size;
use crate::engine::lineutility::bounds::calc_mbr_points;
use crate::engine::partition::Partition;
use crate::engine::tactical_lines as lt;
use crate::engine::tg::Tg;
use crate::style::Rgba;

/// Upstream `getLCPixels`: when the line of contact's bounding box is
/// smaller than one flot symbol in both directions, replaces its points
/// with a horizontal two-point line one flot long. `None` keeps the
/// graphic's own points.
pub(super) fn get_lc_pixels(tg: &Tg) -> Result<Option<Vec<Pt>>, EngineError> {
    if tg.line_type != lt::LC {
        return Ok(None);
    }
    let count = i32::try_from(tg.pixels.len()).unwrap_or(i32::MAX);
    let mut ul = Pt::default();
    let mut lr = Pt::default();
    calc_mbr_points(&tg.pixels, count, &mut ul, &mut lr)?;
    let flot_diameter = get_scaled_size(21.0, f64::from(tg.line_thickness), tg.pattern_scale);
    if lr.x - ul.x >= flot_diameter || lr.y - ul.y >= flot_diameter {
        return Ok(None);
    }
    let first = tg.pixels.at(0)?;
    let second = tg.pixels.at(1)?;
    let x1 = if first.x <= second.x {
        first.x + flot_diameter
    } else {
        first.x - flot_diameter
    };
    Ok(Some(vec![Pt::new(first.x, first.y), Pt::new(x1, first.y)]))
}

/// Upstream `DrawLCSingleLineSegments`: draws the partitions whose angle is
/// too small for a channel as a red FLOT along the client line. Errors end
/// the drawing of the remaining partitions without failing the graphic, as
/// upstream's catch does.
pub(super) fn draw_lc_single_line_segments<E: ChannelExternals>(
    tg: &Tg,
    ext: &E,
    pixels: &[f64],
    partitions: &[Partition],
    shapes: &mut Vec<Shape>,
) {
    for partition in partitions {
        let Ok(flot_shapes) = single_line_flot(tg, ext, pixels, *partition) else {
            return;
        };
        for mut shape in flot_shapes {
            shape.line_color = Some(Rgba::RED);
            shapes.push(shape);
        }
    }
}

fn single_line_flot<E: ChannelExternals>(
    tg: &Tg,
    ext: &E,
    pixels: &[f64],
    partition: Partition,
) -> Result<Vec<Shape>, EngineError> {
    let count = partition.end - partition.start + 1;
    let mut flot_pixels = Vec::new();
    for i in 0..count {
        let k = usize::try_from(2 * (i + partition.start)).unwrap_or(usize::MAX);
        flot_pixels.push(Pt::new(pixels.at(k)?, pixels.at(k + 1)?));
    }
    ext.lc_flot_shapes(tg, flot_pixels)
}
