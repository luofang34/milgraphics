//! `AddModifiersGeo` of Modifier2.java: the labels that depend only on the
//! control points, placed before the geometry is built.
//!
//! Upstream chooses the centre of an area by the geodesic centre of the
//! geographic points when it has a converter; this port always takes the
//! pixel branch (`CalcCenterPointDouble2`).

use super::add::{area_modifier, count, integral_modifier, px};
use super::center_label::get_center_label;
use super::geo_areas::area_labels;
use super::geo_lines::line_labels;
use super::geo_routes::route_labels;
use super::group_strings::{build_area_group_dtg_string, build_area_group_string};
use super::layout::{
    EndPoints, add_modifier_on_line, highest_point_left_of_center, middle_segment,
    shift_modifier_path,
};
use super::scale::scale_modifiers;
use super::type_sets::handled_before_geometry;
use super::{ABOVE_MIDDLE, AREA, TO_END};
use crate::engine::base::{EngineError, Pt};
use crate::engine::lineutility::basics::{calc_center_point_double2, mid_point_double};
use crate::engine::lineutility::exterior::get_deep_copy;
use crate::engine::settings::Settings;
use crate::engine::tactical_lines as tl;
use crate::engine::tg::Tg;

/// What every label rule of `AddModifiersGeo` reads besides the graphic.
pub(super) struct Geo<'a> {
    pub(super) settings: &'a Settings,
    pub(super) text_width: &'a dyn Fn(&str) -> f64,
    /// The generic label for the line type.
    pub(super) label: String,
    /// " - " between W and W1 when both are set.
    pub(super) w_dash: &'static str,
    /// " " before T when it is set.
    pub(super) t_space: &'static str,
    /// " - " before T when it is set.
    pub(super) t_dash: &'static str,
    /// Copies of the end points, shifted off the vertical.
    pub(super) ends: EndPoints,
    /// `tg.Pixels.get(2)` and `get(3)` when present.
    pub(super) pt2: Option<Pt>,
    pub(super) pt3: Option<Pt>,
    /// Centre of the area.
    pub(super) center: Pt,
    pub(super) middle_segment: i32,
}

impl Geo<'_> {
    /// `metrics.stringWidth` as the whole pixels upstream stores.
    pub(super) fn sw(&self, text: &str) -> i32 {
        (self.text_width)(text) as i32
    }
}

/// A point a rule needs but a graphic of this size does not have; upstream
/// would dereference null.
pub(super) fn req(p: Option<Pt>) -> Result<Pt, EngineError> {
    p.ok_or(EngineError::Degenerate("label rule needs more points"))
}

/// The same point with `x` moved, as `new POINT2(p.x + dx, p.y)`.
pub(super) fn nudged(p: Pt, dx: f64) -> Pt {
    Pt::new(p.x + dx, p.y)
}

/// Upstream `AddModifiersGeo`: adds the labels that depend on the control
/// points `tg.pixels`. Where upstream's catch would end the phase early this
/// returns the error, leaving the labels added so far (and, as upstream
/// does, the pixels as they were when the failure happened).
pub(crate) fn add_modifiers_geo(
    tg: &mut Tg,
    settings: &Settings,
    text_width: &dyn Fn(&str) -> f64,
) -> Result<(), EngineError> {
    if tg.pixels.is_empty() || !handled_before_geometry(tg.line_type) {
        return Ok(());
    }
    let orig_points = get_deep_copy(&tg.pixels);
    let mut ends = EndPoints::of(tg)?;
    shift_modifier_path(tg, &mut ends)?;
    let center = calc_center_point_double2(&tg.pixels, count(tg))?;
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
        pt2: tg.pixels.get(2).copied(),
        pt3: tg.pixels.get(3).copied(),
        center,
        middle_segment: middle_segment(tg),
    };
    let grouped = settings.group_modifiers && grouped_labels(tg, &mut g)?;
    if !grouped {
        let lt = tg.line_type;
        if !line_labels(tg, &mut g, lt)? && !route_labels(tg, &mut g, lt)? {
            area_labels(tg, &mut g, lt)?;
        }
    }
    scale_modifiers(tg, settings)?;
    tg.pixels = orig_points;
    Ok(())
}

/// The first switch of `AddModifiersGeo`: one multi-line label for the
/// graphics that group their modifiers. True when it handled the graphic.
fn grouped_labels(tg: &mut Tg, g: &mut Geo<'_>) -> Result<bool, EngineError> {
    let (c, cs) = (g.center, 1.0);
    match tg.line_type {
        tl::PAA => {
            add_modifier_on_line(tg, "PAA", false)?;
            let text = build_area_group_string(tg, &g.label);
            area_modifier(tg, &text, AREA, 0.0, (c, c), false);
        }
        tl::ACA
        | tl::FFA
        | tl::RFA
        | tl::NFA
        | tl::FSA
        | tl::WFZ_REVD
        | tl::WFZ
        | tl::OBSFAREA
        | tl::OBSAREA
        | tl::ROZ
        | tl::AARROZ
        | tl::UAROZ
        | tl::WEZ
        | tl::FEZ
        | tl::JEZ
        | tl::FAADZ
        | tl::HIDACZ
        | tl::MEZ
        | tl::LOMEZ
        | tl::HIMEZ => {
            let text = build_area_group_string(tg, &g.label);
            area_modifier(tg, &text, AREA, 0.0, (c, c), false);
        }
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
            let text = build_area_group_string(tg, &g.label);
            area_modifier(tg, &text, AREA, 0.0 * cs, (c, c), false);
            let highest = highest_point_left_of_center(&tg.pixels, c)?;
            let dtg = build_area_group_dtg_string(tg);
            area_modifier(
                tg,
                &dtg,
                TO_END,
                cs,
                (highest, nudged(highest, 0.001)),
                false,
            );
        }
        tl::SPT
        | tl::FRONTAL_ATTACK
        | tl::TURNING_MOVEMENT
        | tl::MOVEMENT_TO_CONTACT
        | tl::AIRAOA
        | tl::AAAAA
        | tl::MAIN => grouped_axis(tg, g)?,
        tl::AC | tl::LLTR | tl::MRR | tl::SL | tl::TC | tl::SAAFR | tl::SC => {
            let seg = g.middle_segment;
            g.center = mid_point_double(px(tg, seg)?, px(tg, seg + 1)?, 0);
            let text = build_area_group_string(tg, &g.label);
            integral_modifier(tg, &text, ABOVE_MIDDLE, 0.0, (seg, seg + 1), false)?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}

/// Grouped text of the axes of advance: the DTG lines and T over the
/// first segment that has room, or over the second.
fn grouped_axis(tg: &mut Tg, g: &Geo<'_>) -> Result<(), EngineError> {
    let mut axis_mod = build_area_group_dtg_string(tg);
    let name = tg.t.clone();
    let (pt0, pt1) = (req(Some(g.ends.pt0))?, req(g.ends.pt1)?);
    let size = tg.pixels.len();
    if size == 3 || size == 4 {
        let mid = if size == 3 {
            mid_point_double(pt0, pt1, 0)
        } else {
            mid_point_double(pt1, req(g.pt2)?, 0)
        };
        if !name.is_empty() {
            if !axis_mod.is_empty() {
                axis_mod.push('\n');
            }
            axis_mod.push_str(&name);
        }
        area_modifier(tg, &axis_mod, AREA, 0.0, (mid, mid), false);
    } else {
        let pt2 = req(g.pt2)?;
        let mid = mid_point_double(pt1, pt2, 0);
        area_modifier(tg, &axis_mod, AREA, 0.0, (mid, mid), false);
        let mid = mid_point_double(pt2, req(g.pt3)?, 0);
        area_modifier(tg, &name, AREA, 0.0, (mid, mid), false);
    }
    Ok(())
}
