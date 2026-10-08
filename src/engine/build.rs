//! Port of mil-sym-java `clsRenderer.createTGLightFromMilStdSymbol` together
//! with the parts of `MultiPointHandler.populateModifiers` and the
//! `MilStdSymbol` constructor that decide what it reads: the line type,
//! control points, colours, line width and the string modifier fields of a
//! graphic, ready for the render pipeline.
//!
//! Upstream converts geographic control points to pixels here and measures
//! air corridor widths by projecting a point 10 km north; this takes pixel
//! points and the ground metres per pixel instead. Not ported:
//! `canRenderMultiPoint`'s point and amplifier checks (the catalog does
//! those), `interceptAndAdjustCode`, and the basic-shape line types
//! (`BS_*`, `PBS_*`, `BBS_*`), which have no symbol code.

mod colors;
mod corridor;
mod fans;
mod text;

#[cfg(test)]
mod tests;

use crate::engine::api::Input;
use crate::engine::base::{EngineError, Pt};
use crate::engine::settings::Settings;
use crate::engine::tactical_lines::*;
use crate::engine::tg::Tg;
use crate::engine::tg_utility::line_classes::is_closed_polygon;
use crate::engine::tg_utility::points::close_polygon;
use crate::style::Rgba;
use text::altitude_label;

/// Upstream's default line width.
const DEFAULT_LINE_WIDTH: i32 = 3;
/// `SymbolID` basic code of the sector range fan: symbol set 25, entity 242200.
const SECTOR_RANGE_FAN_ENTITY: u32 = 242_200;

/// The attributes `populateModifiers` takes from the caller's attribute map.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Overrides {
    /// `LineColor`: also the text colour unless `text_color` is set.
    pub(crate) line_color: Option<Rgba>,
    /// `FillColor`.
    pub(crate) fill_color: Option<Rgba>,
    /// `TextColor`.
    pub(crate) text_color: Option<Rgba>,
    /// `LineWidth`, used when positive.
    pub(crate) line_width: Option<i32>,
}

/// The AM, AN and X lists as `MilStdSymbol` holds them: absent when the
/// graphic gives none. X holds the finished altitude labels.
#[derive(Clone, Debug, Default)]
struct Amps {
    am: Option<Vec<f64>>,
    an: Option<Vec<f64>>,
    x: Option<Vec<String>>,
}

/// `populateModifiers`' list handling, including its rule that a sector
/// range fan with fewer distances than sectors plus one starts at range 0.
fn amplifiers(input: &Input<'_>) -> Amps {
    let m = input.modifiers;
    let non_empty = |v: &Vec<f64>| (!v.is_empty()).then(|| v.clone());
    let mut amps = Amps {
        am: non_empty(&m.distances_m),
        an: non_empty(&m.azimuths_deg),
        x: (!m.altitudes.is_empty()).then(|| m.altitudes.iter().map(altitude_label).collect()),
    };
    if input.symbol.symbol_set() == 25 && input.symbol.entity().get() == SECTOR_RANGE_FAN_ENTITY {
        if let (Some(an), Some(am)) = (&amps.an, &mut amps.am) {
            let first_not_zero = am.first().is_some_and(|d| *d != 0.0);
            if am.len() < an.len() / 2 + 1 && first_not_zero {
                am.insert(0, 0.0);
            }
        }
    }
    amps
}

/// Builds the graphic for `input`: upstream `populateModifiers` followed by
/// `createTGLightFromMilStdSymbol`. Like upstream, a failure while deriving
/// the amplifier strings stops there and leaves the graphic as far as it got.
pub(crate) fn build_tg(
    input: &Input<'_>,
    settings: &Settings,
    overrides: &Overrides,
) -> Result<Tg, EngineError> {
    let mut tg = Tg::new(settings);
    tg.set_symbol_id(input.symbol.as_str(), &|_| None)?;
    tg.line_type = input.line_type;
    tg.pixels.clone_from(&input.pixels);
    set_appearance(&mut tg, input, overrides);
    set_text_fields(&mut tg, input);
    close_areas(&mut tg);
    // The amplifier strings have no later consumer that tolerates a half
    // result, but upstream keeps the graphic when they fail.
    amplifier_strings(&mut tg, input).ok();
    Ok(tg)
}

fn set_appearance(tg: &mut Tg, input: &Input<'_>, overrides: &Overrides) {
    let symbol = input.symbol;
    let line = overrides
        .line_color
        .unwrap_or_else(|| colors::default_line_color(symbol));
    tg.line_color = Some(line);
    tg.text_color = Some(
        overrides
            .text_color
            .or(overrides.line_color)
            .unwrap_or_else(|| colors::line_color_of_affiliation(symbol)),
    );
    tg.fill_color = overrides
        .fill_color
        .or_else(|| colors::default_fill_color(symbol));
    tg.line_thickness = overrides
        .line_width
        .filter(|w| *w > 0)
        .unwrap_or(DEFAULT_LINE_WIDTH);
    // `MilStdSymbol`'s dash and hatch flags are static and true; the dash
    // pattern then stays on the stroke, and hatch is an image attribute.
    tg.use_dash_array = true;
    tg.use_hatch_fill = true;
    tg.hide_optional_labels = false;
}

fn set_text_fields(tg: &mut Tg, input: &Input<'_>) {
    let m = input.modifiers;
    let set = |target: &mut String, v: &Option<String>| {
        if let Some(s) = v {
            s.clone_into(target);
        }
    };
    set(&mut tg.w, &m.dtg_start);
    set(&mut tg.w1, &m.dtg_end);
    set(&mut tg.h, &m.additional_info);
    set(&mut tg.h1, &m.additional_info2);
    set(&mut tg.t, &m.designation);
    set(&mut tg.t1, &m.designation2);
    set(&mut tg.t2, &m.designation3);
    set(&mut tg.v, &m.equipment_type);
    set(&mut tg.as_, &m.country);
    set(&mut tg.ap, &m.target_number);
    set(&mut tg.y, &m.location);
    set(&mut tg.n, &m.hostile);
}

/// Closes area outlines; `STRIKWARN` is two areas, each closed on its own.
fn close_areas(tg: &mut Tg) {
    if tg.line_type == STRIKWARN {
        let half = tg.pixels.len() / 2;
        let mut second: Vec<Pt> = tg.pixels.split_off(half);
        close_polygon(&mut tg.pixels);
        close_polygon(&mut second);
        tg.pixels.append(&mut second);
    } else if is_closed_polygon(tg.line_type) {
        close_polygon(&mut tg.pixels);
    }
}

/// The ROZ, WEZ and ACA families label their altitude limits with X and X1.
const ALTITUDE_LABELLED: &[i32] = &[
    ROZ,
    AARROZ,
    UAROZ,
    WEZ,
    FEZ,
    JEZ,
    FAADZ,
    HIDACZ,
    MEZ,
    LOMEZ,
    HIMEZ,
    ACA,
    ACA_RECTANGULAR,
    ACA_CIRCULAR,
    WFZ,
];

fn set_altitude_limits(tg: &mut Tg, amps: &Amps) {
    if let Some(x) = &amps.x {
        if let Some(first) = x.first() {
            first.clone_into(&mut tg.x);
        }
        if let Some(second) = x.get(1) {
            second.clone_into(&mut tg.x1);
        }
    }
}

/// The per-line-type AM/AN/X handling of `createTGLightFromMilStdSymbol`, in
/// upstream's order.
fn amplifier_strings(tg: &mut Tg, input: &Input<'_>) -> Result<(), EngineError> {
    let amps = amplifiers(input);
    let line_type = tg.line_type;
    if line_type == RANGE_FAN_SECTOR {
        fans::sector_fan(tg, &amps)?;
    } else if line_type == RADAR_SEARCH {
        fans::radar_search(tg, &amps)?;
    }
    if matches!(
        line_type,
        LAUNCH_AREA | DEFENDED_AREA_CIRCULAR | SHIP_AOI_CIRCULAR
    ) {
        fans::ellipse(tg, &amps);
    }
    if ALTITUDE_LABELLED.contains(&line_type) {
        set_altitude_limits(tg, &amps);
    } else if corridor::is_corridor(line_type) {
        corridor::corridor_widths(tg, &amps, input.meters_per_pixel);
        set_altitude_limits(tg, &amps);
    }
    if line_type == RANGE_FAN {
        fans::circular_fan(tg, &amps);
    }
    fans::first_am(tg, line_type, &amps);
    if line_type == RECTANGULAR || line_type == CUED_ACQUISITION {
        fans::rectangle(tg, &amps);
    }
    Ok(())
}
