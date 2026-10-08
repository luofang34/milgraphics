//! The cases that delegate to the mission-task builders.

use crate::engine::arraysupport::work::{Work, get, set};
use crate::engine::base::{EngineError, Pt};
use crate::engine::dism::{bypass, cover, delay, disrupt, escort, fire, fix, rip, seize, target};
use crate::engine::lineutility::basics::calc_distance_double;
use crate::engine::tactical_lines as lt;

/// Runs the builder of a mission-task line type; false for other types.
pub(crate) fn build(w: &mut Work<'_>) -> Result<bool, EngineError> {
    if let Some(done) = simple(w)? {
        w.ac = done;
        return Ok(true);
    }
    match w.line_type {
        lt::SCREEN | lt::GUARD | lt::COVER => w.ac = cover_points(w)?,
        lt::SARA => w.ac = sara(w)?,
        lt::SEIZE | lt::CAPTURE | lt::EVACUATE => w.ac = seize_points(w)?,
        _ => return Ok(false),
    }
    Ok(true)
}

/// The cases that are one call; the point count is the call's result or a
/// fixed count.
fn simple(w: &mut Work<'_>) -> Result<Option<i32>, EngineError> {
    let (s, lt_) = (w.settings, w.line_type);
    let p = &mut w.p;
    Ok(Some(match lt_ {
        lt::CANALIZE => bypass::get_dism_canalize_double(p, s)?,
        lt::BREACH => bypass::get_dism_breach_double(p, s)?,
        lt::ESCORT => escort::get_dism_escort_double(w.tg, p)?,
        lt::DISRUPT => disrupt::get_dism_disrupt_double(p, s)?,
        lt::CONTAIN => disrupt::get_dism_contain_double(p, s)?,
        lt::PENETRATE => {
            rip::get_dism_penetrate_double(p, s)?;
            7
        }
        lt::MNFLDBLK | lt::BLOCK => {
            target::get_dism_block_double2(p, lt_)?;
            4
        }
        lt::LINTGT | lt::LINTGTS | lt::FPF => {
            target::get_dism_linear_target_double(p, lt_, w.vbl, s)?
        }
        lt::GAP | lt::ASLTXING => {
            target::get_dism_gap_double(p, s)?;
            12
        }
        lt::MNFLDDIS => target::get_dism_minefield_disrupt_double(p, s)?,
        lt::SPTBYFIRE => fire::get_dism_support_by_fire_double(p, s)?,
        lt::ATKBYFIRE => fire::get_dism_atk_by_fire_double(p, s)?,
        lt::BYIMP => bypass::get_dism_by_imp_double(p, s)?,
        lt::CLEAR => fix::get_dism_clear_double(p, s)?,
        lt::BYDIF => rip::get_dism_by_dif_double(p, s)?,
        lt::FIX | lt::MNFLDFIX => fix::get_dism_fix_double(p, lt_, s)?,
        lt::RIP | lt::DEMONSTRATE => rip::get_dism_rip_double(p, lt_, s)?,
        lt::DELAY
        | lt::WITHDRAW
        | lt::DISENGAGE
        | lt::WDRAWUP
        | lt::RETIRE
        | lt::FPOL
        | lt::RPOL
        | lt::PURSUIT => delay::get_delay_graphic_etc_double(p, lt_, s)?,
        lt::ENVELOPMENT => delay::get_envelopment_graphic_double(p, s)?,
        lt::EASY => bypass::get_dism_easy_double(p, s)?,
        lt::DECEIVE => {
            disrupt::get_dism_deceive_double(p)?;
            4
        }
        lt::BYPASS => bypass::get_dism_bypass_double(p, s)?,
        lt::AMBUSH => escort::ambush_points_double(p, s)?,
        _ => return Ok(None),
    }))
}

/// SCREEN, GUARD and COVER: four control points select the revision C
/// drawing.
fn cover_points(w: &mut Work<'_>) -> Result<i32, EngineError> {
    if w.save == 4 {
        cover::get_dism_cover_double_rev_c(&mut w.p, w.line_type, w.save, w.settings)
    } else {
        cover::get_dism_cover_double(&mut w.p, w.line_type, w.settings)
    }
}

/// SARA: the cover drawing with its middle two groups of four swapped, in a
/// 16-point array.
fn sara(w: &mut Work<'_>) -> Result<i32, EngineError> {
    let count = cover::get_dism_cover_double(&mut w.p, w.line_type, w.settings)?;
    let mut out = vec![Pt::default(); 16];
    for j in 0..16 {
        let from = match j {
            4..=7 => j + 4,
            8..=11 => j - 4,
            _ => j,
        };
        set(&mut out, j, get(&w.p, from)?)?;
    }
    w.p = out;
    Ok(count)
}

/// SEIZE, CAPTURE and EVACUATE: with four control points the radius is the
/// first two points' distance and the arc is taken from the last two.
fn seize_points(w: &mut Work<'_>) -> Result<i32, EngineError> {
    let mut radius = 0.0;
    if w.save == 4 {
        radius = calc_distance_double(get(&w.p, 0)?, get(&w.p, 1)?);
        let (p3, p2) = (get(&w.p, 3)?, get(&w.p, 2)?);
        set(&mut w.p, 1, p3)?;
        set(&mut w.p, 2, p2)?;
    }
    seize::get_dism_seize_double(&mut w.p, radius, w.settings)
}
