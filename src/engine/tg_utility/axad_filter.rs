//! Port of `FilterAXADPoints` from mil-sym-java
//! RenderMultipoints/clsUtility.java, in pixel form (no geodesic converter
//! round trips).

use crate::engine::base::{At, EngineError, Pt};
use crate::engine::lineutility::basics::calc_distance_double;
use crate::engine::lineutility::extend::extend_along_line_double;
use crate::engine::lineutility::relative::point_relative_to_line_at;
use crate::engine::tactical_lines as lt;
use crate::engine::tg::Tg;

/// Minimum spacing in pixels between kept points after the first two.
const MIN_POINT_SPACING: f64 = 5.0;

/// Upstream `FilterAXADPoints`: for axis-of-advance types, moves or drops
/// the points near the first one so the second point is further from the
/// first than the control point is from the first segment's line (plus 5
/// pixels), then drops later points within 5 pixels of the previous kept
/// point. The first two points and the control point are always kept.
pub(crate) fn filter_axad_points(tg: &mut Tg) -> Result<(), EngineError> {
    if !matches!(
        tg.line_type,
        lt::CATK
            | lt::CATKBYFIRE
            | lt::AIRAOA
            | lt::AAAAA
            | lt::SPT
            | lt::FRONTAL_ATTACK
            | lt::TURNING_MOVEMENT
            | lt::MOVEMENT_TO_CONTACT
            | lt::MAIN
    ) {
        return Ok(());
    }
    let count = tg.pixels.len();
    let pt0 = tg.pixels.at(0)?;
    let pt1 = tg.pixels.at(1)?;
    let control = tg.pixels.at(count.saturating_sub(1))?;
    let pt0_relative = point_relative_to_line_at(pt0, pt1, pt0, control);
    let relative_dist = calc_distance_double(pt0_relative, control) + MIN_POINT_SPACING;

    let mut pts: Vec<Pt>;
    if relative_dist > calc_distance_double(pt0, pt1) {
        pts = vec![pt0, extend_along_line_double(pt0, pt1, relative_dist)];
        let mut found_good_point = false;
        for j in 2..count.saturating_sub(1) {
            let ptj = tg.pixels.at(j)?;
            if found_good_point {
                pts.push(ptj);
            } else if relative_dist <= calc_distance_double(pt0, ptj) {
                pts.push(ptj);
                found_good_point = true;
            }
        }
        pts.push(control);
    } else {
        pts = tg.pixels.clone();
    }

    let mut last_good = pts.at(1)?;
    let mut kept = vec![pts.at(0)?, last_good];
    for j in 2..pts.len().saturating_sub(1) {
        let current = pts.at(j)?;
        if calc_distance_double(current, last_good) > MIN_POINT_SPACING {
            last_good = current;
            kept.push(current);
        }
    }
    kept.push(pts.at(pts.len().saturating_sub(1))?);
    tg.pixels = kept;
    Ok(())
}

#[cfg(test)]
mod tests;
