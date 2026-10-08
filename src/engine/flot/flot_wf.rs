//! Port of the warm-front family builders in flot.java
//! (`GetFlotCount2Double`, `GetFlot2Double`) for WF, UWF, WFG and WFY.

use super::segment::{FlipState, get_flot_segment2, segment_coords};
use super::wf_style10::{Scratch, Style10, mid_features, segment_ends};
use super::{FlotStyle, budgeted_len, count, int_coords, slot, store};
use crate::engine::base::{At, EngineError, Pt, idx};
use crate::engine::lineutility::basics::calc_distance_double;
use crate::engine::lineutility::intersect::calc_distance2;
use crate::engine::tactical_lines as tl;

/// Upstream `GetFlotCount2Double`: the points the front's flots need
/// (`n_factor` per flot, two more for a WFG/WFY segment too short for one).
pub(crate) fn get_flot_count2_double(
    style: &FlotStyle,
    pts: &[Pt],
    num_pts: i32,
) -> Result<i32, EngineError> {
    let (increment, factor) = match style.line_type {
        tl::WF | tl::UWF => (style.scaled(40.0), 10),
        tl::WFG => (style.scaled(60.0), 17),
        tl::WFY => (style.scaled(60.0), 20),
        _ => (style.scaled(20.0), 10),
    };
    let n = count(num_pts)?;
    let mut total = 0_i32;
    for j in 0..n.saturating_sub(1) {
        let d = calc_distance_double(pts.at(j)?, pts.at(j + 1)?);
        let segs = (d / increment) as i32;
        total = total.wrapping_add(segs.wrapping_mul(factor));
        if matches!(style.line_type, tl::WFG | tl::WFY) && segs == 0 {
            total = total.wrapping_add(2);
        }
    }
    Ok(total)
}

fn flot_pitch(style: &FlotStyle) -> f64 {
    match style.line_type {
        tl::WF | tl::UWF => style.scaled(40.0),
        tl::WFG | tl::WFY => style.scaled(60.0),
        _ => style.scaled(20.0),
    }
}

/// What `GetFlot2Double` needs to place one flot point.
struct PointCtx<'a> {
    j: i32,
    seg_pts: i32,
    ends: ([i32; 2], [i32; 2]),
    style: &'a FlotStyle,
}

/// Places the flot point `x`,`y` as output point `written` and styles it;
/// every tenth point closes a flot and feeds the mid-segment features.
fn place_point(
    pts: &mut Vec<Pt>,
    written: usize,
    (x, y): (i32, i32),
    ctx: &PointCtx<'_>,
    s10: &mut Style10,
    scratch: &mut Scratch,
) -> Result<(), EngineError> {
    {
        let p = slot(pts, written)?;
        p.x = f64::from(x);
        p.y = f64::from(y);
    }
    if !matches!(ctx.style.line_type, tl::WF | tl::WFG | tl::WFY) {
        slot(pts, written)?.style = 0;
        return Ok(());
    }
    if (written + 1) % 10 != 0 {
        slot(pts, written)?.style = 9;
        return Ok(());
    }
    slot(pts, written)?.style = 10;
    let current = pts.at(written)?;
    if ctx.j < ctx.seg_pts - 1 {
        s10.push(super::styled(current, 0))?;
        if ctx.j < ctx.seg_pts - 2 {
            mid_features(ctx.style, s10, scratch)?;
        }
        Ok(())
    } else {
        segment_ends(ctx.style, s10, scratch, ctx.ends, current)
    }
}

/// Upstream `GetFlot2Double`: replaces the first `num_pts` points of `pts`
/// with the front's flots and returns the number of points. WFG and WFY
/// append the collected line pieces and dots after the flots.
pub(crate) fn get_flot2_double(
    style: &FlotStyle,
    pts: &mut Vec<Pt>,
    num_pts: i32,
) -> Result<i32, EngineError> {
    let flot_count = get_flot_count2_double(style, pts, num_pts)?;
    if flot_count <= 0 {
        return Ok(0);
    }
    let n = count(num_pts)?;
    let mut s10 = Style10::new(budgeted_len(flot_count, 1)?);
    let increment = flot_pitch(style);
    let vb = int_coords(pts, n)?;
    let mut state = FlipState::unset();
    let mut scratch = Scratch::default();
    let mut written = 0_usize;
    let limit = idx(flot_count, 0)?;
    for l in 0..n.saturating_sub(1) {
        let c = segment_coords(&vb, l)?;
        let d = calc_distance2(
            i64::from(c[0]),
            i64::from(c[1]),
            i64::from(c[2]),
            i64::from(c[3]),
        );
        let segs = (d / increment) as i32;
        if segs <= 0 {
            s10.push(Pt::styled(f64::from(c[0]), f64::from(c[1]), 0))?;
            s10.push(Pt::styled(f64::from(c[2]), f64::from(c[3]), 5))?;
            continue;
        }
        let mut points = vec![0_i32; budgeted_len(segs, 30)?];
        let seg_pts = get_flot_segment2(style, &vb, l, &mut points, &mut state)?;
        for j in 0..seg_pts {
            let k = idx(j, 0)? * 3;
            if j < seg_pts - 1 {
                scratch.pt1.x = f64::from(points.at(k + 3)?);
                scratch.pt1.y = f64::from(points.at(k + 4)?);
                scratch.pt1.style = points.at(k + 5)?;
            }
            if written < limit {
                let ctx = PointCtx {
                    j,
                    seg_pts,
                    ends: ([c[0], c[1]], [c[2], c[3]]),
                    style,
                };
                let at = (points.at(k)?, points.at(k + 1)?);
                place_point(pts, written, at, &ctx, &mut s10, &mut scratch)?;
                written += 1;
            }
        }
        let end_style = if matches!(style.line_type, tl::WF | tl::WFG | tl::WFY) {
            10
        } else {
            5
        };
        let last = idx(i32::try_from(written).unwrap_or(i32::MAX) - 1, pts.len())?;
        slot(pts, last)?.style = end_style;
    }
    if matches!(style.line_type, tl::WFG | tl::WFY) {
        for j in 0..s10.len() {
            store(pts, written, s10.get(j)?)?;
            written += 1;
        }
    }
    i32::try_from(written).map_err(|_| EngineError::Degenerate("flot point count overflow"))
}
