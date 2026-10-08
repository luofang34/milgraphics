//! `AddModifiers2` of Modifier2.java: the labels that depend on points the
//! geometry computed, so they are added after it.

use super::add::{area_modifier, count};
use super::center_label::get_center_label;
use super::geo::{Geo, nudged};
use super::group_strings::{build_area_group_dtg_string, build_area_group_string};
use super::layout::{EndPoints, highest_point_left_of_center, shift_modifier_path};
use super::post_areas::post_area_labels;
use super::post_sector::sector_labels;
use super::post_tasks::task_labels;
use super::scale::scale_modifiers;
use super::type_sets::handled_after_geometry;
use super::{AREA, TO_END};
use crate::engine::base::{At, EngineError, Pt};
use crate::engine::lineutility::basics::{calc_center_point_double2, mid_point_double};
use crate::engine::lineutility::exterior::get_deep_copy;
use crate::engine::settings::Settings;
use crate::engine::tactical_lines as tl;
use crate::engine::tg::Tg;

/// Where the sector labels are measured from: the ground size of a pixel,
/// and the first control point in pixels (the origin of a radar search).
#[derive(Clone, Copy, Debug)]
pub(crate) struct SectorFrame {
    /// Ground metres per pixel.
    pub(crate) meters_per_pixel: f64,
    /// The first control point, in pixels.
    pub(crate) origin: Pt,
}

/// `Math.round(float)` of `size * 0.5625f`, the index of the upper left
/// point of a circle drawn with `size` points.
pub(super) fn upper_left_index(size: usize) -> i32 {
    let v = size as f32 * 0.5625_f32;
    (v + 0.5_f32).floor() as i32
}

/// Upstream `AddModifiers2`: adds the labels that depend on the drawn
/// points in `tg.pixels`. `frame` gives the ground scale for the labels of
/// range-fan sectors, which upstream places by geodesic computation.
pub(crate) fn add_modifiers2(
    tg: &mut Tg,
    settings: &Settings,
    text_width: &dyn Fn(&str) -> f64,
    frame: SectorFrame,
) -> Result<(), EngineError> {
    if tg.pixels.is_empty() || !handled_after_geometry(tg.line_type) {
        return Ok(());
    }
    let orig_points = get_deep_copy(&tg.pixels);
    let mut ends = EndPoints::of(tg)?;
    shift_modifier_path(tg, &mut ends)?;
    let mut g = Geo {
        settings,
        text_width,
        label: get_center_label(tg),
        w_dash: if tg.w.is_empty() || tg.w1.is_empty() {
            ""
        } else {
            " - "
        },
        t_space: if tg.t.is_empty() { "" } else { " " },
        t_dash: if tg.t.is_empty() { "" } else { " - " },
        ends,
        pt2: None,
        pt3: None,
        center: Pt::default(),
        middle_segment: 0,
    };
    let grouped = settings.group_modifiers && grouped_labels(tg, &g)?;
    if !grouped {
        let lt = tg.line_type;
        if !task_labels(tg, &mut g, lt)? && !post_area_labels(tg, &mut g, lt)? {
            sector_labels(tg, lt, frame)?;
        }
    }
    scale_modifiers(tg, settings);
    tg.pixels = orig_points;
    Ok(())
}

/// The center of a circle drawn with the pixels, as the circular areas use.
pub(super) fn circle_center(tg: &Tg) -> Result<Pt, EngineError> {
    Ok(mid_point_double(
        tg.pixels.at(0)?,
        tg.pixels.at(tg.pixels.len() / 2)?,
        0,
    ))
}

/// The center of a rectangle drawn with the first four pixels.
pub(super) fn rectangle_center(tg: &Tg) -> Result<Pt, EngineError> {
    let left = mid_point_double(tg.pixels.at(0)?, tg.pixels.at(1)?, 0);
    let right = mid_point_double(tg.pixels.at(2)?, tg.pixels.at(3)?, 0);
    Ok(mid_point_double(left, right, 0))
}

/// The grouping switch: one multi-line label for the areas that group their
/// modifiers. True when it handled the graphic.
fn grouped_labels(tg: &mut Tg, g: &Geo<'_>) -> Result<bool, EngineError> {
    let cs = 1.0;
    let lt = tg.line_type;
    match lt {
        tl::ACA_RECTANGULAR | tl::ACA_CIRCULAR => {
            let c = calc_center_point_double2(&tg.pixels, count(tg))?;
            let text = build_area_group_string(tg, &g.label);
            area_modifier(tg, &text, AREA, 0.0, (c, c), false);
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
        | tl::KILLBOXPURPLE_CIRCULAR => {
            let c = circle_center(tg)?;
            let text = build_area_group_string(tg, &g.label);
            area_modifier(tg, &text, AREA, 0.0, (c, c), false);
            let pos = tg
                .pixels
                .at(usize::try_from(upper_left_index(tg.pixels.len())).unwrap_or(0))?;
            let dtg = build_area_group_dtg_string(tg);
            area_modifier(tg, &dtg, TO_END, -cs, (pos, nudged(pos, 0.001)), false);
        }
        tl::FFA_CIRCULAR | tl::NFA_CIRCULAR | tl::RFA_CIRCULAR => {
            let c = mid_point_double(tg.pixels.at(0)?, tg.pixels.at(51)?, 0);
            let text = build_area_group_string(tg, &g.label);
            area_modifier(tg, &text, AREA, 0.0, (c, c), false);
        }
        tl::FFA_RECTANGULAR | tl::NFA_RECTANGULAR | tl::RFA_RECTANGULAR => {
            let c = rectangle_center(tg)?;
            let text = build_area_group_string(tg, &g.label);
            area_modifier(tg, &text, AREA, 0.0, (c, c), false);
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
        | tl::TVAR_RECTANGULAR => {
            let c = rectangle_center(tg)?;
            let text = build_area_group_string(tg, &g.label);
            area_modifier(tg, &text, AREA, 0.0 * cs, (c, c), false);
            let pos = highest_point_left_of_center(&tg.pixels, c)?;
            let dtg = build_area_group_dtg_string(tg);
            area_modifier(tg, &dtg, TO_END, cs, (pos, nudged(pos, 0.001)), false);
        }
        _ => return Ok(false),
    }
    Ok(true)
}
