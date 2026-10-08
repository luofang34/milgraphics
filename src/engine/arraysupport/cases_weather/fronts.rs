//! Weather fronts built from the flot module (SF, OFY, occluded, WF, CF)
//! and FLOT.

use super::super::work::{Work, get, scaled_size, set, set_style};
use super::atwall::get_at_wall_points_double;
use crate::engine::base::EngineError;
use crate::engine::flot::FlotStyle;
use crate::engine::flot::flot_line::get_flot_double;
use crate::engine::flot::flot_wf::get_flot2_double;
use crate::engine::flot::occluded::get_occluded_points_double;
use crate::engine::flot::ofy::get_ofy_points_double;
use crate::engine::flot::sf::get_sf_points_double;
use crate::engine::lineutility::basics::reverse_points_double2;

fn style(w: &Work<'_>) -> FlotStyle {
    FlotStyle::new(
        w.line_type,
        f64::from(w.tg.line_thickness),
        w.tg.pattern_scale,
    )
}

/// Appends the original control points after the symbol points; returns
/// the new total.
fn append_originals(w: &mut Work<'_>, at: i32) -> Result<(), EngineError> {
    for j in 0..w.save {
        let q = get(&w.orig, j)?;
        set(&mut w.p, at + j, q)?;
    }
    Ok(())
}

pub(super) fn sf_family(w: &mut Work<'_>) -> Result<(), EngineError> {
    reverse_points_double2(&mut w.p, w.save)?;
    let st = style(w);
    w.vbl = get_sf_points_double(&st, &mut w.p, w.save)?;
    w.ac = w.vbl;
    Ok(())
}

pub(super) fn ofy(w: &mut Work<'_>) -> Result<(), EngineError> {
    reverse_points_double2(&mut w.p, w.save)?;
    let st = style(w);
    w.vbl = get_ofy_points_double(&st, &mut w.p, w.save)?;
    w.ac = w.vbl;
    Ok(())
}

pub(super) fn occluded(w: &mut Work<'_>) -> Result<(), EngineError> {
    reverse_points_double2(&mut w.p, w.save)?;
    let st = style(w);
    w.vbl = get_occluded_points_double(&st, &mut w.p, w.save)?;
    append_originals(w, w.vbl)?;
    w.vbl += w.save;
    w.ac = w.vbl;
    Ok(())
}

pub(super) fn wf(w: &mut Work<'_>) -> Result<(), EngineError> {
    reverse_points_double2(&mut w.p, w.save)?;
    let st = style(w);
    let flot_count = get_flot2_double(&st, &mut w.p, w.save)?;
    append_originals(w, w.vbl - w.save)?;
    w.ac = flot_count + w.save;
    Ok(())
}

pub(super) fn wfg(w: &mut Work<'_>) -> Result<(), EngineError> {
    reverse_points_double2(&mut w.p, w.save)?;
    let st = style(w);
    w.ac = get_flot2_double(&st, &mut w.p, w.save)?;
    Ok(())
}

pub(super) fn cfg(w: &mut Work<'_>) -> Result<(), EngineError> {
    w.vbl = get_at_wall_points_double(w.tg, &mut w.p, w.save)?;
    w.ac = w.vbl;
    Ok(())
}

pub(super) fn cf(w: &mut Work<'_>) -> Result<(), EngineError> {
    w.vbl = get_at_wall_points_double(w.tg, &mut w.p, w.save)?;
    set_style(&mut w.p, w.vbl - 1, 5)?;
    append_originals(w, w.vbl)?;
    w.vbl += w.save;
    set_style(&mut w.p, w.vbl - 1, 5)?;
    w.ac = w.vbl;
    Ok(())
}

pub(super) fn flot(w: &mut Work<'_>) -> Result<(), EngineError> {
    let d = scaled_size(w.tg, 20.0);
    w.ac = get_flot_double(&mut w.p, d, w.save)?;
    Ok(())
}
