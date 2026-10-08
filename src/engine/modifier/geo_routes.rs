//! The route, corridor and axis rules of `AddModifiersGeo` (Modifier2.java):
//! supply routes labelled on every segment long enough for the text, air
//! corridors, axes of advance, screens and bearing lines.

use super::add::{add_modifier, add_modifier2, area_modifier, count, integral_modifier, px};
use super::geo::{Geo, req};
use super::layout::{add_dtg, remove_decimal};
use super::{ABOVE_MIDDLE, AREA};
use crate::engine::base::{At, EngineError, Pt};
use crate::engine::lineutility::basics::{calc_distance_double, mid_point_double};
use crate::engine::lineutility::extend::{extend_along_line_double2, extend_directed_line};
use crate::engine::lineutility::slope::calc_true_slope_double;
use crate::engine::tactical_lines as tl;
use crate::engine::tg::Tg;

/// Upstream `arraysupport.SupplyRouteArrowSide`: the side of the segment
/// its direction arrow is drawn on.
pub(super) fn supply_route_arrow_side(pt0: Pt, pt1: Pt) -> i32 {
    let (vertical, m) = calc_true_slope_double(pt0, pt1);
    if pt0.x < pt1.x {
        if m < 1.0 { 2 } else { 1 }
    } else if pt0.x > pt1.x {
        if m < 1.0 { 3 } else { 0 }
    } else if vertical == 0 && pt0.y > pt1.y {
        0
    } else if vertical == 0 {
        1
    } else {
        0
    }
}

/// Labels `line_type` if it is one of the route, corridor or axis rules.
pub(super) fn route_labels(
    tg: &mut Tg,
    g: &mut Geo<'_>,
    line_type: i32,
) -> Result<bool, EngineError> {
    let cs = 1.0;
    let label = g.label.clone();
    match line_type {
        tl::MSR_ONEWAY
        | tl::ASR_ONEWAY
        | tl::TRAFFIC_ROUTE_ONEWAY
        | tl::MSR_TWOWAY
        | tl::ASR_TWOWAY
        | tl::MSR_ALT
        | tl::ASR_ALT
        | tl::TRAFFIC_ROUTE_ALT => one_way_route(tg, g, line_type)?,
        tl::MSR | tl::ASR | tl::TRAFFIC_ROUTE => route(tg, g)?,
        tl::TRIP => trip(tg, g)?,
        tl::AC | tl::LLTR | tl::MRR | tl::SL | tl::TC | tl::SAAFR | tl::SC => air_corridor(tg, g)?,
        tl::SPT
        | tl::FRONTAL_ATTACK
        | tl::TURNING_MOVEMENT
        | tl::MOVEMENT_TO_CONTACT
        | tl::AIRAOA
        | tl::AAAAA
        | tl::MAIN => axis(tg, g)?,
        tl::SCREEN | tl::COVER | tl::GUARD => screen(tg, g)?,
        tl::ASLTXING | tl::GAP => crossing(tg, line_type == tl::GAP)?,
        tl::BEARING_J
        | tl::BEARING_RDF
        | tl::BEARING
        | tl::ELECTRO
        | tl::BEARING_EW
        | tl::ACOUSTIC
        | tl::ACOUSTIC_AMB
        | tl::TORPEDO
        | tl::OPTICAL => {
            let mid = mid_point_double(g.ends.pt0, req(g.ends.pt1)?, 0);
            area_modifier(tg, &label, ABOVE_MIDDLE, 0.0, (mid, mid), true);
            let pt1 = req(g.ends.pt1)?;
            let pt3 = extend_directed_line(g.ends.pt0, pt1, pt1, 3, f64::from(tg.font.size) / 2.0);
            let h = tg.h.clone();
            area_modifier(tg, &h, ABOVE_MIDDLE, cs, (pt3, pt3), true);
        }
        _ => return Ok(false),
    }
    Ok(true)
}

/// A route's name label, with the arrow's side deciding which side of the
/// segment the label goes on, and "ALT" where the route is alternating.
fn route_segment_labels(
    tg: &mut Tg,
    g: &Geo<'_>,
    pts: (Pt, Pt),
    arrow: (f64, bool),
) -> Result<(), EngineError> {
    let cs = 1.0;
    let (arrow_offset, is_alt) = arrow;
    let (mut pt0, mut pt1) = pts;
    let side = supply_route_arrow_side(pt0, pt1);
    let text = format!("{}{}{}", g.label, g.t_space, tg.t);
    if side == 1 || side == 2 {
        // The arrow is shifted by the DPI-scaled offset; so is the label.
        pt0 = extend_directed_line(pt1, pt0, pt0, side, arrow_offset);
        pt1 = extend_directed_line(pt1, pt0, pt1, side, arrow_offset);
        add_modifier(tg, &text, ABOVE_MIDDLE, -1.7 * cs, pt0, pt1);
        if is_alt {
            add_modifier(tg, "ALT", ABOVE_MIDDLE, 0.0, pt0, pt1);
        }
    } else {
        add_modifier(tg, &text, ABOVE_MIDDLE, -0.7 * cs, pt0, pt1);
        if is_alt {
            pt0 = extend_directed_line(pt1, pt0, pt0, side, arrow_offset);
            pt1 = extend_directed_line(pt1, pt0, pt1, side, arrow_offset);
            add_modifier(tg, "ALT", ABOVE_MIDDLE, 0.0, pt0, pt1);
        }
    }
    Ok(())
}

/// Index of the point with the largest (`north`) or smallest y among all
/// but the last pixel, the last such point winning ties as upstream's `>=`
/// and `<=` do.
fn extreme_y_index(tg: &Tg, take_largest: bool, upto: usize) -> Result<i32, EngineError> {
    let mut best = tg.pixels.at(0)?;
    let mut index = 0;
    for j in 0..upto {
        let p = tg.pixels.at(j)?;
        if (take_largest && p.y >= best.y) || (!take_largest && p.y <= best.y) {
            best = p;
            index = j;
        }
    }
    Ok(i32::try_from(index).unwrap_or(i32::MAX))
}

/// One-way, two-way and alternating routes.
fn one_way_route(tg: &mut Tg, g: &Geo<'_>, line_type: i32) -> Result<(), EngineError> {
    let cs = 1.0;
    let text = format!("{}{}{}", g.label, g.t_space, tg.t);
    let mut width = (1.5 * f64::from(g.sw(&text))) as i32;
    let dpi = g.settings.dpi_scale_factor();
    let two_way = matches!(line_type, tl::MSR_TWOWAY | tl::ASR_TWOWAY);
    let arrow_offset = if two_way { 25.0 * dpi } else { 10.0 * dpi };
    let is_alt = matches!(line_type, tl::MSR_ALT | tl::ASR_ALT | tl::TRAFFIC_ROUTE_ALT);
    if is_alt {
        width = width.max((1.5 * f64::from(g.sw("ALT"))) as i32);
    }
    if !g.settings.two_label_only {
        let mut found = false;
        for j in 0..count(tg) - 1 {
            let pts = (px(tg, j)?, px(tg, j + 1)?);
            if calc_distance_double(pts.0, pts.1) >= f64::from(width) {
                route_segment_labels(tg, g, pts, (arrow_offset, is_alt))?;
                found = true;
            }
        }
        if !found {
            let seg = g.middle_segment;
            let pts = (px(tg, seg)?, px(tg, seg + 1)?);
            route_segment_labels(tg, g, pts, (arrow_offset, is_alt))?;
        }
    } else {
        // One label at the north-most and one at the south-most point.
        let last = tg.pixels.len().saturating_sub(1);
        let north = extreme_y_index(tg, true, last)?;
        let south = extreme_y_index(tg, false, last)?;
        for index in [Some(north), (north != south).then_some(south)]
            .into_iter()
            .flatten()
        {
            integral_modifier(
                tg,
                &text,
                ABOVE_MIDDLE,
                -1.7 * cs,
                (index, index + 1),
                false,
            )?;
            if is_alt {
                integral_modifier(
                    tg,
                    "ALT",
                    ABOVE_MIDDLE,
                    -0.7 * cs,
                    (index, index + 1),
                    false,
                )?;
            }
        }
    }
    Ok(())
}

/// `MSR`, `ASR` and `TRAFFIC_ROUTE`: the name on every segment long enough.
fn route(tg: &mut Tg, g: &Geo<'_>) -> Result<(), EngineError> {
    let cs = 1.0;
    let text = format!("{}{}{}", g.label, g.t_space, tg.t);
    if !g.settings.two_label_only {
        let width = (1.5 * f64::from(g.sw(&text))) as i32;
        let mut found = false;
        for j in 0..count(tg) - 1 {
            let dist = calc_distance_double(px(tg, j)?, px(tg, j + 1)?);
            if dist >= f64::from(width) {
                integral_modifier(tg, &text, ABOVE_MIDDLE, -cs, (j, j + 1), false)?;
                found = true;
            }
        }
        if !found {
            let seg = g.middle_segment;
            integral_modifier(tg, &text, ABOVE_MIDDLE, -cs, (seg, seg + 1), false)?;
        }
    } else {
        let n = tg.pixels.len();
        let mut seg = extreme_y_index(tg, true, n)?;
        let mut seg2 = extreme_y_index(tg, false, n)?;
        let last = count(tg) - 1;
        if seg == last {
            seg -= 1;
        }
        if seg2 == last {
            seg2 -= 1;
        }
        if seg == seg2 {
            seg2 -= 1;
        }
        integral_modifier(tg, &text, ABOVE_MIDDLE, 0.0, (seg, seg + 1), false)?;
        integral_modifier(tg, &text, ABOVE_MIDDLE, 0.0, (seg2, seg2 + 1), false)?;
    }
    Ok(())
}

/// `TRIP`: the label at the middle of every segment long enough.
fn trip(tg: &mut Tg, g: &Geo<'_>) -> Result<(), EngineError> {
    let cs = 1.0;
    let width = 1.5 * f64::from(g.sw(&g.label));
    let mut found = false;
    for j in 0..count(tg) - 1 {
        let (pt0, pt1) = (px(tg, j)?, px(tg, j + 1)?);
        let mid = mid_point_double(pt0, pt1, 0);
        if calc_distance_double(pt0, pt1) > f64::from(width as i32) {
            add_modifier2(
                tg,
                &g.label,
                ABOVE_MIDDLE,
                -0.7 * cs,
                (mid, mid),
                false,
                None,
            );
            found = true;
        }
    }
    if !found {
        let seg = g.middle_segment;
        let mid = mid_point_double(px(tg, seg)?, px(tg, seg + 1)?, 0);
        add_modifier2(
            tg,
            &g.label,
            ABOVE_MIDDLE,
            -0.7 * cs,
            (mid, mid),
            false,
            None,
        );
    }
    Ok(())
}

/// Air corridors: a stacked block of the corridor's data over a segment.
fn air_corridor(tg: &mut Tg, g: &Geo<'_>) -> Result<(), EngineError> {
    let cs = 1.0;
    let seg = g.middle_segment;
    let path = (seg, seg + 1);
    let rows = [
        (format!("Name: {}", tg.t), -7.0),
        (format!("Width: {}", remove_decimal(&tg.am)?), -6.0),
        (format!("Min Alt: {}", tg.x), -5.0),
        (format!("Max Alt: {}", tg.x1), -4.0),
        (format!("DTG Start: {}", tg.w), -3.0),
        (format!("DTG End: {}", tg.w1), -2.0),
        (format!("{}{}{}", g.label, g.t_space, tg.t), 0.0),
    ];
    for (text, factor) in rows {
        integral_modifier(tg, &text, ABOVE_MIDDLE, factor * cs, path, false)?;
    }
    Ok(())
}

/// Axes of advance: DTG and name over the first segment with room.
fn axis(tg: &mut Tg, g: &Geo<'_>) -> Result<(), EngineError> {
    let cs = 1.0;
    let (pt0, pt1) = (g.ends.pt0, req(g.ends.pt1)?);
    let (dtg, dtg1, name) = (format!("{}{}", tg.w, g.w_dash), tg.w1.clone(), tg.t.clone());
    let size = tg.pixels.len();
    if size == 3 || size == 4 {
        let mid = if size == 3 {
            mid_point_double(pt0, pt1, 0)
        } else {
            mid_point_double(pt1, req(g.pt2)?, 0)
        };
        area_modifier(tg, &dtg, ABOVE_MIDDLE, 0.0, (mid, mid), false);
        area_modifier(tg, &dtg1, ABOVE_MIDDLE, cs, (mid, mid), false);
        area_modifier(tg, &name, ABOVE_MIDDLE, 2.0 * cs, (mid, mid), false);
    } else {
        let pt2 = req(g.pt2)?;
        let mid = mid_point_double(pt1, pt2, 0);
        area_modifier(tg, &dtg, ABOVE_MIDDLE, -cs / 2.0, (mid, mid), false);
        area_modifier(tg, &dtg1, ABOVE_MIDDLE, cs / 2.0, (mid, mid), false);
        let mid = mid_point_double(pt2, req(g.pt3)?, 0);
        area_modifier(tg, &name, ABOVE_MIDDLE, -cs / 2.0, (mid, mid), false);
    }
    Ok(())
}

/// `SCREEN`, `COVER` and `GUARD`: the label at both flanks.
fn screen(tg: &mut Tg, g: &Geo<'_>) -> Result<(), EngineError> {
    let label = g.label.clone();
    let (a, b) = if tg.pixels.len() == 4 {
        (px(tg, 1)?, px(tg, 2)?)
    } else {
        let height = f64::from((0.5 * f64::from(tg.font.size)) as i32);
        let (p0, p1, p2) = (px(tg, 0)?, px(tg, 1)?, px(tg, 2)?);
        let angle0 = (p0.y - p1.y).atan2(p0.x - p1.x);
        let angle1 = (p0.y - p2.y).atan2(p0.x - p2.x);
        let mut a = p0;
        a.x -= 30.0 * angle0.cos();
        a.y -= 30.0 * angle0.sin() + height;
        let mut b = p0;
        b.x -= 30.0 * angle1.cos();
        b.y -= 30.0 * angle1.sin() + height;
        (a, b)
    };
    area_modifier(tg, &label, AREA, 0.0, (a, a), true);
    area_modifier(tg, &label, AREA, 0.0, (b, b), true);
    Ok(())
}

/// `ASLTXING` and `GAP`: DTG (and for the gap the name) along the crossing,
/// 20 pixels outside it.
fn crossing(tg: &mut Tg, is_gap: bool) -> Result<(), EngineError> {
    let cs = 1.0;
    let (p0, p1, p2, p3) = (px(tg, 0)?, px(tg, 1)?, px(tg, 2)?, px(tg, 3)?);
    let (pt0, pt1, pt2, pt3) = if p1.y > p0.y {
        (p1, p3, p0, p2)
    } else {
        (p0, p2, p1, p3)
    };
    let pt2 = extend_along_line_double2(pt0, pt2, -20.0);
    let pt3 = extend_along_line_double2(pt1, pt3, -20.0);
    if is_gap {
        let name = tg.t.clone();
        area_modifier(tg, &name, ABOVE_MIDDLE, 0.0, (pt0, pt1), false);
    }
    add_dtg(tg, ABOVE_MIDDLE, (0.0, cs), pt2, pt3);
    Ok(())
}
