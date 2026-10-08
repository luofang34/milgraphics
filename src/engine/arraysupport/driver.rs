//! Port of `arraysupport.GetLineArray2Double` without the case bodies: sets
//! up the working state, runs the point builders, then the shape builders.

use crate::engine::base::{At, EngineError, Pt, Shape};
use crate::engine::lineutility::bounds::mbr_distance;
use crate::engine::settings::Settings;
use crate::engine::tactical_lines as lt;
use crate::engine::tg::Tg;

use super::points::build_points;
use super::shapes;
use super::work::{Work, get};

/// Line types whose point list is returned alongside the shapes
/// (`FillPoints(pLinePoints, acCounter, points)` in upstream's second
/// switch).
fn returns_points(line_type: i32) -> bool {
    matches!(
        line_type,
        lt::CONTAIN
            | lt::BLOCK
            | lt::COVER
            | lt::SCREEN
            | lt::GUARD
            | lt::ESCORT
            | lt::PAA_RECTANGULAR
            | lt::RECTANGULAR_TARGET
            | lt::FOLSP
            | lt::FOLLA
            | lt::BREACH
            | lt::BYPASS
            | lt::CANALIZE
            | lt::CLEAR
            | lt::DISRUPT
            | lt::FIX
            | lt::ISOLATE
            | lt::OCCUPY
            | lt::PENETRATE
            | lt::RETAIN
            | lt::SECURE
            | lt::CONTROL
            | lt::LOCATE
            | lt::AREA_DEFENSE
            | lt::SEIZE
            | lt::CAPTURE
            | lt::EVACUATE
            | lt::TURN
            | lt::BS_RECTANGLE
            | lt::BBS_RECTANGLE
            | lt::AIRFIELD
            | lt::CORDONKNOCK
            | lt::CORDONSEARCH
            | lt::DENY
            | lt::MSDZ
            | lt::CONVOY
            | lt::HCONVOY
            | lt::MFLANE
            | lt::DIRATKAIR
            | lt::ABATIS
            | lt::MOBILE_DEFENSE
            | lt::ENVELOPMENT
    )
}

/// `FillPoints`: the first `count` points of the array.
fn fill_points(p: &[Pt], count: i32) -> Result<Vec<Pt>, EngineError> {
    (0..count).map(|j| get(p, j)).collect()
}

/// Upstream `GetLineArray2Double`. `p` is the point array sized by
/// [`super::count::get_counters_double`] and holding the `save` control
/// points first. Shapes are appended to `shapes`; the returned list is
/// empty for most line types, as upstream's.
pub(crate) fn get_line_array2_double(
    tg: &mut Tg,
    p: Vec<Pt>,
    vbl: i32,
    save: i32,
    shapes_out: &mut Vec<Shape>,
    settings: &Settings,
) -> Result<Vec<Pt>, EngineError> {
    if p.len() < 2 {
        return Err(EngineError::Degenerate("fewer than two points"));
    }
    let line_type = tg.line_type;
    let d_mbr = mbr_distance(&p, save)?;
    let quiet = |pt: Pt| Pt { style: 0, ..pt };
    let pt0 = quiet(p.at(0)?);
    let pt1 = quiet(p.at(1)?);
    let pt2 = if vbl > 2 { quiet(p.at(2)?) } else { pt1 };
    let orig = (0..save)
        .map(|j| get(&p, j))
        .collect::<Result<Vec<_>, _>>()?;
    let mut w = Work {
        tg,
        settings,
        line_type,
        dpi: settings.dpi_scale_factor(),
        p,
        orig,
        vbl,
        save,
        d_mbr,
        pt0,
        pt1,
        pt2,
        ac: 0,
        points: Vec::new(),
    };
    build_points(&mut w)?;
    if line_type == lt::BOUNDARY {
        return fill_points(&w.p, w.ac);
    }
    if returns_points(line_type) {
        w.points = fill_points(&w.p, w.ac)?;
    }
    shapes::build(&mut w, shapes_out)?;
    Ok(w.points)
}
