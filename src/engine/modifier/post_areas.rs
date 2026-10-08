//! The area and range-fan rules of `AddModifiers2` (Modifier2.java): the
//! circular and rectangular fire support areas, the airspace coordination
//! areas and the range fans.

use super::add::{add_area_modifier, area_modifier, area_modifier_id, integral_modifier};
use super::center_label::is_app6e_2;
use super::geo::{Geo, establishing_hq, nudged};
use super::layout::{add_dtg, get_rfa_lines, highest_point_left_of_center, remove_decimal};
use super::post::{circle_center, rectangle_center, upper_left_index};
use super::{AREA, TO_END};
use crate::engine::base::{At, EngineError, Pt};
use crate::engine::lineutility::basics::{calc_center_point_double2, mid_point_double};
use crate::engine::tactical_lines as tl;
use crate::engine::tg::Tg;

/// Java `String.split(",")`: no limit, trailing empty strings removed; a
/// string without a comma comes back whole.
pub(super) fn split_commas(text: &str) -> Vec<&str> {
    if !text.contains(',') {
        return vec![text];
    }
    let mut parts: Vec<&str> = text.split(',').collect();
    while parts.last().is_some_and(|p| p.is_empty()) {
        parts.pop();
    }
    parts
}

/// Labels `line_type` if it is one of the area or range-fan rules.
pub(super) fn post_area_labels(
    tg: &mut Tg,
    g: &mut Geo<'_>,
    line_type: i32,
) -> Result<bool, EngineError> {
    let label = g.label.clone();
    match line_type {
        tl::ACA_RECTANGULAR | tl::ACA_CIRCULAR => {
            let c =
                calc_center_point_double2(&tg.pixels, i32::try_from(tg.pixels.len()).unwrap_or(0))?;
            aca_stack(tg, g, c);
        }
        tl::FSA_CIRCULAR
        | tl::ATI_CIRCULAR
        | tl::CFFZ_CIRCULAR
        | tl::SENSOR_CIRCULAR
        | tl::CENSOR_CIRCULAR
        | tl::DA_CIRCULAR
        | tl::CFZ_CIRCULAR
        | tl::ZOR_CIRCULAR
        | tl::TBA_CIRCULAR
        | tl::TVAR_CIRCULAR
        | tl::KILLBOXBLUE_CIRCULAR
        | tl::KILLBOXPURPLE_CIRCULAR => circular_zone(tg, g)?,
        tl::FFA_CIRCULAR | tl::NFA_CIRCULAR | tl::RFA_CIRCULAR => {
            let c = mid_point_double(tg.pixels.at(0)?, tg.pixels.at(51)?, 0);
            rfa_labels(tg, g, c);
        }
        tl::FFA_RECTANGULAR | tl::NFA_RECTANGULAR | tl::RFA_RECTANGULAR => {
            let c = rectangle_center(tg)?;
            rfa_labels(tg, g, c);
        }
        tl::KILLBOXBLUE_RECTANGULAR
        | tl::KILLBOXPURPLE_RECTANGULAR
        | tl::FSA_RECTANGULAR
        | tl::ATI_RECTANGULAR
        | tl::CFFZ_RECTANGULAR
        | tl::SENSOR_RECTANGULAR
        | tl::CENSOR_RECTANGULAR
        | tl::DA_RECTANGULAR
        | tl::CFZ_RECTANGULAR
        | tl::ZOR_RECTANGULAR
        | tl::TBA_RECTANGULAR
        | tl::TVAR_RECTANGULAR => rectangular_zone(tg, g)?,
        tl::PAA_RECTANGULAR => paa_rectangular(tg, g, &label)?,
        tl::PAA_CIRCULAR => paa_circular(tg, g, &label)?,
        tl::RANGE_FAN => range_fan(tg)?,
        _ => return Ok(false),
    }
    Ok(true)
}

/// The airspace coordination area's stack of text, upright at `c`. The
/// version 16 template has T2 in the second line and H after "Grids".
fn aca_stack(tg: &mut Tg, g: &Geo<'_>, c: Pt) {
    let cs = 1.0;
    let text = format!("{}{}{}", g.label, g.t_space, tg.t);
    area_modifier(tg, &text, AREA, -3.0 * cs, (c, c), false);
    let (second, second_id, grid, eff) = if is_app6e_2(tg) {
        (tg.t2.clone(), "T2", format!("Grids {}", tg.h), "EFF:")
    } else {
        (
            tg.t1.clone(),
            "T1",
            format!("GRID {}", tg.location()),
            "EFF",
        )
    };
    super::add::add_modifier2(tg, &second, AREA, -2.0 * cs, (c, c), false, Some(second_id));
    area_modifier_id(tg, &format!("MIN ALT: {}", tg.x), AREA, -cs, c, false, "H");
    area_modifier_id(
        tg,
        &format!("MAX ALT: {}", tg.x1),
        AREA,
        0.0,
        c,
        false,
        "H1",
    );
    area_modifier_id(tg, &grid, AREA, cs, c, false, "H2");
    area_modifier_id(
        tg,
        &format!("{eff} {}{}", tg.w, g.w_dash),
        AREA,
        2.0 * cs,
        c,
        false,
        "W",
    );
    area_modifier_id(tg, &tg.w1.clone(), AREA, 3.0 * cs, c, false, "W1");
}

/// A circular zone: label over name at the centre, the DTG at the upper
/// left of the circle.
fn circular_zone(tg: &mut Tg, g: &Geo<'_>) -> Result<(), EngineError> {
    let cs = 1.0;
    let c = circle_center(tg)?;
    area_modifier(tg, &g.label, AREA, -0.5 * cs, (c, c), false);
    let name = tg.t.clone();
    area_modifier(tg, &name, AREA, 0.5 * cs, (c, c), false);
    let index = usize::try_from(upper_left_index(tg.pixels.len())).unwrap_or(0);
    let pos = tg.pixels.at(index)?;
    let path = (pos, nudged(pos, 0.001));
    area_modifier(
        tg,
        &format!("{}{}", tg.w, g.w_dash),
        TO_END,
        -2.0 * cs,
        path,
        false,
    );
    area_modifier(tg, &tg.w1.clone(), TO_END, -cs, path, false);
    Ok(())
}

/// A rectangular zone: label over name upright at the centre, the DTG at
/// the highest point left of centre.
fn rectangular_zone(tg: &mut Tg, g: &Geo<'_>) -> Result<(), EngineError> {
    let cs = 1.0;
    let c = rectangle_center(tg)?;
    area_modifier(tg, &g.label, AREA, -0.5 * cs, (c, c), false);
    let name = tg.t.clone();
    area_modifier(tg, &name, AREA, 0.5 * cs, (c, c), false);
    let pos = highest_point_left_of_center(&tg.pixels, c)?;
    let path = (pos, nudged(pos, 0.001));
    area_modifier(
        tg,
        &format!("{}{}", tg.w, g.w_dash),
        TO_END,
        0.5 * cs,
        path,
        false,
    );
    area_modifier(tg, &format!("{} ", tg.w1), TO_END, 1.5 * cs, path, false);
    Ok(())
}

/// The fire support areas: the label, the name and the DTG, as many lines
/// as the graphic has text for. The version 16 templates show T2 and AS in
/// place of the name.
fn rfa_labels(tg: &mut Tg, g: &Geo<'_>, c: Pt) {
    let cs = 1.0;
    let (name, lines) = if is_app6e_2(tg) {
        let name = establishing_hq(tg);
        let dtg = !tg.w.is_empty() || !tg.w1.is_empty();
        let lines = 1 + i32::from(!name.is_empty()) + i32::from(dtg);
        (name, lines)
    } else {
        (tg.t.clone(), get_rfa_lines(tg))
    };
    match lines {
        3 => {
            area_modifier(tg, &g.label, AREA, -cs, (c, c), false);
            area_modifier(tg, &name, AREA, 0.0, (c, c), false);
            add_dtg(tg, AREA, (cs, 2.0 * cs), c, c);
        }
        2 => {
            area_modifier(tg, &g.label, AREA, -0.5 * cs, (c, c), false);
            if name.is_empty() {
                add_dtg(tg, AREA, (0.5 * cs, 1.5 * cs), c, c);
            } else {
                area_modifier(tg, &name, AREA, 0.5 * cs, (c, c), false);
            }
        }
        _ => area_modifier(tg, &g.label, AREA, 0.0, (c, c), false),
    }
}

/// The name and DTG of the position area for artillery, below its labels.
fn paa_text(tg: &mut Tg, g: &Geo<'_>, c: Pt) {
    let cs = 1.0;
    let name = tg.t.clone();
    match get_rfa_lines(tg) {
        3 => {
            if g.settings.group_modifiers {
                let text = super::group_strings::build_area_group_string(tg, &g.label);
                area_modifier(tg, &text, AREA, 0.0, (c, c), false);
            } else {
                area_modifier(tg, &name, AREA, -1.5, (c, c), false);
                add_dtg(tg, AREA, (0.5 * cs, 1.5 * cs), c, c);
            }
        }
        2 => {
            if name.is_empty() {
                add_dtg(tg, AREA, (0.0, cs), c, c);
            } else {
                area_modifier(tg, &name, AREA, 0.0, (c, c), false);
            }
        }
        _ => {}
    }
}

/// `PAA_RECTANGULAR`: the label on all four sides, then the name and DTG.
fn paa_rectangular(tg: &mut Tg, g: &Geo<'_>, label: &str) -> Result<(), EngineError> {
    let cs = 1.0;
    for (a, b) in [(0, 1), (1, 2), (2, 3), (3, 0)] {
        let mid = mid_point_double(tg.pixels.at(a)?, tg.pixels.at(b)?, 0);
        area_modifier(tg, label, AREA, -0.5 * cs, (mid, mid), true);
    }
    let c = rectangle_center(tg)?;
    paa_text(tg, g, c);
    Ok(())
}

/// `PAA_CIRCULAR`: the label at four points of the circle, then the name
/// and DTG.
fn paa_circular(tg: &mut Tg, g: &Geo<'_>, label: &str) -> Result<(), EngineError> {
    let cs = 1.0;
    let n = i32::try_from(tg.pixels.len()).unwrap_or(0);
    for i in 0..4 {
        let at = n / 4 * i;
        integral_modifier(tg, label, AREA, -0.5 * cs, (at, at), false)?;
    }
    let far = tg.pixels.at((f64::from(n) / 2.0 + 0.5) as usize)?;
    let c = mid_point_double(tg.pixels.at(0)?, far, 0);
    paa_text(tg, g, c);
    Ok(())
}

/// `RANGE_FAN`: the altitude and range at each ring's label point, 102
/// pixels apart.
fn range_fan(tg: &mut Tg) -> Result<(), EngineError> {
    let at = |tg: &Tg, j: usize| tg.pixels.get(j * 102 + 25).copied();
    let altitudes = tg.x.clone();
    for (j, alt) in split_commas(&altitudes).into_iter().enumerate() {
        if let Some(p) = at(tg, j) {
            add_area_modifier(tg, &format!("ALT {alt}"), AREA, 0.0, (p, p));
        }
    }
    if !tg.hide_optional_labels {
        let ranges = tg.am.clone();
        for (j, range) in split_commas(&ranges).into_iter().enumerate() {
            if let Some(p) = at(tg, j) {
                let text = if j == 0 {
                    format!("MIN RG {}", remove_decimal(range)?)
                } else {
                    format!("MAX RG ({j}) {}", remove_decimal(range)?)
                };
                add_area_modifier(tg, &text, AREA, -1.0, (p, p));
            }
        }
    }
    Ok(())
}
