//! Ports of `clsUtilityCPOF.FilterPoints2`, `ClearPixelsStyle` and
//! `LinesWithSeparateFill`.

use super::groups::SEPARATE_FILL;
use crate::engine::base::{Shape, shape_type};
use crate::engine::flot::get_scaled_size;
use crate::engine::lineutility::basics::calc_distance_double;
use crate::engine::tactical_lines::*;
use crate::engine::tg::Tg;

/// Upstream `FilterPoints2`: removes a point that is closer to its neighbour
/// than the glyph size of the line type. Upstream leaves its search loop with
/// a labelled `break` after the first removal, so at most one point goes per
/// call. Points are never marked clipped (-1) or segmented (-2) here, so the
/// "segmented" types remove nothing in practice; the rules are kept.
pub(crate) fn filter_points2(tg: &mut Tg) {
    if tg.pixels.len() < 3 {
        return;
    }
    let scaled = |size: f64| get_scaled_size(size, f64::from(tg.line_thickness), tg.pattern_scale);
    let (min_spike, segmented) = match tg.line_type {
        PL | DECISION_LINE | FEBA | LOA | LL | EWL | FCL | LOD | LDLC | PLD | HOLD | HOLD_GE
        | RELEASE | HOL | BHL | BRDGHD | BRDGHD_GE | NFL => (scaled(5.0), false),
        ATDITCH | ATDITCHC | ATDITCHM | FLOT | FORT_REVD | FORT | FORTL | STRONG => {
            (scaled(25.0), true)
        }
        LC
        | OBSAREA
        | OBSFAREA
        | ENCIRCLE
        | ZONE
        | LINE
        | ATWALL
        | UNSP
        | SFENCE
        | DFENCE
        | DOUBLEA
        | LWFENCE
        | HWFENCE
        | SINGLEC
        | DOUBLEC
        | TRIPLE
        | ICE_EDGE_RADAR
        | ICE_OPENINGS_FROZEN
        | CRACKS_SPECIFIC_LOCATION => (scaled(35.0), true),
        _ => return,
    };
    let pts = &mut tg.pixels;
    for j in 0..pts.len().saturating_sub(1) {
        let (Some(&pt0), Some(&pt1)) = (pts.get(j), pts.get(j + 1)) else {
            break;
        };
        if calc_distance_double(pt0, pt1) >= min_spike {
            continue;
        }
        let remove = if !segmented {
            if j + 1 == pts.len() - 1 {
                Some(j)
            } else {
                Some(j + 1)
            }
        } else {
            match (pt0.style, pt1.style) {
                (0, -1) | (0, -2) | (-1, -1) | (-1, -2) | (-2, -2) => Some(j + 1),
                (-1, 0) | (-2, 0) | (-2, -1) => Some(j),
                _ => None,
            }
        };
        if let Some(index) = remove {
            pts.remove(index);
            return;
        }
    }
}

/// Upstream `ClearPixelsStyle`: resets every point's style to 0 so later
/// shape builders do not read stale markers, except for the types that keep
/// a segment width in the style.
pub(crate) fn clear_pixels_style(tg: &mut Tg) {
    if matches!(
        tg.line_type,
        BBS_AREA | BBS_LINE | BBS_RECTANGLE | SC | MRR | SL | TC | LLTR | AC | SAAFR | BS_ELLIPSE
    ) {
        return;
    }
    for p in &mut tg.pixels {
        p.style = 0;
    }
}

/// Upstream `LinesWithSeparateFill`: for the line types whose fill is drawn
/// as a separate shape, strips the fill from the outline shapes. Returns
/// whether the type is one of them.
pub(crate) fn lines_with_separate_fill(line_type: i32, shapes: &mut [Shape]) -> bool {
    if line_type == MSDZ {
        return true;
    }
    if !SEPARATE_FILL.contains(&line_type) {
        return false;
    }
    for shape in shapes.iter_mut() {
        if shape.shape_type == shape_type::POLYLINE {
            shape.fill_color = None;
        }
    }
    true
}
