//! `GetAXADDouble` of Channels.java: the axis-of-advance arrowhead built on
//! the two channel edges.

use super::point_index::{at_i, mut_i, set_i};
use crate::engine::base::{EngineError, Pt};
use crate::engine::lineutility::basics::{calc_distance_double, mid_point_double};
use crate::engine::lineutility::extend::extend_along_line_double;
use crate::engine::lineutility::slope::get_offset_point_double;
use crate::engine::tactical_lines as lt;

/// How far a counterattack by fire arrow tip is pulled back from the
/// control point so the rotary feature ends on the anchor.
pub(crate) const CATKBYFIRE_SHIFT: f64 = 45.0;

/// The inputs of `GetAXADDouble`.
#[derive(Debug)]
pub(crate) struct AxadInput<'a> {
    /// Printer scale, always 1.
    pub(crate) printer: f64,
    /// The lower channel edge.
    pub(crate) lower: &'a mut [Pt],
    /// The upper channel edge.
    pub(crate) upper: &'a mut [Pt],
    /// The arrow tip.
    pub(crate) arrow: Pt,
    /// The line type being drawn.
    pub(crate) draw_this: i32,
    /// The arrowhead flare, `channel width / 4` pixels.
    pub(crate) offset_factor: f64,
}

/// Upstream `GetAXADDouble`: fills `line_points` (which must hold at least
/// `lower + upper + 8` points) with the lower edge, the upper edge and the
/// eight arrowhead points. For a long counterattack by fire the edge ends
/// and tip are pulled back 45 pixels while the arrow is built and restored
/// afterwards.
pub(crate) fn get_axad_double(
    input: AxadInput<'_>,
    line_points: &mut [Pt],
) -> Result<(), EngineError> {
    let AxadInput {
        printer,
        lower,
        upper,
        mut arrow,
        draw_this,
        offset_factor,
    } = input;
    let l_lower = i32::try_from(lower.len()).unwrap_or(i32::MAX);
    let l_upper = i32::try_from(upper.len()).unwrap_or(i32::MAX);
    let l_counter = l_lower + l_upper + 8;
    let (lo_last, lo_prev) = (l_lower - 1, l_lower - 2);
    let (up_last, up_prev) = (l_upper - 1, l_upper - 2);

    let pt_upper0 = at_i(upper, up_last)?;
    let pt_lower0 = at_i(lower, lo_last)?;
    let dist = calc_distance_double(at_i(lower, lo_last)?, at_i(lower, lo_prev)?);
    let pulled_back = draw_this == lt::CATKBYFIRE && dist > CATKBYFIRE_SHIFT;
    if pulled_back {
        let mid = mid_point_double(at_i(lower, lo_prev)?, at_i(upper, up_prev)?, 0);
        arrow = extend_along_line_double(arrow, mid, CATKBYFIRE_SHIFT);
        let new_lower = extend_along_line_double(
            at_i(lower, lo_last)?,
            at_i(lower, lo_prev)?,
            CATKBYFIRE_SHIFT,
        );
        set_i(lower, lo_last, new_lower)?;
        let new_upper = extend_along_line_double(
            at_i(upper, up_last)?,
            at_i(upper, up_prev)?,
            CATKBYFIRE_SHIFT,
        );
        set_i(upper, up_last, new_upper)?;
    }

    let edges = Edges {
        lower,
        upper,
        arrow,
        flare: (offset_factor * printer) as i32,
    };
    write_arrowhead(&edges, line_points)?;

    if matches!(
        draw_this,
        lt::SPT_STRAIGHT
            | lt::SPT
            | lt::FRONTAL_ATTACK
            | lt::TURNING_MOVEMENT
            | lt::MOVEMENT_TO_CONTACT
            | lt::AAAAA
            | lt::AIRAOA
            | lt::CATK
            | lt::CATKBYFIRE
    ) {
        mut_i(line_points, l_counter - 6)?.style = 5;
        mut_i(line_points, l_counter - 5)?.style = 5;
    }

    if pulled_back {
        set_i(
            upper,
            up_last,
            Pt {
                x: pt_upper0.x,
                y: pt_upper0.y,
                ..at_i(upper, up_last)?
            },
        )?;
        set_i(
            lower,
            lo_last,
            Pt {
                x: pt_lower0.x,
                y: pt_lower0.y,
                ..at_i(lower, lo_last)?
            },
        )?;
    }
    Ok(())
}

/// The two edges and the arrow tip, with the flare in whole pixels.
struct Edges<'a> {
    lower: &'a [Pt],
    upper: &'a [Pt],
    arrow: Pt,
    flare: i32,
}

/// Copies both edges to `line_points` and appends the eight arrowhead points.
fn write_arrowhead(edges: &Edges<'_>, line_points: &mut [Pt]) -> Result<(), EngineError> {
    let Edges {
        lower,
        upper,
        arrow,
        flare,
    } = *edges;
    let l_lower = i32::try_from(lower.len()).unwrap_or(i32::MAX);
    let l_upper = i32::try_from(upper.len()).unwrap_or(i32::MAX);
    let l_counter = l_lower + l_upper + 8;
    let (lo_last, up_last) = (l_lower - 1, l_upper - 1);
    for j in 0..l_lower {
        set_i(line_points, j, at_i(lower, j)?)?;
    }
    mut_i(line_points, l_lower - 1)?.style = 5;
    for j in 0..l_upper {
        set_i(line_points, l_lower + j, at_i(upper, j)?)?;
    }
    let upper0 = at_i(upper, 0)?;
    for j in l_counter - 8..l_counter {
        set_i(line_points, j, upper0)?;
    }

    let lower_end = at_i(lower, lo_last)?;
    let upper_end = at_i(upper, up_last)?;
    let mut end_line = upper0;
    end_line.x = f64::from(((lower_end.x + upper_end.x) / 2.0) as i32);
    end_line.y = f64::from(((lower_end.y + upper_end.y) / 2.0) as i32);

    let outer_tip = arrow;
    let inner_tip = get_offset_point_double(end_line, outer_tip, -i64::from(flare));
    mut_i(line_points, l_counter - 9)?.style = 5;
    set_i(line_points, l_counter - 8, outer_tip)?;

    let pt0 = Pt::new(upper_end.x, upper_end.y);
    let pt1 = Pt::new(lower_end.x, lower_end.y);
    let temp = get_offset_point_double(pt0, pt1, i64::from(flare));
    set_i(line_points, l_counter - 7, temp)?;
    set_i(line_points, l_counter - 6, lower_end)?;
    set_i(line_points, l_counter - 5, inner_tip)?;
    set_i(line_points, l_counter - 4, upper_end)?;

    let pt0 = Pt::new(lower_end.x, lower_end.y);
    let pt1 = Pt::new(upper_end.x, upper_end.y);
    let temp = get_offset_point_double(pt0, pt1, i64::from(flare));
    set_i(line_points, l_counter - 3, temp)?;
    set_i(line_points, l_counter - 2, outer_tip)?;
    set_i(line_points, l_counter - 1, outer_tip)?;
    mut_i(line_points, l_counter - 1)?.style = 5;

    Ok(())
}
