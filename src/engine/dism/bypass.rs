//! BYPASS, BREACH and CANALIZE from DISMSupport.java: an open rectangle
//! through the first two control points with a barb pair at each end.

use super::support::{
    MAX_LENGTH, MIN_LENGTH, calc_endpiece_deltas_double, determine_direction_double,
    draw_endpiece_deltas_double, draw_open_rectangle_double, put,
};
use crate::engine::base::{At, EngineError, Pt};
use crate::engine::lineutility::basics::{calc_distance_double, mid_point_double};
use crate::engine::lineutility::bounds::mbr_distance;
use crate::engine::lineutility::extend::{extend_line_double, extend_line2_double};
use crate::engine::lineutility::relative::point_relative_to_line;
use crate::engine::settings::Settings;
use std::f64::consts::PI;

/// The four barb orientations the three figures choose between, as the
/// (dx1, dy1, dx2, dy2) the end piece is drawn with.
#[derive(Clone, Copy, Debug)]
enum Barbs {
    /// `(dx, dy, dy, -dx)`
    Square,
    /// `(dy, -dx, dx, dy)`
    SquareFlipped,
    /// `(dx, dy, -dx, -dy)`
    Straight,
    /// `(dy, -dx, -dy, dx)`
    StraightFlipped,
}

impl Barbs {
    fn deltas(self, dx: f64, dy: f64) -> (f64, f64, f64, f64) {
        match self {
            Self::Square => (dx, dy, dy, -dx),
            Self::SquareFlipped => (dy, -dx, dx, dy),
            Self::Straight => (dx, dy, -dx, -dy),
            Self::StraightFlipped => (dy, -dx, -dy, dx),
        }
    }
}

/// Which control point each barb set is drawn at, and its orientation, for
/// a figure whose first point is above (`above`) the second and that opens
/// to the right (`right`). The first entry is emitted first.
type Plan = [(usize, Barbs); 2];

fn open_figure(
    points: &mut Vec<Pt>,
    settings: &Settings,
    plan: impl Fn(bool, bool) -> Plan,
) -> Result<i32, EngineError> {
    let save = [points.at(0)?, points.at(1)?, points.at(2)?];
    let rect = draw_open_rectangle_double(&save)?;
    let mut counter = 0;
    for p in rect {
        put(points, counter, p);
        counter += 1;
    }
    let right = determine_direction_double(&save)? != 0;
    let (dx, dy) = calc_endpiece_deltas_double(&save, PI / 4.0, settings.dpi_scale_factor())?;
    let above = (save[0].y - save[1].y) < 0.0;
    for (at, barbs) in plan(above, right) {
        let (d1, d2, d3, d4) = barbs.deltas(dx, dy);
        for p in draw_endpiece_deltas_double(save.at(at)?, d1, d2, d3, d4) {
            put(points, counter, p);
            counter += 1;
        }
    }
    Ok(counter as i32)
}

/// Upstream `GetDISMBypassDouble`: the BYPASS figure, 12 points.
pub(crate) fn get_dism_bypass_double(
    points: &mut Vec<Pt>,
    settings: &Settings,
) -> Result<i32, EngineError> {
    open_figure(points, settings, |above, right| {
        let barbs = if above == right {
            Barbs::Square
        } else {
            Barbs::SquareFlipped
        };
        [(0, barbs), (1, barbs)]
    })
}

/// Upstream `GetDISMBreachDouble`: the BREACH figure, 12 points.
pub(crate) fn get_dism_breach_double(
    points: &mut Vec<Pt>,
    settings: &Settings,
) -> Result<i32, EngineError> {
    open_figure(points, settings, |above, right| {
        if above == right {
            [(0, Barbs::Straight), (1, Barbs::StraightFlipped)]
        } else {
            [(0, Barbs::StraightFlipped), (1, Barbs::Straight)]
        }
    })
}

/// Upstream `GetDISMCanalizeDouble`: the CANALIZE figure, 12 points.
pub(crate) fn get_dism_canalize_double(
    points: &mut Vec<Pt>,
    settings: &Settings,
) -> Result<i32, EngineError> {
    open_figure(points, settings, |above, right| {
        if above == right {
            [(1, Barbs::Straight), (0, Barbs::StraightFlipped)]
        } else {
            [(1, Barbs::StraightFlipped), (0, Barbs::Straight)]
        }
    })
}

/// The two barb sets BYPASS and BYDIF draw at the first two control points.
pub(super) fn square_barb_sets(
    save: &[Pt; 3],
    settings: &Settings,
) -> Result<([Pt; 4], [Pt; 4]), EngineError> {
    let right = determine_direction_double(save)? != 0;
    let (dx, dy) = calc_endpiece_deltas_double(save, PI / 4.0, settings.dpi_scale_factor())?;
    let above = (save[0].y - save[1].y) < 0.0;
    let (d1, d2, d3, d4) = if above == right {
        Barbs::Square
    } else {
        Barbs::SquareFlipped
    }
    .deltas(dx, dy);
    Ok((
        draw_endpiece_deltas_double(save[0], d1, d2, d3, d4),
        draw_endpiece_deltas_double(save[1], d1, d2, d3, d4),
    ))
}

/// Upstream `GetDISMByImpDouble`: BYIMP, an open rectangle with a gap in the
/// back line and two filled barb triangles.
pub(crate) fn get_dism_by_imp_double(
    points: &mut Vec<Pt>,
    settings: &Settings,
) -> Result<i32, EngineError> {
    let dpi = settings.dpi_scale_factor();
    let mut mbr = mbr_distance(points, 3)?;
    if mbr > 40.0 * MAX_LENGTH * dpi {
        mbr = 40.0 * MAX_LENGTH * dpi;
    }
    if mbr < 5.0 * MIN_LENGTH * dpi {
        mbr = 5.0 * MIN_LENGTH * dpi;
    }
    if mbr > 250.0 * dpi {
        mbr = 250.0 * dpi;
    }
    if mbr / 15.0 > calc_distance_double(points.at(0)?, points.at(1)?) {
        mbr = 15.0 * calc_distance_double(points.at(0)?, points.at(1)?);
    }
    let save = [points.at(0)?, points.at(1)?, points.at(2)?];
    let rect = draw_open_rectangle_double(&save)?;
    let mid = mid_point_double(rect[1], rect[2], 0);
    let gap0 = extend_line2_double(rect[1], mid, -mbr / 30.0, 5);
    let gap1 = extend_line2_double(rect[1], mid, mbr / 30.0, 5);
    let rel = point_relative_to_line(rect[0], rect[1], gap0);
    let tick0 = extend_line_double(rel, gap0, -mbr / 30.0);
    let tick1 = extend_line_double(rel, gap0, mbr / 30.0);
    let rel = point_relative_to_line(rect[2], rect[3], gap1);
    let tick2 = extend_line_double(rel, gap1, -mbr / 30.0);
    let tick3 = extend_line_double(rel, gap1, mbr / 30.0);
    let outline = [
        (rect[0], 0),
        (rect[1], 0),
        (gap0, 5),
        (tick0, 0),
        (tick1, 5),
        (tick2, 0),
        (tick3, 5),
        (gap1, 0),
        (rect[2], 0),
        (rect[3], 5),
    ];
    let mut counter = 0;
    for (p, style) in outline {
        let mut q = p;
        q.style = style;
        put(points, counter, q);
        counter += 1;
    }
    push_filled_barbs(points, &mut counter, &save, settings)?;
    Ok(counter as i32)
}

/// Appends the two filled barb triangles (styles 9, 9, 9, 10 each) that
/// close BYIMP, BYDIF and EASY.
pub(super) fn push_filled_barbs(
    points: &mut Vec<Pt>,
    counter: &mut usize,
    save: &[Pt; 3],
    settings: &Settings,
) -> Result<(), EngineError> {
    let (d1, d2) = square_barb_sets(save, settings)?;
    for d in [d1, d2] {
        for (p, style) in [(d[1], 9), (d[0], 9), (d[3], 9), (d[3], 10)] {
            let mut q = p;
            q.style = style;
            put(points, *counter, q);
            *counter += 1;
        }
    }
    Ok(())
}

/// Upstream `GetDISMEasyDouble`: EASY, an open rectangle with two filled
/// barb triangles.
pub(crate) fn get_dism_easy_double(
    points: &mut Vec<Pt>,
    settings: &Settings,
) -> Result<i32, EngineError> {
    let save = [points.at(0)?, points.at(1)?, points.at(2)?];
    let rect = draw_open_rectangle_double(&save)?;
    let mut counter = 0;
    for p in rect {
        put(points, counter, p);
        counter += 1;
    }
    push_filled_barbs(points, &mut counter, &save, settings)?;
    Ok(counter as i32)
}
