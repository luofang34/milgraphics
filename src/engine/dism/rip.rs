//! RIP, DEMONSTRATE, MOBILE_DEFENSE (base figure), BYDIF and PENETRATE from
//! DISMSupport.java.
//!
//! BYDIF implements only the path where no clip rectangle bounds the
//! jagged back line, so the zigzag is always drawn.

use super::bypass::push_filled_barbs;
use super::fix::push_jaggy_line;
use super::support::{
    MAX_LENGTH, MIN_LENGTH, arc_approximation_double, clamp_size, draw_endpiece_deltas_double,
    draw_open_rectangle_double, put, side,
};
use crate::engine::base::{At, EngineError, Pt};
use crate::engine::lineutility::arrow::get_arrow_head4_double;
use crate::engine::lineutility::basics::mid_point_double;
use crate::engine::lineutility::bounds::mbr_distance;
use crate::engine::settings::Settings;
use crate::engine::tactical_lines as lt;
use std::f64::consts::PI;

const RIGHT_SIDE: i32 = 1;
const COLINEAR: i32 = 2;

fn styled(p: Pt, style: i32) -> Pt {
    let mut q = p;
    q.style = style;
    q
}

fn push_segment(points: &mut Vec<Pt>, counter: &mut usize, a: Pt, b: Pt) {
    put(points, *counter, styled(a, 0));
    put(points, *counter + 1, styled(b, 5));
    *counter += 2;
}

fn is_clockwise(p: &[Pt]) -> Result<bool, EngineError> {
    let (p0, p1, p2, p3) = (p.at(0)?, p.at(1)?, p.at(2)?, p.at(3)?);
    let side01 = side(p0.x, p0.y, p1.x, p1.y, p2.x, p2.y);
    let side12 = side(p1.x, p1.y, p2.x, p2.y, p3.x, p3.y);
    Ok((side01 == RIGHT_SIDE && side12 == RIGHT_SIDE)
        || (side01 == RIGHT_SIDE && side12 == COLINEAR)
        || (side01 == COLINEAR && side12 == RIGHT_SIDE))
}

/// An open arrowhead (two barbs) at `from`, pointing along `from -> to`.
fn push_line_arrowhead(points: &mut Vec<Pt>, counter: &mut usize, from: Pt, to: Pt, dpi: f64) {
    let length = ((to.x - from.x) * (to.x - from.x) + (to.y - from.y) * (to.y - from.y)).sqrt();
    let diag = clamp_size(length / 8.0, dpi);
    let angle = (to.y - from.y).atan2(to.x - from.x);
    let barbs = draw_endpiece_deltas_double(
        from,
        diag * (angle - PI / 4.0).cos(),
        diag * (angle - PI / 4.0).sin(),
        diag * (angle + PI / 4.0).cos(),
        diag * (angle + PI / 4.0).sin(),
    );
    for (k, p) in barbs.iter().enumerate() {
        put(points, *counter, styled(*p, if k % 2 == 1 { 5 } else { 0 }));
        *counter += 1;
    }
}

/// Upstream `GetDISMRIPDouble`: two lines joined by a semicircle, with
/// arrowheads (one for DEMONSTRATE and MOBILE_DEFENSE, two for RIP).
pub(crate) fn get_dism_rip_double(
    points: &mut Vec<Pt>,
    line_type: i32,
    settings: &Settings,
) -> Result<i32, EngineError> {
    let clockwise = is_clockwise(points)?;
    let sp = [points.at(0)?, points.at(1)?, points.at(2)?, points.at(3)?];
    let mut counter = 0;
    push_segment(points, &mut counter, sp[0], sp[1]);
    push_segment(points, &mut counter, sp[2], sp[3]);
    let dpi = settings.dpi_scale_factor();
    push_line_arrowhead(points, &mut counter, sp[0], sp[1], dpi);
    if line_type == lt::RIP {
        push_line_arrowhead(points, &mut counter, sp[2], sp[3], dpi);
    }
    let radius = ((sp[2].x - sp[1].x) * (sp[2].x - sp[1].x)
        + (sp[2].y - sp[1].y) * (sp[2].y - sp[1].y))
        .sqrt()
        / 2.0;
    let center = Pt::new((sp[1].x + sp[2].x) / 2.0, (sp[1].y + sp[2].y) / 2.0);
    let bounds = (
        center.x - radius,
        center.y - radius,
        center.x + radius,
        center.y + radius,
    );
    let (a, b) = if clockwise {
        (sp[1], sp[2])
    } else {
        (sp[2], sp[1])
    };
    let arc = arc_approximation_double(bounds, (a.x, a.y), (b.x, b.y));
    for (k, p) in arc.iter().enumerate() {
        put(points, counter, styled(*p, if k == 16 { 5 } else { 0 }));
        counter += 1;
    }
    Ok(counter as i32)
}

/// Upstream `GetDISMByDifDouble`: BYDIF, an open rectangle with a jagged
/// back line and two filled barb triangles.
pub(crate) fn get_dism_by_dif_double(
    points: &mut Vec<Pt>,
    settings: &Settings,
) -> Result<i32, EngineError> {
    let save = [points.at(0)?, points.at(1)?, points.at(2)?];
    let rect = draw_open_rectangle_double(&save)?;
    let back = [rect[1], rect[2]];
    let mut counter = 0;
    for (k, p) in rect.iter().enumerate() {
        put(points, counter, styled(*p, if k % 2 == 1 { 5 } else { 0 }));
        counter += 1;
    }
    push_jaggy_line(points, &mut counter, back[0], back[1], settings);
    push_filled_barbs(points, &mut counter, &save, settings)?;
    Ok(counter as i32)
}

/// Upstream `GetDISMPenetrateDouble`: PENETRATE, a line with a long arrow
/// from its midpoint to the third control point; 7 points.
pub(crate) fn get_dism_penetrate_double(
    points: &mut Vec<Pt>,
    settings: &Settings,
) -> Result<(), EngineError> {
    let save = [points.at(0)?, points.at(1)?, points.at(2)?];
    let first = points.at_mut(0)?;
    first.x = save[0].x;
    first.y = save[0].y;
    first.style = 0;
    let second = points.at_mut(1)?;
    second.x = save[1].x;
    second.y = save[1].y;
    second.style = 5;
    let mid = mid_point_double(save[0], save[1], 0);
    put(points, 2, save[2]);
    put(points, 3, styled(mid, 5));
    let mut d = mbr_distance(&save, 3)?;
    let dpi = settings.dpi_scale_factor();
    if d / 5.0 > MAX_LENGTH * dpi {
        d = 5.0 * MAX_LENGTH * dpi;
    }
    if d / 5.0 < MIN_LENGTH * dpi {
        d = 5.0 * MIN_LENGTH * dpi;
    }
    if d < 150.0 * dpi {
        d = 150.0 * dpi;
    }
    if d > 600.0 * dpi {
        d = 600.0 * dpi;
    }
    let mut arrow = [Pt::default(); 3];
    let size = d as i32 / 20;
    get_arrow_head4_double(save[2], styled(mid, 5), size, size, &mut arrow, 0)?;
    for (k, p) in arrow.iter().enumerate() {
        put(points, 4 + k, *p);
    }
    Ok(())
}
