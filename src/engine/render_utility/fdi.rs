//! Port of `clsRenderer.addFDI` and `clsUtility.GetMBR`: the feint, decoy or
//! dummy indicator drawn on top of a graphic's shapes.

use crate::engine::base::{At, EngineError, Pt, Shape};
use crate::engine::dism::escort::{get_fdi_shape, get_fdi_shape_arrow};
use crate::engine::line_type::classes::MsInfo;
use crate::engine::tactical_lines::*;
use crate::engine::tg::Tg;

/// Upstream `DrawRules.AXIS1`.
const DRAW_RULE_AXIS1: i32 = 501;
/// Upstream `DrawRules.AXIS2`.
const DRAW_RULE_AXIS2: i32 = 502;
/// Entity code of "movement to contact", whose arrowhead sits in a later shape.
const MOVEMENT_TO_CONTACT_ENTITY: u32 = 342_900;

/// Upstream `SymbolUtilities.hasFDI`: the headquarters/task force/dummy digit
/// marks a feint or dummy (1, 3, 5 or 7).
pub(crate) fn has_fdi(hq_tf_dummy: u8) -> bool {
    matches!(hq_tf_dummy, 1 | 3 | 5 | 7)
}

/// Upstream `addFDI`: appends the indicator shape, placed at the arrowhead
/// where the graphic has one. Called only for control measures with
/// `has_fdi`. `ms_info` is the catalog entry (its draw rule selects the axis
/// of advance branch) and `entity` the entity code. A failure adds nothing,
/// as upstream's catch does.
pub(crate) fn add_fdi(tg: &Tg, shapes: &mut Vec<Shape>, ms_info: Option<MsInfo>, entity: u32) {
    if let Ok(shape) = fdi_shape(tg, shapes, ms_info, entity) {
        shapes.push(shape);
    }
}

fn points_of(shapes: &[Shape], index: usize) -> Result<Vec<Pt>, EngineError> {
    let shape = shapes.get(index).ok_or(EngineError::Index {
        index: i64::try_from(index).unwrap_or(i64::MAX),
        len: shapes.len(),
    })?;
    Ok(shape.points())
}

fn from_end(len: usize, back: usize) -> Result<usize, EngineError> {
    len.checked_sub(back)
        .ok_or(EngineError::Index { index: -1, len })
}

fn fdi_shape(
    tg: &Tg,
    shapes: &[Shape],
    ms_info: Option<MsInfo>,
    entity: u32,
) -> Result<Shape, EngineError> {
    let draw_rule = ms_info.map_or(-1, |m| m.draw_rule);
    let three = |points: &[Pt], a: usize, b: usize, c: usize| -> Result<Shape, EngineError> {
        Ok(get_fdi_shape_arrow(
            tg,
            points.at(a)?,
            points.at(b)?,
            points.at(c)?,
        ))
    };
    match tg.line_type {
        MAIN => {
            let points = points_of(shapes, 1)?;
            let n = points.len();
            three(&points, from_end(n, 3)?, from_end(n, 8)?, from_end(n, 7)?)
        }
        _ if draw_rule == DRAW_RULE_AXIS1 || draw_rule == DRAW_RULE_AXIS2 => {
            let index = if entity == MOVEMENT_TO_CONTACT_ENTITY {
                if tg.fill_color.is_some() { 3 } else { 2 }
            } else {
                from_end(shapes.len(), 1)?
            };
            let points = points_of(shapes, index)?;
            let (tip, left, right) = arrowhead_indices(&points);
            three(&points, left, tip, right)
        }
        DIRATKAIR => three(&points_of(shapes, 2)?, 0, 1, 2),
        DIRATKGND => three(&points_of(shapes, 1)?, 7, 4, 9),
        DIRATKSPT | EXFILTRATION | INFILTRATION | EXPLOIT => three(&points_of(shapes, 1)?, 0, 1, 2),
        _ => {
            let (ul, ur, _, _) = shapes_mbr(shapes)?;
            Ok(get_fdi_shape(tg, ul, ur))
        }
    }
}

/// Tip, left and right point indices of an axis-of-advance arrowhead: the
/// third subpath starts at the tip. Without a third subpath all three are 0.
fn arrowhead_indices(points: &[Pt]) -> (usize, usize, usize) {
    let mut moves = 0;
    for (i, p) in points.iter().enumerate() {
        if p.style == 0 {
            moves += 1;
        }
        if moves == 3 {
            return (i, i + 1, i + 4);
        }
    }
    (0, 0, 0)
}

/// Upstream `GetMBR`: the corners (upper-left, upper-right, lower-right,
/// lower-left) of the bounding box of all shape points, starting from the
/// first point of the first shape.
pub(crate) fn shapes_mbr(shapes: &[Shape]) -> Result<(Pt, Pt, Pt, Pt), EngineError> {
    let first = shapes
        .first()
        .and_then(|s| s.points().first().copied())
        .ok_or(EngineError::Degenerate("no shape points"))?;
    let (mut ul, mut ur, mut lr, mut ll) = (first, first, first, first);
    for shape in shapes {
        for p in shape.points() {
            if p.x < ll.x {
                ll.x = p.x;
                ul.x = p.x;
            }
            if p.x > lr.x {
                lr.x = p.x;
                ur.x = p.x;
            }
            if p.y > ll.y {
                ll.y = p.y;
                lr.y = p.y;
            }
            if p.y < ul.y {
                ul.y = p.y;
                ur.y = p.y;
            }
        }
    }
    Ok((ul, ur, lr, ll))
}
