//! The mission-task and axis rules of `AddModifiers2` (Modifier2.java):
//! labels placed on points of the drawn symbol.

use super::add::{area_modifier, integral_modifier, px};
use super::center_label::is_app6e_2;
use super::geo::Geo;
use super::layout::{add_dtg, get_mbr};
use super::{
    ABOVE_END_INSIDE, ABOVE_MIDDLE, ABOVE_MIDDLE_PERPENDICULAR, ABOVE_START_INSIDE, AREA, TO_END,
};
use crate::engine::base::{At, EngineError, Pt};
use crate::engine::lineutility::basics::{
    calc_center_point_double2, calc_distance_double, mid_point_double,
};
use crate::engine::lineutility::extend::{extend_along_line_double, extend_along_line_double2};
use crate::engine::tactical_lines as tl;
use crate::engine::tg::Tg;

/// Index of the first drawn point carrying the marker style 14, where a
/// task puts its label.
fn marker_index(tg: &Tg) -> Option<i32> {
    tg.pixels
        .iter()
        .position(|p| p.style == 14)
        .and_then(|j| i32::try_from(j).ok())
}

/// Labels `line_type` if it is one of the mission-task rules.
pub(super) fn task_labels(
    tg: &mut Tg,
    g: &mut Geo<'_>,
    line_type: i32,
) -> Result<bool, EngineError> {
    let cs = 1.0;
    let label = g.label.clone();
    let name = tg.t.clone();
    match line_type {
        tl::BS_RECTANGLE | tl::BBS_RECTANGLE => {
            let first = tg.pixels.get(..4).ok_or(EngineError::Index {
                index: 3,
                len: tg.pixels.len(),
            })?;
            let c = calc_center_point_double2(first, 4)?;
            area_modifier(tg, &name, AREA, -0.125 * cs, (c, c), false);
        }
        tl::CONVOY | tl::HCONVOY => {
            let pt2 = mid_point_double(px(tg, 0)?, px(tg, 3)?, 0);
            let pt3 = mid_point_double(px(tg, 1)?, px(tg, 2)?, 0);
            let (v, h) = (tg.v.clone(), tg.h.clone());
            area_modifier(tg, &v, ABOVE_END_INSIDE, 0.0, (pt2, pt3), false);
            area_modifier(tg, &h, ABOVE_START_INSIDE, 0.0, (pt2, pt3), false);
            add_dtg(tg, ABOVE_MIDDLE, (1.2 * cs, 2.2 * cs), pt2, pt3);
        }
        tl::BREACH | tl::BYPASS | tl::CANALIZE => {
            let path = (px(tg, 1)?, px(tg, 2)?);
            area_modifier(
                tg,
                &label,
                ABOVE_MIDDLE_PERPENDICULAR,
                -0.125 * cs,
                path,
                true,
            );
        }
        tl::PENETRATE | tl::CLEAR => task_segment(tg, &label, (2, 3))?,
        tl::DISRUPT => task_segment(tg, &label, (4, 5))?,
        tl::FIX => task_segment(tg, &label, (0, 1))?,
        tl::ISOLATE
        | tl::OCCUPY
        | tl::RETAIN
        | tl::SECURE
        | tl::CONTROL
        | tl::LOCATE
        | tl::AREA_DEFENSE => task_midpoint(tg, &label, (13, 14), ABOVE_MIDDLE)?,
        tl::SEIZE | tl::CAPTURE | tl::EVACUATE => {
            task_midpoint(tg, &label, (26, 27), ABOVE_MIDDLE)?;
        }
        tl::TURN => task_midpoint(tg, &label, (12, 13), AREA)?,
        tl::CONTAIN => contain(tg, &label)?,
        tl::BLOCK => {
            if let Some(j) = marker_index(tg) {
                integral_modifier(tg, &label, ABOVE_MIDDLE, 0.0, (j, j + 1), true)?;
            }
        }
        tl::CORDONKNOCK | tl::CORDONSEARCH | tl::DENY => cordon(tg, g, &label)?,
        tl::ESCORT => escort(tg, &label)?,
        tl::EXFILTRATION | tl::INFILTRATION => {
            if let Some(pt1) = g.ends.pt1 {
                area_modifier(tg, &label, ABOVE_MIDDLE, 0.0, (g.ends.pt0, pt1), true);
            }
        }
        tl::FOLLA => follow(tg, &name, (0, None))?,
        tl::FOLSP => follow(tg, &name, (3, Some(6)))?,
        tl::ENVELOPMENT => integral_modifier(tg, &label, ABOVE_MIDDLE, 0.0, (0, 1), true)?,
        tl::MOBILE_DEFENSE => integral_modifier(tg, &label, AREA, 0.0, (16, 16), true)?,
        _ => return ship_and_flank_labels(tg, g, line_type),
    }
    Ok(true)
}

/// The label above the segment between two of the drawn points.
fn task_segment(tg: &mut Tg, label: &str, (a, b): (i32, i32)) -> Result<(), EngineError> {
    let path = (px(tg, a)?, px(tg, b)?);
    area_modifier(tg, label, ABOVE_MIDDLE, -0.125, path, true);
    Ok(())
}

/// The label at the middle of the segment between two of the drawn points.
fn task_midpoint(
    tg: &mut Tg,
    label: &str,
    (a, b): (i32, i32),
    kind: i32,
) -> Result<(), EngineError> {
    let mid = mid_point_double(px(tg, a)?, px(tg, b)?, 0);
    area_modifier(tg, label, kind, -0.125, (mid, mid), true);
    Ok(())
}

/// `CONTAIN`: the label on the curve, and "ENY" even for friendly forces.
fn contain(tg: &mut Tg, label: &str) -> Result<(), EngineError> {
    let pt0 = px(tg, 13)?;
    px(tg, 14)?;
    area_modifier(tg, label, ABOVE_MIDDLE, -0.125, (pt0, pt0), true);
    if let Some(j) = marker_index(tg) {
        let path = (px(tg, j)?, px(tg, j + 1)?);
        area_modifier(tg, "ENY", ABOVE_MIDDLE, 0.0, path, true);
    }
    Ok(())
}

/// `CORDONKNOCK`, `CORDONSEARCH` and `DENY`: the label beside the arrow
/// tail, three quarters of its width along the line.
fn cordon(tg: &mut Tg, g: &Geo<'_>, label: &str) -> Result<(), EngineError> {
    let (pt0, pt1) = (px(tg, 13)?, px(tg, 0)?);
    let mut width = g.sw(label);
    if pt0.x < pt1.x {
        width = -width;
    }
    let pt1 = extend_along_line_double2(pt0, pt1, 0.75 * f64::from(width));
    let mid = mid_point_double(pt0, pt1, 0);
    area_modifier(tg, label, ABOVE_MIDDLE, 0.0, (mid, mid), true);
    Ok(())
}

/// `ESCORT`: "E" at both ends unless the two middle points coincide.
fn escort(tg: &mut Tg, label: &str) -> Result<(), EngineError> {
    if tg.pixels.len() != 6 {
        return Ok(());
    }
    let (p2, p3) = (px(tg, 2)?, px(tg, 3)?);
    if p2.x == p3.x && p2.y == p3.y {
        return Ok(());
    }
    area_modifier(tg, label, TO_END, 0.0, (p2, px(tg, 1)?), true);
    area_modifier(tg, label, TO_END, 0.0, (p3, px(tg, 4)?), true);
    Ok(())
}

/// `FOLLA` and `FOLSP`: the name along a line from `start` towards the
/// middle of points 5 and 6 (`FOLLA`) or point `end` (`FOLSP`), ending ten
/// pixels short.
fn follow(tg: &mut Tg, name: &str, (start, end): (i32, Option<i32>)) -> Result<(), EngineError> {
    let pt0 = px(tg, start)?;
    let pt1 = match end {
        Some(i) => px(tg, i)?,
        None => mid_point_double(px(tg, 5)?, px(tg, 6)?, 0),
    };
    let pt1 = extend_along_line_double(pt1, pt0, -10.0);
    area_modifier(tg, name, ABOVE_MIDDLE, 0.0, (pt0, pt1), true);
    Ok(())
}

/// `SHIP_AOI_RECTANGULAR`, `SHIP_AOI_CIRCULAR`, `NOTACK`, `MFLANE`.
pub(super) fn ship_and_flank_labels(
    tg: &mut Tg,
    g: &Geo<'_>,
    line_type: i32,
) -> Result<bool, EngineError> {
    let cs = 1.0;
    let label = g.label.clone();
    match line_type {
        tl::DEFENDED_AREA_RECTANGULAR => {
            let left = mid_point_double(px(tg, 0)?, px(tg, 1)?, 0);
            let right = mid_point_double(px(tg, 2)?, px(tg, 3)?, 0);
            let text = format!("{label}{}{}", g.t_dash, tg.t);
            area_modifier(tg, &text, ABOVE_MIDDLE, 0.0, (left, right), false);
        }
        tl::SHIP_AOI_RECTANGULAR => {
            let path = if px(tg, 0)?.x > px(tg, 3)?.x {
                (px(tg, 0)?, px(tg, 3)?)
            } else {
                (px(tg, 1)?, px(tg, 2)?)
            };
            area_modifier(tg, &label, ABOVE_MIDDLE, 0.6 * cs, path, false);
        }
        tl::NOTACK => {
            let c = mid_point_double(px(tg, 0)?, tg.pixels.at(tg.pixels.len() / 2)?, 0);
            area_modifier(tg, &label, AREA, -1.0, (c, c), false);
            add_dtg(tg, AREA, (cs, 2.0 * cs), c, c);
        }
        tl::SHIP_AOI_CIRCULAR => {
            let (_, _, lr, ll) = get_mbr(tg)?;
            area_modifier(tg, &label, ABOVE_MIDDLE, cs, (ll, lr), false);
        }
        tl::MFLANE if is_app6e_2(tg) => safe_lane(tg, g)?,
        tl::MFLANE => {
            let (pt0, pt1) = (px(tg, 4)?, px(tg, 2)?);
            let factors = if px(tg, 0)?.y < px(tg, 1)?.y {
                (0.5 * cs, 1.5 * cs)
            } else {
                (-0.5 * cs, -1.5 * cs)
            };
            add_dtg(tg, ABOVE_MIDDLE, factors, pt0, pt1);
        }
        _ => return Ok(false),
    }
    Ok(true)
}

/// `MFLANE` in version 16: T, AM, W and W1 stacked along the lane beside
/// its middle, as the template shows them. The text runs across the lane
/// as upstream's DTG does, so the stack is moved off the lane by half its
/// widest line.
fn safe_lane(tg: &mut Tg, g: &Geo<'_>) -> Result<(), EngineError> {
    let dash = if tg.w.is_empty() || tg.w1.is_empty() {
        ""
    } else {
        " - "
    };
    let texts = [
        tg.t.clone(),
        // A whole number of metres without Java's ".0".
        tg.am.strip_suffix(".0").unwrap_or(&tg.am).to_owned(),
        format!("{}{dash}", tg.w),
        tg.w1.clone(),
    ];
    let widest = texts.iter().map(|t| g.sw(t)).max().unwrap_or(0);
    // The fork's crossbar runs across the lane.
    let (a, b) = (px(tg, 4)?, px(tg, 2)?);
    let across = calc_distance_double(a, b);
    if across <= 0.0 {
        return Ok(());
    }
    let mid = mid_point_double(px(tg, 0)?, px(tg, 1)?, 0);
    let bar_mid = mid_point_double(a, b, 0);
    let shift = (f64::from(widest) / 2.0 + f64::from(tg.font.size) / 2.0) / across;
    let dx = mid.x - bar_mid.x + (b.x - a.x) * shift;
    let dy = mid.y - bar_mid.y + (b.y - a.y) * shift;
    let path = (Pt::new(a.x + dx, a.y + dy), Pt::new(b.x + dx, b.y + dy));
    // The lines keep the template's order within the block, as upstream
    // orders W and W1.
    for (factor, text) in [-1.5, -0.5, 0.5, 1.5].into_iter().zip(texts) {
        area_modifier(tg, &text, ABOVE_MIDDLE, factor, path, false);
    }
    Ok(())
}
