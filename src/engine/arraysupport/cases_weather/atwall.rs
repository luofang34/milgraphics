//! Port of `GetATWallPointsDouble`: the teeth of the cold-front family
//! (CF, UCF, CFG, CFY).

use super::super::work::{get, scaled_size, set, set_style};
use crate::engine::base::{EngineError, Pt};
use crate::engine::lineutility::basics::calc_distance_double;
use crate::engine::lineutility::extend::{
    extend_along_line_double_style, extend_directed_line, extend_directed_line_style,
    extend_line_double, extend_line2_double,
};
use crate::engine::tactical_lines as lt;
use crate::engine::tg::Tg;

/// The sizes of one segment's pattern: step, tooth size and tooth count.
fn segment_pattern(tg: &Tg, length: f64) -> (f64, f64, i32) {
    match tg.line_type {
        lt::UCF | lt::CF | lt::CFG | lt::CFY => {
            let increment = scaled_size(tg, 60.0);
            let spike = scaled_size(tg, 20.0);
            let ratio = length / increment;
            let remainder = ratio - f64::from(ratio as i32);
            let limit = if remainder < 0.75 {
                ratio as i32
            } else {
                ratio as i32 + 1
            };
            (increment, spike, limit)
        }
        _ => {
            let increment = scaled_size(tg, 20.0);
            (
                increment,
                scaled_size(tg, 10.0),
                (length / increment) as i32 - 1,
            )
        }
    }
}

/// The points before a tooth: where the line leaves the previous tooth and,
/// for the glyph types, the dot or cross between teeth. Returns the first
/// point of the tooth itself.
fn lead_in(tg: &Tg, sp: &mut Vec<Pt>, ends: (Pt, Pt), k: i32, increment: f64) -> Pt {
    let (a, b) = ends;
    let s = |size: f64| scaled_size(tg, size);
    let off = -f64::from(k) * increment;
    let base = |d: f64, style: i32| extend_line2_double(b, a, off + d, style);
    match (tg.line_type, k > 0) {
        (lt::CFG, true) => {
            sp.push(base(s(45.0), 0));
            sp.push(base(s(4.0), 5));
            sp.push(base(-s(1.0), 20));
            base(-s(10.0), 0)
        }
        (lt::CFY, true) => {
            sp.push(base(s(45.0), 0));
            let second = base(s(10.0), 5);
            sp.push(second);
            let third = extend_along_line_double_style(second, b, s(5.0), 0);
            sp.push(third);
            let fourth = extend_along_line_double_style(third, b, s(10.0), 5);
            sp.push(fourth);
            sp.push(extend_directed_line_style(
                third,
                fourth,
                fourth,
                3,
                s(5.0),
                0,
            ));
            sp.push(extend_directed_line_style(
                fourth,
                third,
                third,
                2,
                s(5.0),
                5,
            ));
            base(-s(10.0), 0)
        }
        (lt::CFG | lt::CFY, false) => base(-s(45.0), 0),
        _ => base(-s(30.0), 0),
    }
}

/// The tooth itself: the base, the tip and the closing points.
fn tooth(tg: &Tg, sp: &mut Vec<Pt>, ends: (Pt, Pt), k: i32, dims: (f64, f64)) {
    let (a, b) = ends;
    let (increment, spike) = dims;
    let glyph = matches!(tg.line_type, lt::CF | lt::CFG | lt::CFY);
    let mut base_pt = extend_line2_double(b, a, -f64::from(k) * increment - spike, 0);
    if glyph {
        base_pt.style = 9;
    }
    sp.push(base_pt);
    let pt0 = extend_line_double(a, base_pt, spike / 2.0);
    let mut tip = pt0;
    if a.x > b.x {
        tip = extend_directed_line(a, base_pt, pt0, 2, spike);
    }
    if a.x < b.x {
        tip = extend_directed_line(a, base_pt, pt0, 3, spike);
    }
    if a.x == b.x {
        tip = pt0;
        tip.x = if a.y < b.y {
            pt0.x - spike
        } else {
            pt0.x + spike
        };
    }
    if glyph {
        tip.style = 9;
    }
    sp.push(tip);
    let mut close = extend_line2_double(a, base_pt, spike, 0);
    match tg.line_type {
        lt::CF => close.style = 10,
        lt::CFG | lt::CFY => {
            close.style = 10;
            sp.push(close);
            close = extend_line2_double(a, base_pt, spike, 0);
        }
        _ => {}
    }
    sp.push(close);
}

/// Upstream `GetATWallPointsDouble`: returns the number of points.
pub(super) fn get_at_wall_points_double(
    tg: &Tg,
    p: &mut [Pt],
    save: i32,
) -> Result<i32, EngineError> {
    let mut sp: Vec<Pt> = Vec::new();
    if matches!(tg.line_type, lt::CFG | lt::CFY) {
        set_style(p, 0, 0)?;
        sp.push(get(p, 0)?);
    }
    for j in 0..save - 1 {
        let ends = (get(p, j)?, get(p, j + 1)?);
        let (increment, spike, limit) = segment_pattern(tg, calc_distance_double(ends.0, ends.1));
        if limit < 1 {
            sp.push(ends.0);
            sp.push(ends.1);
            continue;
        }
        for k in 0..limit {
            let mut lead = lead_in(tg, &mut sp, ends, k, increment);
            if tg.line_type == lt::CF {
                lead.style = 0;
            }
            sp.push(lead);
            tooth(tg, &mut sp, ends, k, (increment, spike));
        }
        // The segment end is the control point itself, so its style change
        // is visible to the next segment.
        set_style(p, j + 1, 0)?;
        sp.push(get(p, j + 1)?);
    }
    for (j, q) in sp.iter().enumerate() {
        set(p, i32::try_from(j).unwrap_or(0), *q)?;
    }
    let n = i32::try_from(sp.len()).unwrap_or(i32::MAX);
    set_style(p, n - 1, 5)?;
    Ok(n)
}
