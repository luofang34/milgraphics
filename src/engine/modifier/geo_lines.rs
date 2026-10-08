//! The line rules of `AddModifiersGeo` (Modifier2.java): labels at the ends
//! of lines and on their middle segment.

use super::add::{add_modifier2, area_modifier, area_modifier_opt, integral_modifier, px};
use super::boundary::add_boundary_modifiers;
use super::center_label::symbol_version;
use super::geo::{Geo, req};
use super::layout::{
    add_dtg, add_modifier_on_line, add_modifier_top_segment, get_mbr, pixels_middle_segment,
};
use super::{ABOVE_END, ABOVE_MIDDLE, AREA, TO_END};
use crate::engine::base::{EngineError, Pt};
use crate::engine::lineutility::basics::{calc_distance_double, mid_point_double};
use crate::engine::lineutility::extend::extend_along_line_double;
use crate::engine::settings::SHIFT_LINES;
use crate::engine::tactical_lines as tl;
use crate::engine::tg::Tg;

/// Labels `line_type` if it is one of the line rules; true when it was.
pub(super) fn line_labels(
    tg: &mut Tg,
    g: &mut Geo<'_>,
    line_type: i32,
) -> Result<bool, EngineError> {
    Ok(end_rules(tg, g, line_type)? || segment_rules(tg, g, line_type)?)
}

/// The rules that label the ends of a line.
fn end_rules(tg: &mut Tg, g: &mut Geo<'_>, line_type: i32) -> Result<bool, EngineError> {
    let cs = 1.0;
    let label = g.label.clone();
    let name = tg.t.clone();
    match line_type {
        tl::PL => {
            let text = format!("{label}{}{name}", g.t_space);
            end_pair(tg, g, &text, TO_END, 0.0);
        }
        tl::BS_LINE | tl::BBS_LINE => battle_symbol_line(tg, g, &name)?,
        tl::FEBA => end_pair(tg, g, &label, TO_END, 0.0),
        tl::FSCL => {
            let text = format!("{name} {label}");
            let width = g.sw(&text).max(g.sw(&tg.w));
            end_labels(tg, g, (&text, &text), width)?;
        }
        tl::ICL | tl::NFL | tl::BCL_REVD | tl::RFL | tl::BCL => limit_line(tg, g, line_type)?,
        tl::DIRATKSPT | tl::DIRATKAIR | tl::DIRATKGND => {
            let pt0 = g.ends.pt0;
            let pt1 = req(g.ends.pt1)?;
            let mid = mid_point_double(pt0, pt1, 0);
            area_modifier(tg, &name, ABOVE_MIDDLE, 0.0, (pt0, mid), false);
            add_dtg(tg, ABOVE_MIDDLE, (cs, 2.0 * cs), pt0, pt1);
        }
        tl::LL
        | tl::LOD
        | tl::LDLC
        | tl::PLD
        | tl::RELEASE
        | tl::HOL
        | tl::BHL
        | tl::FCL
        | tl::HOLD
        | tl::BRDGHD
        | tl::HOLD_GE
        | tl::BRDGHD_GE
        | tl::LOA
        | tl::IFF_OFF
        | tl::IFF_ON => end_pair(tg, g, &label, ABOVE_END, -cs),
        tl::EWL => {
            end_pair(tg, g, &label, ABOVE_END, -cs);
            tg.echelon_symbol.clear();
            add_boundary_modifiers(tg, g.text_width)?;
        }
        tl::BOUNDARY => add_boundary_modifiers(tg, g.text_width)?,
        _ => return Ok(false),
    }
    Ok(true)
}

/// The rules that label the middle or a chosen segment of a line.
fn segment_rules(tg: &mut Tg, g: &mut Geo<'_>, line_type: i32) -> Result<bool, EngineError> {
    let cs = 1.0;
    let label = g.label.clone();
    let name = tg.t.clone();
    match line_type {
        tl::MFP => {
            let seg = g.middle_segment;
            px(tg, seg)?;
            px(tg, seg + 1)?;
            integral_modifier(tg, &label, ABOVE_MIDDLE, 0.0, (seg, seg + 1), true)?;
            let dtg = format!("{}{}", tg.w, g.w_dash);
            integral_modifier(tg, &dtg, ABOVE_END, cs, (0, 1), false)?;
            let dtg1 = tg.w1.clone();
            integral_modifier(tg, &dtg1, ABOVE_END, 2.0 * cs, (0, 1), false)?;
        }
        tl::LINTGT | tl::LINTGTS | tl::FPF => target_lines(tg, g, line_type)?,
        tl::LINE => {
            let seg = g.middle_segment;
            integral_modifier(tg, &name, ABOVE_MIDDLE, cs, (seg, seg + 1), false)?;
        }
        tl::CFL => cfl(tg, g)?,
        tl::FLOT => flot(tg, g)?,
        tl::LC => lc(tg, g)?,
        tl::CATK => integral_modifier(tg, &label, ABOVE_MIDDLE, 0.0, (1, 0), false)?,
        tl::CATKBYFIRE => {
            let width = (1.5 * f64::from(g.sw(&label))) as i32;
            let pt1 = req(g.ends.pt1)?;
            let pt2 = extend_along_line_double(g.ends.pt0, pt1, f64::from(width));
            add_modifier2(tg, &label, ABOVE_MIDDLE, 0.0, (pt1, pt2), false, None);
        }
        tl::IL => integral_modifier(tg, &name, ABOVE_MIDDLE, 0.0, (1, 0), false)?,
        tl::RETIRE
        | tl::PURSUIT
        | tl::WITHDRAW
        | tl::DISENGAGE
        | tl::WDRAWUP
        | tl::FPOL
        | tl::RPOL
        | tl::DEMONSTRATE => integral_modifier(tg, &label, ABOVE_MIDDLE, 0.0, (0, 1), true)?,
        tl::DELAY => {
            let dtg = tg.w.clone();
            integral_modifier(tg, &dtg, ABOVE_MIDDLE, -cs, (0, 1), false)?;
            integral_modifier(tg, &label, ABOVE_MIDDLE, 0.0, (0, 1), true)?;
        }
        tl::GENERIC_LINE => {
            let text = format!("{} {name}", tg.h);
            let width = g.sw(&text).max(g.sw(&tg.w));
            end_labels(tg, g, (&text, &text), width)?;
        }
        tl::MINED | tl::FENCED => mined(tg, g, line_type == tl::MINED)?,
        tl::MINE_LINE => {
            if tg.is_hostile() {
                let n = tg.n.clone();
                end_pair(tg, g, &n, TO_END, 0.0);
            }
        }
        tl::ENCIRCLE => {
            if tg.is_hostile() {
                let n = tg.n.clone();
                integral_modifier(tg, &n, ABOVE_MIDDLE, 0.0, (0, 1), true)?;
                let seg = g.middle_segment;
                integral_modifier(tg, &n, ABOVE_MIDDLE, 0.0, (seg, seg + 1), true)?;
            }
        }
        tl::SERIES | tl::DRCL => add_modifier_top_segment(tg, &name)?,
        tl::STRIKWARN => {
            integral_modifier(tg, "1", ABOVE_MIDDLE, 0.0, (0, 1), true)?;
            let half = i32::try_from(tg.pixels.len() / 2).unwrap_or(i32::MAX);
            integral_modifier(tg, "2", ABOVE_MIDDLE, 0.0, (half, half + 1), true)?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}

/// The same label at the first and the last segment of the line, from the
/// shifted end points. A missing second point adds nothing.
pub(super) fn end_pair(tg: &mut Tg, g: &Geo<'_>, text: &str, kind: i32, line_factor: f64) {
    let e = g.ends;
    area_modifier_opt(tg, text, kind, line_factor, (e.pt0, e.pt1), false);
    area_modifier_opt(
        tg,
        text,
        kind,
        line_factor,
        (e.pt_last, e.pt_next_to_last),
        false,
    );
}

/// Upstream `BS_LINE` and `BBS_LINE`: where the name goes depends on T1.
fn battle_symbol_line(tg: &mut Tg, g: &Geo<'_>, name: &str) -> Result<(), EngineError> {
    match tg.t1.as_str() {
        "" | "2" => end_pair(tg, g, name, TO_END, 0.0),
        "1" => {
            for j in 0..i32::try_from(tg.pixels.len()).unwrap_or(0) - 1 {
                let (p0, p1) = (px(tg, j)?, px(tg, j + 1)?);
                area_modifier(tg, name, ABOVE_MIDDLE, 0.0, (p0, p1), false);
            }
        }
        "3" => {
            // The name sits beyond either end of the polyline and on each
            // intermediate vertex.
            let half = f64::from(g.sw(name) / 2);
            let (pt0, pt1) = (g.ends.pt0, req(g.ends.pt1)?);
            let (last, next) = (g.ends.pt_last, req(g.ends.pt_next_to_last)?);
            let dist = calc_distance_double(pt0, pt1);
            let pt2 = crate::engine::lineutility::extend::extend_along_line_double2(
                pt1,
                pt0,
                dist + half,
            );
            area_modifier(tg, name, AREA, 0.0, (pt2, pt2), false);
            let dist = calc_distance_double(next, last);
            let pt2 = crate::engine::lineutility::extend::extend_along_line_double2(
                next,
                last,
                dist + half,
            );
            area_modifier(tg, name, AREA, 0.0, (pt2, pt2), false);
            for j in 1..i32::try_from(tg.pixels.len()).unwrap_or(0) - 1 {
                let p = px(tg, j)?;
                area_modifier(tg, name, AREA, 0.0, (p, p), false);
            }
        }
        _ => end_pair(tg, g, name, TO_END, 0.0),
    }
    Ok(())
}

/// The three labels (text, W, W1) over a path that starts at `pt0`.
fn text_dtg_triple(tg: &mut Tg, g: &Geo<'_>, text: &str, path: (Pt, Pt)) {
    let cs = 1.0;
    add_modifier2(tg, text, ABOVE_MIDDLE, -0.7 * cs, path, false, None);
    let dtg = format!("{}{}", tg.w, g.w_dash);
    add_modifier2(tg, &dtg, ABOVE_MIDDLE, 0.7 * cs, path, false, None);
    let dtg1 = tg.w1.clone();
    add_modifier2(tg, &dtg1, ABOVE_MIDDLE, 1.7 * cs, path, false, None);
}

/// The labels at the ends of FSCL, the limit lines and the generic line:
/// `texts.0` over the first segment (and over the last when the line has
/// more than one), `texts.1` over the last of a single segment. The ends
/// are labelled only where there is room for the text.
pub(super) fn end_labels(
    tg: &mut Tg,
    g: &Geo<'_>,
    texts: (&str, &str),
    string_width: i32,
) -> Result<(), EngineError> {
    let width = f64::from(string_width);
    let (pt0, pt1) = (px(tg, 0)?, px(tg, 1)?);
    let last = i32::try_from(tg.pixels.len()).unwrap_or(i32::MAX) - 1;
    let (pt2, pt3) = (px(tg, last)?, px(tg, last - 1)?);
    let dist = calc_distance_double(pt0, pt1);
    let dist2 = calc_distance_double(pt2, pt3);
    let first_end = (pt0, extend_along_line_double(pt0, pt1, width));
    let last_end = (pt2, extend_along_line_double(pt2, pt3, width));
    if tg.pixels.len() == 2 {
        text_dtg_triple(tg, g, texts.0, first_end);
        if dist > 3.5 * width {
            text_dtg_triple(tg, g, texts.1, last_end);
        }
    } else {
        let dist3 = calc_distance_double(pt0, pt2);
        if dist > width + 5.0 || dist >= dist2 || dist3 > width + 5.0 {
            text_dtg_triple(tg, g, texts.0, first_end);
        }
        if dist2 > width + 5.0 || dist2 > dist || dist3 > width + 5.0 {
            text_dtg_triple(tg, g, texts.0, last_end);
        }
    }
    Ok(())
}

/// Upstream's "T after label" group: the label, then T (how depends on the
/// symbol's standard version).
fn limit_line(tg: &mut Tg, g: &Geo<'_>, line_type: i32) -> Result<(), EngineError> {
    let label = &g.label;
    let name = tg.t.clone();
    let version = symbol_version(&tg.symbol_id).ok_or(EngineError::Number(tg.symbol_id.clone()))?;
    let (mut t_mod, mut width) = (String::new(), 0);
    if version < 13 {
        t_mod = name.clone();
        width = g.sw(&format!("{t_mod} {label}"));
    } else if version == 13 || version == 15 {
        if line_type == tl::BCL {
            if !name.is_empty() {
                t_mod = format!(" ({name})");
            }
            width = g.sw(&format!("{label}{t_mod}"));
        } else {
            t_mod = name.clone();
            width = g.sw(&format!("{name} {label}"));
        }
    } else if version == 16 {
        if !name.is_empty() {
            t_mod = format!(" {name}");
        }
        if !tg.as_.is_empty() {
            t_mod.push_str(&format!(" ({})", tg.as_));
        }
        width = g.sw(&format!("{label}{t_mod}"));
    }
    width = width.max(g.sw(&tg.w));
    let near = format!("{label}{}{t_mod}", g.t_space);
    let far_single = format!("{label}{t_mod}");
    end_labels(tg, g, (&near, &far_single), width)
}

/// `LINTGT`, `LINTGTS` and `FPF`: AP, label, T1 and V over a segment.
fn target_lines(tg: &mut Tg, g: &Geo<'_>, line_type: i32) -> Result<(), EngineError> {
    let cs = 1.0;
    let (ap, v, t1) = (tg.ap.clone(), tg.v.clone(), tg.t1.clone());
    let seg = g.middle_segment;
    match line_type {
        tl::LINTGT => integral_modifier(tg, &ap, ABOVE_MIDDLE, -0.7 * cs, (seg, seg + 1), false),
        tl::LINTGTS => {
            integral_modifier(tg, &ap, ABOVE_MIDDLE, -0.7 * cs, (seg, seg + 1), false)?;
            integral_modifier(tg, &g.label, ABOVE_MIDDLE, 0.7 * cs, (seg, seg + 1), false)
        }
        _ => {
            integral_modifier(tg, &ap, ABOVE_MIDDLE, -0.7 * cs, (0, 1), false)?;
            integral_modifier(tg, &g.label, ABOVE_MIDDLE, 0.7 * cs, (0, 1), false)?;
            integral_modifier(tg, &t1, ABOVE_MIDDLE, 1.7 * cs, (0, 1), false)?;
            integral_modifier(tg, &v, ABOVE_MIDDLE, 2.7 * cs, (0, 1), false)
        }
    }
}

/// Upstream `CFL`: the label on a segment of the text's width centred on
/// the middle of the line.
fn cfl(tg: &mut Tg, g: &Geo<'_>) -> Result<(), EngineError> {
    let cs = 1.0;
    let text = format!("{}{}{}", g.label, g.t_space, tg.t);
    let dtg = format!("{}{}{}", tg.w, g.w_dash, tg.w1);
    let width = g.sw(&text).max(g.sw(&dtg));
    let seg = g.middle_segment;
    let start = (px(tg, seg)?, px(tg, seg + 1)?);
    let (pt0, pt1) = pixels_middle_segment(tg, f64::from(width), start)?;
    add_modifier2(tg, &text, ABOVE_MIDDLE, -0.7 * cs, (pt0, pt1), false, None);
    add_dtg(tg, ABOVE_MIDDLE, (0.7 * cs, 1.7 * cs), pt0, pt1);
    Ok(())
}

/// Upstream `FLOT`: H picks the label ("1" gives LC, "2" none).
fn flot(tg: &mut Tg, g: &Geo<'_>) -> Result<(), EngineError> {
    let label = match tg.h.as_str() {
        "1" => "LC",
        "2" => "",
        _ => g.label.as_str(),
    };
    end_pair(tg, g, label, TO_END, 0.0);
    if tg.is_hostile() {
        let n = tg.n.clone();
        end_pair(tg, g, &n, TO_END, -1.0);
    }
    Ok(())
}

/// Upstream `LC`: N (when hostile) on the side of the line the line's
/// decoration is not on, then the label.
fn lc(tg: &mut Tg, g: &Geo<'_>) -> Result<(), EngineError> {
    let shift = if SHIFT_LINES { 0.5 } else { 1.0 };
    let e = g.ends;
    if tg.is_hostile() {
        let n = tg.n.clone();
        let pt1 = req(e.pt1)?;
        let factor = if e.pt0.x < pt1.x { -shift } else { shift };
        area_modifier(tg, &n, TO_END, factor, (e.pt0, pt1), false);
        let next = req(e.pt_next_to_last)?;
        let factor = if next.x < e.pt_last.x { -shift } else { shift };
        area_modifier(tg, &n, TO_END, factor, (e.pt_last, next), false);
    }
    end_pair(tg, g, &g.label, TO_END, 0.0);
    Ok(())
}

/// `MINED` and `FENCED`: N on the first and middle segments when hostile,
/// then "M" on the outline. Mined areas also get H and W above and below the
/// bounding box.
fn mined(tg: &mut Tg, g: &Geo<'_>, is_mined: bool) -> Result<(), EngineError> {
    let cs = 1.0;
    if tg.is_hostile() {
        let n = tg.n.clone();
        let pt1 = mid_point_double(g.ends.pt0, req(g.ends.pt1)?, 0);
        area_modifier(tg, &n, ABOVE_MIDDLE, 0.0, (g.ends.pt0, pt1), true);
        if g.middle_segment != 0 {
            let seg = g.middle_segment;
            let pt0 = px(tg, seg)?;
            let pt1 = mid_point_double(pt0, px(tg, seg + 1)?, 0);
            area_modifier(tg, &n, ABOVE_MIDDLE, 0.0, (pt0, pt1), true);
        }
    }
    if is_mined {
        let (ul, ur, lr, ll) = get_mbr(tg)?;
        let (h, w) = (tg.h.clone(), tg.w.clone());
        area_modifier(tg, &h, ABOVE_MIDDLE, -1.5 * cs, (ul, ur), false);
        area_modifier(tg, &w, ABOVE_MIDDLE, 1.5 * cs, (ll, lr), false);
    }
    add_modifier_on_line(tg, "M", false)?;
    Ok(())
}
