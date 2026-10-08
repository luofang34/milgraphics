//! Ports of the point-array helpers in mil-sym-java
//! JavaTacticalRenderer/clsUtility.java and RenderMultipoints/clsUtility.java /
//! clsRenderer.java that touch only `Tg::pixels`.

#[cfg(test)]
mod tests;

use crate::engine::base::{At, EngineError, Pt};
use crate::engine::line_type::classes::{MsInfo, is_autoshape, is_change1_area};
use crate::engine::tactical_lines::*;
use crate::engine::tg::Tg;
use crate::engine::tg_utility::line_classes::is_closed_polygon;
use crate::engine::tg_utility::segment_colors::java_split;

/// Upstream `SymbolID.Version_2525E`.
const VERSION_2525E: i32 = 13;

/// Upstream `ClosePolygon`: appends a copy of the first point unless the
/// last point already equals it.
pub(crate) fn close_polygon(pixels: &mut Vec<Pt>) {
    if let (Some(first), Some(last)) = (pixels.first().copied(), pixels.last().copied()) {
        if first.x != last.x || first.y != last.y {
            pixels.push(Pt::new(first.x, first.y));
        }
    }
}

/// Upstream `CalcIntersectPt`: intersection of the line through `pt1` with
/// slope `m1` and the line through `pt2` with slope `m2`. Parallel lines
/// leave `intersection` unchanged.
pub(crate) fn calc_intersect_pt(pt1: Pt, m1: f64, pt2: Pt, m2: f64, intersection: &mut Pt) {
    if m1 == m2 {
        return;
    }
    let dx2 = (pt1.y - pt2.y + m1 * pt2.x - m1 * pt1.x) / (m2 - m1);
    intersection.x = pt2.x + dx2;
    intersection.y = pt2.y + m2 * dx2;
}

/// Upstream `InYOrder`: the y values are monotonic along the three points.
pub(crate) fn in_y_order(pt0: Pt, pt1: Pt, pt2: Pt) -> bool {
    (pt0.y <= pt1.y && pt1.y <= pt2.y) || (pt2.y <= pt1.y && pt1.y <= pt0.y)
}

/// Upstream `InXOrder`: the x values are monotonic along the three points.
pub(crate) fn in_x_order(pt0: Pt, pt1: Pt, pt2: Pt) -> bool {
    (pt0.x <= pt1.x && pt1.x <= pt2.x) || (pt2.x <= pt1.x && pt1.x <= pt0.x)
}

/// Upstream `ReorderPixels`: reverses the x,y pairs except the last pair.
pub(crate) fn reorder_pixels(pixels: &mut [f64]) -> Result<(), EngineError> {
    let len = pixels.len();
    let count = len / 2;
    let mut reordered = vec![0.0; len];
    for j in 0..count.saturating_sub(1) {
        let from = len
            .checked_sub(2 * j + 4)
            .ok_or(EngineError::Index { index: -1, len })?;
        *reordered.at_mut(2 * j)? = pixels.at(from)?;
        *reordered.at_mut(2 * j + 1)? = pixels.at(from + 1)?;
    }
    let last_x = pixels.at(len
        .checked_sub(2)
        .ok_or(EngineError::Index { index: -2, len })?)?;
    let last_y = pixels.at(len - 1)?;
    let tail = 2 * count.saturating_sub(1);
    *reordered.at_mut(tail)? = last_x;
    *reordered.at_mut(tail + 1)? = last_y;
    pixels.copy_from_slice(&reordered);
    Ok(())
}

/// Upstream `FilterVerticalSegments`: for the axis, fence and route types,
/// moves a point 1 px sideways when it is under 1 px from its predecessor in
/// x, so the segment is never vertical.
pub(crate) fn filter_vertical_segments(tg: &mut Tg) {
    if !matches!(
        tg.line_type,
        MAIN | CATK
            | CATKBYFIRE
            | AIRAOA
            | AAAAA
            | SPT
            | FRONTAL_ATTACK
            | TURNING_MOVEMENT
            | MOVEMENT_TO_CONTACT
            | LC
            | UNSP
            | DFENCE
            | SFENCE
            | DOUBLEA
            | LWFENCE
            | HWFENCE
            | BBS_LINE
            | SINGLEC
            | DOUBLEC
            | TRIPLE
            | MSR_ONEWAY
            | MSR_TWOWAY
            | MSR_ALT
            | ASR_ONEWAY
            | ASR_TWOWAY
            | ASR_ALT
            | TRAFFIC_ROUTE_ONEWAY
            | TRAFFIC_ROUTE_ALT
            | ATWALL
    ) {
        return;
    }
    for j in 1..tg.pixels.len() {
        let (Some(last), Some(current)) = (tg.pixels.get(j - 1).copied(), tg.pixels.get_mut(j))
        else {
            continue;
        };
        if (current.x - last.x).abs() < 1.0 {
            if current.x >= last.x {
                current.x += 1.0;
            } else {
                current.x -= 1.0;
            }
        }
    }
}

/// Upstream `RemoveDuplicatePoints` (RenderMultipoints): drops points within
/// half a pixel of their predecessor, keeping at least the minimum a line
/// (2) or area (3) needs. `ms_info` is the catalog entry for `isAutoshape`.
pub(crate) fn remove_duplicate_points(tg: &mut Tg, ms_info: Option<MsInfo>) {
    let keeps_duplicates = matches!(tg.line_type, SC | MRR | SL | TC | LLTR | AC | SAAFR);
    if !keeps_duplicates && is_autoshape(tg.line_type, ms_info) {
        return;
    }
    match tg.line_type {
        // These keep their segment data.
        CATK | AIRAOA | AAAAA | SPT | FRONTAL_ATTACK | TURNING_MOVEMENT | MOVEMENT_TO_CONTACT
        | MAIN | CATKBYFIRE => return,
        // A comma-delimited H holds per-segment colours.
        BOUNDARY | MSR | ASR | TRAFFIC_ROUTE
            if !tg.h.is_empty() && java_split(&tg.h, ',').len() > 1 =>
        {
            return;
        }
        _ => {}
    }
    if is_change1_area(tg.line_type) {
        return;
    }
    let min_size = if is_closed_polygon(tg.line_type) {
        3
    } else {
        2
    };
    let mut j = 1;
    while j < tg.pixels.len() {
        if let (Some(last), Some(current)) = (tg.pixels.get(j - 1), tg.pixels.get(j)) {
            let close = (current.x - last.x).abs() < 0.5 && (current.y - last.y).abs() < 0.5;
            if close && tg.pixels.len() > min_size {
                tg.pixels.remove(j);
                // Upstream restarts the scan; the loop increment then
                // resumes at the third point.
                j = 1;
            }
        }
        j += 1;
    }
}

/// Upstream `reversePointsRevD`: control measures whose point order changed
/// between 2525C and 2525D are reversed so the drawing code sees one order.
pub(crate) fn reverse_points_rev_d(tg: &mut Tg) {
    let id = &tg.symbol_id;
    if id.len() < 20 {
        return;
    }
    let (Some(set), Some(version)) = (
        id.get(4..6).and_then(|s| s.parse::<i32>().ok()),
        id.get(0..2).and_then(|s| s.parse::<i32>().ok()),
    ) else {
        return;
    };
    if set != 25 {
        return;
    }
    let reverse = match tg.line_type {
        UNSP | LWFENCE | HWFENCE | SINGLEC | DOUBLEC | TRIPLE | LINE => true,
        CLUSTER => version < VERSION_2525E,
        _ => false,
    };
    if reverse {
        tg.pixels.reverse();
    }
}
