//! `scaleModifiers` and `getChange1Height` of Modifier2.java: the line
//! factors of an area's labels are stretched when the area is much taller
//! than they are, and the labels that no longer fit are replaced by an
//! ellipsis.
//!
//! Upstream's `RemoveModifiers`, which hides labels outside the area, runs
//! for the CPOF clients only and so has no counterpart here.

use super::{ABOVE_MIDDLE, ABOVE_START_INSIDE, AREA, TO_END, layout::get_mbr};
use crate::engine::base::EngineError;
use crate::engine::line_type::classes::is_change1_area;
use crate::engine::settings::Settings;
use crate::engine::tactical_lines as tl;
use crate::engine::tg::{ModifierLabel, Tg};
use crate::engine::tg_utility::line_classes::is_closed_polygon;

/// Three black circles, U+25CF, as the replacement for labels that do not
/// fit.
const ELLIPSIS: &str = "\u{25cf}\u{25cf}\u{25cf}";

/// Upstream `getChange1Height`: the length of the first segment of a
/// rectangular area, else 0.
fn get_change1_height(tg: &Tg) -> f64 {
    let rectangular = matches!(
        tg.line_type,
        tl::FSA_RECTANGULAR
            | tl::SHIP_AOI_RECTANGULAR
            | tl::DEFENDED_AREA_RECTANGULAR
            | tl::FFA_RECTANGULAR
            | tl::ACA_RECTANGULAR
            | tl::NFA_RECTANGULAR
            | tl::RFA_RECTANGULAR
            | tl::ATI_RECTANGULAR
            | tl::CFFZ_RECTANGULAR
            | tl::SENSOR_RECTANGULAR
            | tl::CENSOR_RECTANGULAR
            | tl::DA_RECTANGULAR
            | tl::CFZ_RECTANGULAR
            | tl::ZOR_RECTANGULAR
            | tl::TBA_RECTANGULAR
            | tl::TVAR_RECTANGULAR
            | tl::KILLBOXBLUE_RECTANGULAR
            | tl::KILLBOXPURPLE_RECTANGULAR
    );
    if !rectangular {
        return 0.0;
    }
    match tg.pixels.as_slice() {
        [a, b, ..] => ((b.x - a.x).powi(2) + (b.y - a.y).powi(2)).sqrt(),
        _ => 0.0,
    }
}

/// Whether `scale_modifiers` moves this label: the upright ones, plus the
/// slanted ones of the change-1 areas.
fn is_scaled(label: &ModifierLabel, change1: bool) -> bool {
    if label.kind == ABOVE_MIDDLE {
        change1
    } else {
        label.kind == AREA
    }
}

/// Upstream `scaleModifiers`: scales the line factor of an area's labels to
/// the area's height, or collapses the labels that do not fit.
///
/// Upstream catches a failure (no pixels) inside this function and goes on.
pub(crate) fn scale_modifiers(tg: &mut Tg, settings: &Settings) {
    scale(tg, settings).ok();
}

/// The body of [`scale_modifiers`].
fn scale(tg: &mut Tg, settings: &Settings) -> Result<(), EngineError> {
    if !settings.auto_collapse_modifiers || tg.modifiers.is_empty() {
        return Ok(());
    }
    let change1 = is_change1_area(tg.line_type);
    if !is_closed_polygon(tg.line_type) && !change1 {
        return Ok(());
    }
    if matches!(
        tg.line_type,
        tl::PAA_CIRCULAR
            | tl::PAA_RECTANGULAR
            | tl::RECTANGULAR_TARGET
            | tl::RANGE_FAN
            | tl::RANGE_FAN_SECTOR
            | tl::RADAR_SEARCH
    ) {
        return Ok(());
    }
    let (_, ur, lr, _) = get_mbr(tg)?;
    let size = f64::from(tg.font.size);
    let change1_height = get_change1_height(tg);
    let height_mbr = if change1_height <= 0.0 {
        (lr.y - ur.y).abs() / 2.0
    } else {
        change1_height
    };
    let mut min_lf = f64::from(i32::MAX);
    let mut valid = false;
    for label in &tg.modifiers {
        if label.kind == TO_END || (label.kind == ABOVE_MIDDLE && !change1) {
            continue;
        }
        min_lf = min_lf.min(label.line_factor);
        valid = true;
    }
    if !valid {
        return Ok(());
    }
    let height_modifiers = min_lf.abs() * size;
    if height_modifiers > height_mbr {
        shrink(
            tg,
            change1,
            min_lf,
            (height_modifiers - height_mbr) / size,
            height_mbr,
        );
    } else if height_modifiers < 0.5 * height_mbr {
        let factor = (1.0 + (height_mbr / height_modifiers - 1.0) / 4.0).min(2.0);
        for label in tg.modifiers.iter_mut().filter(|l| is_scaled(l, change1)) {
            label.line_factor *= factor;
        }
    }
    Ok(())
}

/// The shrinking half of `scaleModifiers`: moves each label towards the
/// area's centre by `delta`, drops the ones that would leave the area (upstream
/// flags them with the type 7) and adds an ellipsis in their place.
fn shrink(tg: &mut Tg, change1: bool, min_lf: f64, delta: f64, height_mbr: f64) {
    let size = f64::from(tg.font.size);
    let mut ellipsis = ModifierLabel::default();
    let mut add_ellipsis = false;
    for label in tg.modifiers.iter_mut().filter(|l| is_scaled(l, change1)) {
        let new_lf = label.line_factor + delta;
        if (new_lf * size).abs() >= height_mbr {
            if label.line_factor > min_lf {
                ellipsis.kind = label.kind;
                label.kind = ABOVE_START_INSIDE;
                if !label.text.is_empty() {
                    add_ellipsis = true;
                }
            }
            label.line_factor = new_lf;
            ellipsis.text_path = label.text_path;
            continue;
        }
        label.line_factor = new_lf;
    }
    tg.modifiers.retain(|l| l.kind != ABOVE_START_INSIDE);
    if add_ellipsis {
        let max_lf = tg
            .modifiers
            .iter()
            .fold(0.0_f64, |m, l| m.max(l.line_factor));
        ellipsis.text = ELLIPSIS.to_owned();
        ellipsis.line_factor = max_lf + 1.0;
        tg.modifiers.push(ellipsis);
    }
}
