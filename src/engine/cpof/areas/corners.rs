//! The point builders of `Change1TacticalAreas` for rectangles, routes,
//! tracks, orbits, circles, ellipses and poly-arcs. Each appends to
//! `tg.pixels`.

use super::super::numeric_fields::NumericFields;
use super::super::planar::{
    azimuth, coordinate, distance_m, geo_ellipse, geodesic_arc, geodesic_arc2,
};
use crate::engine::base::{At, EngineError, Pt};
use crate::engine::java_text::{is_number, parse_double};
use crate::engine::lineutility::basics::calc_center_point_double2;
use crate::engine::tg::Tg;
use crate::engine::tg_utility::segment_colors::java_split;

fn styled(mut p: Pt, style: i32) -> Pt {
    p.style = style;
    p
}

/// Ellipse about the first point (`LAUNCH_AREA` and kin).
pub(super) fn ellipse(tg: &mut Tg, pt0: Pt, f: &NumericFields, mpp: f64) {
    for p in geo_ellipse(pt0, f.width, f.length, f.attitude[0], mpp) {
        tg.pixels.push(p);
    }
}

/// Rectangle `width` wide about the segment `pt0`-`pt1` (the PAA, FSA, ...
/// `_RECTANGULAR` family), closed by repeating the first corner.
pub(super) fn centered_rectangle(tg: &mut Tg, pt0: Pt, pt1: Pt, f: &NumericFields, mpp: f64) {
    let half = f.width / 2.0;
    let att = f.attitude[0];
    let first = coordinate(pt0, half, att - 90.0, mpp);
    tg.pixels.push(first);
    tg.pixels.push(coordinate(pt0, half, att + 90.0, mpp));
    tg.pixels.push(coordinate(pt1, half, att + 90.0, mpp));
    tg.pixels.push(coordinate(pt1, half, att - 90.0, mpp));
    tg.pixels.push(first);
}

/// `BS_ORBIT`: two parallel sides closed by half circles at both ends.
pub(super) fn orbit(tg: &mut Tg, pt0: Pt, pt1: Pt, f: &NumericFields, mpp: f64) {
    let half = f.width / 2.0;
    let att = f.attitude[0];
    tg.pixels.push(coordinate(pt0, half, att - 90.0, mpp));
    let end = coordinate(pt1, half, att - 90.0, mpp);
    let arc = geodesic_arc(pt1, end, end, mpp);
    tg.pixels
        .extend(arc.iter().take(arc.len() / 2).map(|p| styled(*p, 0)));
    let start = coordinate(pt0, half, att + 90.0, mpp);
    let arc = geodesic_arc(pt0, start, start, mpp);
    tg.pixels
        .extend(arc.iter().take(arc.len() / 2).map(|p| styled(*p, 0)));
}

/// `BS_ROUTE` and `BS_TRACK`: one rectangle per leg. The route has one width
/// per leg, the track a left and right width per leg; widths run out by
/// repeating the last one for the route only, so a short track list fails as
/// upstream's index error does.
pub(super) fn legs(tg: &mut Tg, control: &[Pt], track: bool, mpp: f64) -> Result<(), EngineError> {
    let mut am: Vec<String> = java_split(&tg.am, ',')
        .into_iter()
        .map(str::to_owned)
        .collect();
    while am.len() < control.len().saturating_sub(1) {
        let last = am
            .last()
            .cloned()
            .ok_or(EngineError::Index { index: -1, len: 0 })?;
        am.push(last);
    }
    let width = |i: usize| -> Result<f64, EngineError> {
        let text = am.get(i).ok_or(EngineError::Index {
            index: i64::try_from(i).unwrap_or(i64::MAX),
            len: am.len(),
        })?;
        Ok(if is_number(text) {
            parse_double(text)?
        } else {
            0.0
        })
    };
    for i in 0..control.len().saturating_sub(1) {
        let (pt0, pt1) = (control.at(i)?, control.at(i + 1)?);
        let att = azimuth(pt0, pt1);
        let (left, right) = if track {
            (width(2 * i)?, width(2 * i + 1)?)
        } else {
            let w = width(i)? / 2.0;
            (w, w)
        };
        let first = coordinate(pt0, left, att - 90.0, mpp);
        tg.pixels.push(first);
        tg.pixels.push(coordinate(pt0, right, att + 90.0, mpp));
        tg.pixels.push(coordinate(pt1, right, att + 90.0, mpp));
        tg.pixels.push(coordinate(pt1, left, att - 90.0, mpp));
        tg.pixels.push(styled(first, 5));
    }
    Ok(())
}

/// `RECTANGULAR_TARGET`: the rectangle and a cross at its centre.
pub(super) fn rectangular_target(
    tg: &mut Tg,
    pt0: Pt,
    pt1: Pt,
    f: &NumericFields,
    mpp: f64,
) -> Result<(), EngineError> {
    let half = f.width / 2.0;
    let att = f.attitude[0];
    let corners = [
        coordinate(pt0, half, att - 90.0, mpp),
        coordinate(pt0, half, att + 90.0, mpp),
        coordinate(pt1, half, att + 90.0, mpp),
        coordinate(pt1, half, att - 90.0, mpp),
    ];
    for c in corners {
        tg.pixels.push(c);
    }
    tg.pixels.push(styled(corners[0], 5));
    let height = distance_m(corners[0], corners[1], mpp);
    let width = distance_m(corners[1], corners[2], mpp);
    let cross = height.min(width) * 0.4;
    let center = calc_center_point_double2(&corners, 4)?;
    tg.pixels.push(coordinate(center, cross, 0.0, mpp));
    tg.pixels
        .push(styled(coordinate(center, cross, 180.0, mpp), 5));
    tg.pixels.push(coordinate(center, cross, -90.0, mpp));
    tg.pixels.push(coordinate(center, cross, 90.0, mpp));
    Ok(())
}

/// `RECTANGULAR`, `PBS_RECTANGLE`, `PBS_SQUARE`, `CUED_ACQUISITION`: a
/// `length` by `width` rectangle about the first point, rotated by the
/// attitude.
pub(super) fn oriented_rectangle(tg: &mut Tg, pt0: Pt, f: &NumericFields, mpp: f64) {
    let (half_l, half_w, att) = (f.length / 2.0, f.width / 2.0, f.attitude[0]);
    let corner = |side: f64, along: f64| {
        let p = coordinate(pt0, half_l, att + side, mpp);
        coordinate(p, half_w, att + along, mpp)
    };
    let corners = [
        corner(-90.0, 0.0),
        corner(90.0, 0.0),
        corner(90.0, 180.0),
        corner(-90.0, 180.0),
    ];
    tg.pixels.extend(corners);
    tg.pixels.push(Pt::new(corners[0].x, corners[0].y));
}

/// The circle family: a full circle of `radius` metres about the first point.
pub(super) fn circle(tg: &mut Tg, pt0: Pt, f: &NumericFields, mpp: f64) {
    let edge = coordinate(pt0, f.radius, 90.0, mpp);
    tg.pixels.extend(geodesic_arc(pt0, edge, edge, mpp));
}

/// `BS_POLYARC`: an arc about the first point from the first to the second
/// azimuth, then the remaining anchor points, closed on the arc's start.
pub(super) fn poly_arc(
    tg: &mut Tg,
    control: &mut Vec<Pt>,
    f: &NumericFields,
    mpp: f64,
) -> Result<(), EngineError> {
    // Anchor points must run counter-clockwise on the ground; pixel y runs
    // down, so the sign flips relative to latitude/longitude.
    if signed_area_on_ground(control) < 0.0 && !control.is_empty() {
        let first = control.remove(0);
        control.reverse();
        control.insert(0, first);
    }
    let pt0 = control.at(0)?;
    let from = coordinate(pt0, f.length, f.attitude[0], mpp);
    let to = coordinate(pt0, f.length, f.attitude[1], mpp);
    let (arc, _) = geodesic_arc2(pt0, from, to, mpp);
    tg.pixels.extend(arc);
    tg.pixels.extend(control.iter().skip(1).copied());
    let first = tg.pixels.at(0)?;
    tg.pixels.push(first);
    Ok(())
}

/// Upstream `CalculateSignedAreaOfPolygon`, positive for counter-clockwise
/// on the ground (y up), computed from pixel coordinates (y down).
fn signed_area_on_ground(pts: &[Pt]) -> f64 {
    let n = pts.len();
    let mut sum = 0.0;
    for i in 0..n {
        if let (Some(a), Some(b)) = (pts.get(i), pts.get((i + 1) % n)) {
            sum += a.x * b.y - b.x * a.y;
        }
    }
    -sum / 2.0
}
