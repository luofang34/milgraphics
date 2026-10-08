//! Cases of `GetLineArray2Double` that build spiked outlines (FORTL, walls,
//! zones), the isolate/occupy/turn arcs, airfields and air corridors.

mod corridor;
mod fortl;
mod isolate;
mod zone;

use super::work::{Work, get, set, set_style};
use crate::engine::base::{EngineError, Pt};
use crate::engine::lineutility::basics::calc_center_point_double;
use crate::engine::lineutility::bounds::mbr_distance;
use crate::engine::tactical_lines as lt;

/// Builds the points of the line types of this group; false when the line
/// type belongs to another group.
pub(crate) fn build(w: &mut Work<'_>) -> Result<bool, EngineError> {
    match w.line_type {
        lt::FORTL => w.ac = fortl::fortl_points(w.tg, &mut w.p, w.save)?,
        lt::ATWALL | lt::LINE => w.ac = fortl::at_wall_points2(w.tg, &mut w.p, w.save)?,
        lt::ENCIRCLE
        | lt::ZONE
        | lt::OBSAREA
        | lt::OBSFAREA
        | lt::STRONG
        | lt::FORT_REVD
        | lt::FORT => w.ac = zone::zone_points2(w.tg, &mut w.p, w.save)?,
        lt::ISOLATE | lt::CORDONKNOCK | lt::CORDONSEARCH | lt::DENY => isolated(w, 50)?,
        lt::AREA_DEFENSE => isolated(w, 67)?,
        lt::OCCUPY | lt::CONTROL | lt::LOCATE => isolated(w, 32)?,
        lt::RETAIN => isolated(w, 75)?,
        lt::SECURE => isolated(w, 29)?,
        lt::TURN_REVD | lt::TURN => {
            // 2525C changed the order of the first two points.
            w.p.swap(0, 1);
            isolated(w, 29)?;
        }
        lt::AIRFIELD => {
            airfield_center_feature(w)?;
            w.ac = w.vbl;
        }
        lt::SC | lt::MRR | lt::SL | lt::TC | lt::LLTR | lt::SAAFR | lt::AC => {
            corridor::corridor(w)?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}

/// `GetIsolatePointsDouble` and the symbol's point count.
fn isolated(w: &mut Work<'_>, count: i32) -> Result<(), EngineError> {
    isolate::isolate_points(&mut w.p, w.line_type, w.dpi)?;
    w.ac = count;
    Ok(())
}

/// Upstream `AirfieldCenterFeature`: the runway cross in the middle of the
/// outline, drawn with the last five points.
fn airfield_center_feature(w: &mut Work<'_>) -> Result<(), EngineError> {
    let vbl = w.vbl;
    let mut d = mbr_distance(&w.p, vbl - 5)?;
    if d > 350.0 * w.dpi {
        d = 350.0 * w.dpi;
    } else if d < 100.0 * w.dpi {
        d = 100.0 * w.dpi;
    }
    for k in 0..vbl {
        set_style(&mut w.p, k, 0)?;
    }
    let first = get(&w.p, 0)?;
    set(&mut w.p, vbl - 5, Pt { style: 5, ..first })?;
    let mut centre = calc_center_point_double(&w.p, vbl - 6)?;
    centre.x -= d / 10.0;
    centre.style = 0;
    set(&mut w.p, vbl - 4, centre)?;
    let right = Pt {
        x: centre.x + d / 5.0,
        style: 5,
        ..centre
    };
    set(&mut w.p, vbl - 3, right)?;
    set(
        &mut w.p,
        vbl - 2,
        Pt {
            y: centre.y + d / 20.0,
            style: 0,
            ..centre
        },
    )?;
    set(
        &mut w.p,
        vbl - 1,
        Pt {
            y: right.y - d / 20.0,
            style: 0,
            ..right
        },
    )
}
