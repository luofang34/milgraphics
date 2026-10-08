//! `getAXADFillShapes` of Channels.java: the fill outline for the axis of
//! advance and channel types, traced around the channel edges and arrowhead.

use super::point_index::at_i;
use crate::engine::base::{EngineError, Pt, Shape, shape_type};
use crate::engine::lineutility::basics::mid_point_double;
use crate::engine::tactical_lines as lt;

/// The points `from..to_exclusive`, as upstream's ascending index loops
/// visit them.
fn run(pts: &[Pt], from: i32, to_exclusive: i32, out: &mut Vec<Pt>) -> Result<(), EngineError> {
    for j in from..to_exclusive {
        out.push(at_i(pts, j)?);
    }
    Ok(())
}

/// The points from `from` down to and including `to`.
fn run_down(pts: &[Pt], from: i32, to: i32, out: &mut Vec<Pt>) -> Result<(), EngineError> {
    let mut j = from;
    while j >= to {
        out.push(at_i(pts, j)?);
        j -= 1;
    }
    Ok(())
}

fn push_at(pts: &[Pt], indices: &[i32], out: &mut Vec<Pt>) -> Result<(), EngineError> {
    for &i in indices {
        out.push(at_i(pts, i)?);
    }
    Ok(())
}

/// The arrow-type fill: the channel's first side, the arrow outline `arrow`
/// (offsets from the end), then the second side back down. `extra` is the
/// number of arrowhead and detail points at the end.
fn arrow_outline(
    pts: &[Pt],
    extra: i32,
    arrow: &[i32],
    out: &mut Vec<Pt>,
) -> Result<(), EngineError> {
    let n = i32::try_from(pts.len()).unwrap_or(i32::MAX);
    run(pts, 0, (n - extra) / 2, out)?;
    let absolute: Vec<i32> = arrow.iter().map(|o| n - o).collect();
    push_at(pts, &absolute, out)?;
    run_down(pts, n - extra - 1, (n - extra) / 2, out)
}

/// The twist-and-triangle fill of the helicopter and airborne arrows.
fn twist_outline(
    pts: &[Pt],
    extra: i32,
    mid_with: i32,
    triangle: [i32; 5],
    out: &mut Vec<Pt>,
) -> Result<(), EngineError> {
    let n = i32::try_from(pts.len()).unwrap_or(i32::MAX);
    run_down(pts, n - extra, (n - extra) / 2 + 1, out)?;
    run(pts, 0, (n - extra) / 2, out)?;
    let last = out
        .last()
        .copied()
        .ok_or(EngineError::Degenerate("empty fill outline"))?;
    let temp = mid_point_double(last, at_i(pts, n - mid_with)?, 0);
    out.push(temp);
    let absolute: Vec<i32> = triangle.iter().map(|o| n - o).collect();
    push_at(pts, &absolute, out)?;
    out.push(temp);
    Ok(())
}

fn outline_points(line_type: i32, pts: &[Pt]) -> Result<Option<Vec<Pt>>, EngineError> {
    let n = i32::try_from(pts.len()).unwrap_or(i32::MAX);
    let mut out = Vec::new();
    match line_type {
        lt::BBS_LINE => run(pts, 0, n, &mut out)?,
        lt::CHANNEL | lt::CHANNEL_FLARED | lt::CHANNEL_DASHED => {
            run(pts, 0, n / 2, &mut out)?;
            run_down(pts, n - 1, n / 2, &mut out)?;
        }
        lt::SPT | lt::CATK | lt::SPT_STRAIGHT | lt::MAIN_STRAIGHT | lt::MAIN => {
            arrow_outline(pts, 8, &[6, 7, 8, 3, 4], &mut out)?;
        }
        lt::AAAAA => twist_outline(pts, 21, 17, [15, 14, 13, 18, 17], &mut out)?,
        lt::AIRAOA => twist_outline(pts, 10, 6, [4, 3, 2, 7, 6], &mut out)?,
        lt::FRONTAL_ATTACK => arrow_outline(pts, 15, &[13, 14, 9, 10, 11], &mut out)?,
        lt::TURNING_MOVEMENT => arrow_outline(pts, 14, &[12, 13, 14, 9, 10], &mut out)?,
        lt::MOVEMENT_TO_CONTACT => arrow_outline(pts, 24, &[22, 23, 24, 19, 20], &mut out)?,
        lt::CATKBYFIRE => arrow_outline(pts, 17, &[15, 16, 17, 12, 13], &mut out)?,
        _ => return Ok(None),
    }
    Ok(Some(out))
}

/// Upstream `getAXADFillShapes`: the fill shape for the types that have
/// one, with no outline colour; `None` for other types.
pub(crate) fn get_axad_fill_shapes(
    line_type: i32,
    pts: &[Pt],
) -> Result<Option<Vec<Shape>>, EngineError> {
    let Some(outline) = outline_points(line_type, pts)? else {
        return Ok(None);
    };
    let mut shape = Shape::new(shape_type::FILL);
    let mut iter = outline.into_iter();
    let first = iter
        .next()
        .ok_or(EngineError::Degenerate("empty fill outline"))?;
    shape.move_to(first);
    for p in iter {
        shape.line_to(p);
    }
    shape.line_color = None;
    Ok(Some(vec![shape]))
}
