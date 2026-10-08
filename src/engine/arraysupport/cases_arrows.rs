//! Point builders of the directed-attack and crossing arrows: DIRATKGND,
//! DIRATKAIR, DIRATKSPT, MFLANE, RAFT, PDF, EXFILTRATION, INFILTRATION,
//! EXPLOIT and ABATIS (first switch of `GetLineArray2Double`).

mod abatis;
mod diratk;
mod diratkair;
mod pdf;

use super::work::{MAX_LENGTH, MIN_LENGTH, Work};
use crate::engine::base::{EngineError, Pt};
use crate::engine::lineutility::arrow::get_arrow_head4_double;
use crate::engine::tactical_lines as lt;

/// Builds the points of the line types of this group; false when the line
/// type belongs to another group.
pub(crate) fn build(w: &mut Work<'_>) -> Result<bool, EngineError> {
    match w.line_type {
        lt::DIRATKGND => diratk::dir_atk_gnd(w)?,
        lt::MFLANE | lt::RAFT => diratk::mf_lane(w)?,
        lt::DIRATKAIR => diratkair::dir_atk_air(w)?,
        lt::PDF => pdf::pdf(w)?,
        lt::DIRATKSPT => diratk::dir_atk_spt(w)?,
        lt::EXFILTRATION | lt::INFILTRATION => diratk::infiltration(w)?,
        lt::EXPLOIT => abatis::exploit(w)?,
        lt::ABATIS => abatis::abatis(w)?,
        _ => return Ok(false),
    }
    Ok(true)
}

/// The size clamp upstream repeats before sizing arrowheads: `d_mbr / div`
/// is kept within the arrow length limits, then `d_mbr` within an optional
/// floor and cap (both scaled by the DPI factor).
pub(crate) fn clamp_mbr(
    d_mbr: f64,
    dpi: f64,
    div: f64,
    floor: Option<f64>,
    cap: Option<f64>,
) -> f64 {
    let mut d = d_mbr;
    if d / div > MAX_LENGTH * dpi {
        d = div * MAX_LENGTH * dpi;
    }
    if d / div < MIN_LENGTH * dpi {
        d = div * MIN_LENGTH * dpi;
    }
    if let Some(f) = floor {
        if d < f * dpi {
            d = f * dpi;
        }
    }
    if let Some(c) = cap {
        if d > c * dpi {
            d = c * dpi;
        }
    }
    d
}

/// `GetArrowHead4Double` into a fresh three-point array.
pub(crate) fn arrow(
    start: Pt,
    end: Pt,
    bisector: i32,
    base: i32,
    style: i32,
) -> Result<[Pt; 3], EngineError> {
    let mut r = [Pt::default(); 3];
    get_arrow_head4_double(start, end, bisector, base, &mut r, style)?;
    Ok(r)
}
