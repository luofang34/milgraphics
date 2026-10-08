//! The area rules of `AddModifiersGeo` (Modifier2.java): labels stacked
//! around the centre of the area, with the lines of text spaced by line
//! factors.

use super::add::{add_modifier, area_modifier, area_modifier_id};
use super::geo::{Geo, nudged};
use super::layout::{
    add_dtg, add_modifier_bottom_segment, add_modifier_on_line, add_n_modifier, get_mbr,
    highest_point_left_of_center,
};
use super::{ABOVE_MIDDLE, AREA, TO_END};
use crate::engine::base::EngineError;
use crate::engine::tactical_lines as tl;
use crate::engine::tg::Tg;

/// A label at the centre with the given line factor.
fn at_center(tg: &mut Tg, g: &Geo<'_>, text: &str, line_factor: f64, is_integral: bool) {
    area_modifier(
        tg,
        text,
        AREA,
        line_factor,
        (g.center, g.center),
        is_integral,
    );
}

/// A label at the centre carrying a modifier type.
fn at_center_id(tg: &mut Tg, g: &Geo<'_>, text: &str, line_factor: f64, id: (bool, &str)) {
    area_modifier_id(tg, text, AREA, line_factor, g.center, id.0, id.1);
}

/// Labels `line_type` if it is one of the area rules.
pub(super) fn area_labels(
    tg: &mut Tg,
    g: &mut Geo<'_>,
    line_type: i32,
) -> Result<bool, EngineError> {
    if simple_areas(tg, g, line_type)?
        || zone_areas(tg, g, line_type)?
        || point_areas(tg, g, line_type)?
    {
        return Ok(true);
    }
    Ok(false)
}

/// Areas labelled with a name, label and DTG at the centre.
fn simple_areas(tg: &mut Tg, g: &mut Geo<'_>, line_type: i32) -> Result<bool, EngineError> {
    let cs = 1.0;
    let (label, name) = (g.label.clone(), tg.t.clone());
    let (c, ts, td) = (g.center, g.t_space, g.t_dash);
    match line_type {
        tl::LAUNCH_AREA | tl::DEFENDED_AREA_CIRCULAR => {
            at_center(tg, g, &format!("{label}{td}{name}"), 0.0, false);
        }
        tl::JTAA | tl::SAA | tl::SGAA => {
            add_n_modifier(tg)?;
            at_center(tg, g, &format!("{label}{td}{name}"), 0.0, false);
            add_dtg(tg, AREA, (cs, 2.0 * cs), c, c);
        }
        tl::FORT | tl::ZONE => at_center(tg, g, &name, 0.0, false),
        tl::BDZ => area_modifier(tg, &label, AREA, 0.0, (g.ends.pt0, g.ends.pt0), false),
        tl::ASSAULT
        | tl::ATKPOS
        | tl::OBJ
        | tl::NAI
        | tl::TAI
        | tl::BASE_CAMP_REVD
        | tl::GUERILLA_BASE_REVD
        | tl::ASSY
        | tl::EA
        | tl::DZ
        | tl::EZ
        | tl::LZ
        | tl::PZ
        | tl::AO => at_center(tg, g, &format!("{label}{ts}{name}"), 0.0, false),
        tl::BASE_CAMP | tl::GUERILLA_BASE => {
            at_center(tg, g, &format!("{label}{ts}{name}"), -cs, false);
            let h = tg.h.clone();
            add_modifier(tg, &h, AREA, 0.0, c, c);
            add_dtg(tg, AREA, (cs, 2.0 * cs), c, c);
            add_n_modifier(tg)?;
            let echelon = tg.echelon_symbol.clone();
            add_modifier_bottom_segment(tg, &echelon)?;
        }
        tl::GENERIC_AREA => {
            at_center(tg, g, &format!("{} {name}", tg.h), -0.5 * cs, false);
            add_dtg(tg, AREA, (0.5 * cs, 1.5 * cs), c, c);
            add_n_modifier(tg)?;
        }
        tl::AIRHEAD => {
            let (_, _, lr, ll) = get_mbr(tg)?;
            area_modifier(tg, &label, ABOVE_MIDDLE, cs, (ll, lr), false);
        }
        tl::AIRFIELD => airfield(tg, g)?,
        _ => return centered_areas(tg, g, line_type),
    }
    Ok(true)
}

/// More areas labelled at the centre.
fn centered_areas(tg: &mut Tg, g: &mut Geo<'_>, line_type: i32) -> Result<bool, EngineError> {
    let cs = 1.0;
    let (label, name) = (g.label.clone(), tg.t.clone());
    let (c, ts) = (g.center, g.t_space);
    match line_type {
        tl::AT => at_center(tg, g, &tg.ap.clone(), 0.0, false),
        tl::RECTANGULAR | tl::CIRCULAR => {
            let ap = tg.ap.clone();
            area_modifier(tg, &ap, AREA, 0.0, (g.ends.pt0, g.ends.pt0), false);
        }
        tl::PBS_CIRCLE | tl::PBS_ELLIPSE | tl::PBS_RECTANGLE | tl::BBS_POINT => {
            area_modifier(tg, &name, AREA, 0.0, (g.ends.pt0, g.ends.pt0), false);
        }
        tl::RECTANGULAR_TARGET => {
            let offset = nudged(c, f64::from(g.sw(&name)) / 2.0);
            area_modifier(tg, &name, AREA, -cs, (offset, offset), false);
        }
        tl::SMOKE => {
            at_center(tg, g, &tg.ap.clone(), -cs, false);
            at_center(tg, g, &label, 0.0, false);
            add_dtg(tg, AREA, (cs, 2.0 * cs), c, c);
        }
        tl::BS_AREA | tl::BBS_AREA => at_center(tg, g, &name, 0.0, false),
        tl::BSA | tl::DSA | tl::CSA | tl::RSA => at_center(tg, g, &label, 0.0, false),
        tl::RIP | tl::BOMB | tl::TGMF => at_center(tg, g, &label, 0.0, true),
        tl::LAA => at_center(tg, g, &label, -cs, false),
        tl::BATTLE | tl::STRONG => {
            at_center(tg, g, &name, 0.0, false);
            let echelon = tg.echelon_symbol.clone();
            add_modifier_bottom_segment(tg, &echelon)?;
        }
        tl::PNO => {
            at_center(tg, g, &format!("{label}{ts}{name}"), 0.0, false);
            let echelon = tg.echelon_symbol.clone();
            add_modifier_bottom_segment(tg, &echelon)?;
            add_n_modifier(tg)?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}

/// `AIRFIELD`: H to the right of the bounding box.
fn airfield(tg: &mut Tg, g: &Geo<'_>) -> Result<(), EngineError> {
    let (_, ur, lr, _) = get_mbr(tg)?;
    let mut pt0 = g.ends.pt0;
    pt0.x = ur.x + f64::from(g.sw(&tg.h) / 2) + 1.0;
    pt0.y = (ur.y + lr.y) / 2.0 - f64::from(tg.font.size);
    let h = tg.h.clone();
    area_modifier(tg, &h, AREA, 1.0, (pt0, pt0), false);
    Ok(())
}

/// The fixed-text stacks of the holding, refugee and prisoner areas and the
/// fire support areas.
fn point_areas(tg: &mut Tg, g: &mut Geo<'_>, line_type: i32) -> Result<bool, EngineError> {
    let cs = 1.0;
    let (label, name) = (g.label.clone(), tg.t.clone());
    let c = g.center;
    match line_type {
        tl::DHA_REVD | tl::EPW | tl::RHA => {
            let first = match line_type {
                tl::DHA_REVD => "DETAINEE",
                tl::EPW => "EPW",
                _ => "REFUGEE",
            };
            at_center(tg, g, first, -1.5 * cs, false);
            at_center(tg, g, "HOLDING", -0.5 * cs, false);
            at_center(tg, g, "AREA", 0.5 * cs, false);
            at_center(tg, g, &name, 1.5 * cs, false);
        }
        tl::UXO => add_modifier_on_line(tg, "UXO", true)?,
        tl::GENERAL => add_n_modifier(tg)?,
        tl::DHA | tl::KILL_ZONE | tl::FARP => {
            at_center(tg, g, &label, -0.5 * cs, false);
            at_center(tg, g, &name, 0.5 * cs, false);
        }
        tl::DEPICT => {
            get_mbr(tg)?;
            add_n_modifier(tg)?;
        }
        tl::FFA | tl::RFA | tl::NFA => {
            at_center(tg, g, &label, -cs, false);
            at_center(tg, g, &name, 0.0, false);
            add_dtg(tg, AREA, (cs, 2.0 * cs), c, c);
        }
        tl::PAA => {
            add_modifier_on_line(tg, "PAA", false)?;
            at_center(tg, g, &name, -0.5 * cs, false);
            add_dtg(tg, AREA, (0.5 * cs, 1.5 * cs), c, c);
        }
        tl::FSA => {
            at_center(
                tg,
                g,
                &format!("{label}{}{name}", g.t_space),
                -0.5 * cs,
                false,
            );
            add_dtg(tg, AREA, (0.5 * cs, 1.5 * cs), c, c);
        }
        tl::ACA => aca(tg, g)?,
        _ => return Ok(false),
    }
    Ok(true)
}

/// `ACA`: the airspace coordination area's stack of text.
fn aca(tg: &mut Tg, g: &Geo<'_>) -> Result<(), EngineError> {
    let cs = 1.0;
    let (label, name) = (g.label.clone(), tg.t.clone());
    at_center(
        tg,
        g,
        &format!("{label}{}{name}", g.t_space),
        -3.0 * cs,
        false,
    );
    let t1 = tg.t1.clone();
    at_center(tg, g, &t1, -2.0 * cs, false);
    at_center_id(tg, g, &format!("MIN ALT: {}", tg.x), -cs, (false, "H"));
    at_center_id(tg, g, &format!("MAX ALT: {}", tg.x1), 0.0, (false, "H1"));
    at_center_id(tg, g, &format!("GRID {}", tg.location()), cs, (false, "H2"));
    at_center_id(
        tg,
        g,
        &format!("EFF {}{}", tg.w, g.w_dash),
        2.0 * cs,
        (false, "W"),
    );
    at_center_id(tg, g, &tg.w1.clone(), 3.0 * cs, (false, "W1"));
    Ok(())
}

/// The altitude, time and zone areas: label over name, DTG at the top
/// left, or the stacked airspace zones.
fn zone_areas(tg: &mut Tg, g: &mut Geo<'_>, line_type: i32) -> Result<bool, EngineError> {
    let cs = 1.0;
    let (label, name) = (g.label.clone(), tg.t.clone());
    match line_type {
        tl::ATI
        | tl::CFFZ
        | tl::CFZ
        | tl::TBA
        | tl::TVAR
        | tl::ZOR
        | tl::DA
        | tl::SENSOR
        | tl::CENSOR
        | tl::KILLBOXBLUE
        | tl::KILLBOXPURPLE => {
            at_center(tg, g, &label, -0.5 * cs, false);
            at_center(tg, g, &name, 0.5 * cs, false);
            // The DTG goes at the highest point left of centre, upright.
            let highest = highest_point_left_of_center(&tg.pixels, g.center)?;
            let path = (highest, nudged(highest, 0.001));
            let dtg = format!("{}{}", tg.w, g.w_dash);
            add_to_end_id(tg, &dtg, 0.5 * cs, path, "W");
            let dtg1 = tg.w1.clone();
            add_to_end_id(tg, &dtg1, 1.5 * cs, path, "W1");
        }
        tl::WFZ_REVD => time_zone(tg, g, &[-1.5, -0.5, 0.5, 1.5]),
        tl::WFZ => {
            time_zone(tg, g, &[-2.5, -1.5, -0.5, 0.5]);
            at_center_id(tg, g, &format!("MIN ALT: {}", tg.x), 1.5 * cs, (false, "H"));
            at_center_id(tg, g, &format!("MAX ALT: {}", tg.x1), 2.5, (false, "H1"));
        }
        tl::OBSFAREA => {
            at_center(tg, g, &label, -1.5 * cs, false);
            at_center(tg, g, &name, -0.5 * cs, false);
            at_center_id(
                tg,
                g,
                &format!("{}{}", tg.w, g.w_dash),
                0.5 * cs,
                (false, "W"),
            );
            at_center_id(tg, g, &tg.w1.clone(), 1.5 * cs, (false, "W1"));
        }
        tl::OBSAREA => {
            at_center(tg, g, &name, -cs, true);
            at_center_id(tg, g, &format!("{}{}", tg.w, g.w_dash), 0.0, (true, "W"));
            at_center_id(tg, g, &tg.w1.clone(), cs, (true, "W1"));
        }
        tl::ROZ
        | tl::AARROZ
        | tl::UAROZ
        | tl::WEZ
        | tl::FEZ
        | tl::JEZ
        | tl::FAADZ
        | tl::HIDACZ
        | tl::MEZ
        | tl::LOMEZ
        | tl::HIMEZ => airspace_zone(tg, g),
        _ => return Ok(false),
    }
    Ok(true)
}

/// A `toEnd` label recording its modifier type.
fn add_to_end_id(
    tg: &mut Tg,
    text: &str,
    line_factor: f64,
    path: (crate::engine::base::Pt, crate::engine::base::Pt),
    id: &str,
) {
    super::add::add_modifier2(tg, text, TO_END, line_factor, path, false, Some(id));
}

/// Label, name, "TIME FROM" and "TIME TO" at the given line factors, all
/// integral.
fn time_zone(tg: &mut Tg, g: &Geo<'_>, factors: &[f64; 4]) {
    let cs = 1.0;
    let (label, name) = (g.label.clone(), tg.t.clone());
    at_center(tg, g, &label, factors[0] * cs, true);
    at_center(tg, g, &name, factors[1] * cs, true);
    at_center_id(
        tg,
        g,
        &format!("TIME FROM: {}", tg.w),
        factors[2] * cs,
        (true, "W"),
    );
    at_center_id(
        tg,
        g,
        &format!("TIME TO: {}", tg.w1),
        factors[3] * cs,
        (true, "W1"),
    );
}

/// The missile and air defence zones: label, name, altitudes and times.
fn airspace_zone(tg: &mut Tg, g: &Geo<'_>) {
    let (label, name) = (g.label.clone(), tg.t.clone());
    at_center_id(tg, g, &label, -2.5, (false, ""));
    at_center_id(tg, g, &name, -1.5, (false, "T"));
    at_center_id(tg, g, &format!("MIN ALT: {}", tg.x), -0.5, (false, "H"));
    at_center_id(tg, g, &format!("MAX ALT: {}", tg.x1), 0.5, (false, "H1"));
    at_center_id(tg, g, &format!("TIME FROM: {}", tg.w), 1.5, (false, "W"));
    at_center_id(tg, g, &format!("TIME TO: {}", tg.w1), 2.5, (false, "W1"));
}
