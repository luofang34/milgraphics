//! Port of `clsUtilityCPOF.Change1TacticalAreas`: the graphics whose outline
//! comes from AM/AN amplifiers (circles, rectangles, ellipses, range fans,
//! tracks, orbits) rather than from the anchor points alone.

mod corners;

use super::groups::{CIRCLES, ELLIPSES, SEGMENT_RECTANGLES};
use super::numeric_fields::{NumericFields, get_numeric_fields};
use super::planar::coordinate;
use super::range_fan::{get_concentric_circles, get_sector_range_fan, range_fan_orientation};
use super::shapes::change1_pixels_to_shapes;
use crate::engine::base::{At, EngineError, Pt, Shape, shape_type};
use crate::engine::lineutility::basics::calc_distance_double;
use crate::engine::lineutility::exterior::get_exterior_points;
use crate::engine::tactical_lines::*;
use crate::engine::tg::Tg;

/// Upstream `arraysupport.GetInsideOutsideDouble2`, called as `(pt0, pt1,
/// all points, count, index, line type)`; the PBS rectangles need it.
pub(crate) type InsideOutside<'a> =
    &'a dyn Fn(Pt, Pt, &[Pt], i32, i32, i32) -> Result<i32, EngineError>;

/// Upstream `Change1TacticalAreas`. `control` are the anchor points
/// (upstream's `LatLongs`); `tg.pixels` is replaced by the generated outline
/// points, which later label placement reads. Returns false for a line type
/// this does not handle, and, like upstream's catch-all, when a step fails.
///
/// Not ported: the 360 degree wrap of `GetFarPixels` (the pixel frame has no
/// world wrap; the geographic tier is split elsewhere) and `postSegmentFSA`
/// (geodesic densification of sides over 250 km).
pub(crate) fn change1_tactical_areas(
    tg: &mut Tg,
    line_type: i32,
    control: &[Pt],
    meters_per_pixel: f64,
    shapes: &mut Vec<Shape>,
    inside_outside: InsideOutside<'_>,
) -> bool {
    tg.pixels.clear();
    build(
        tg,
        line_type,
        control,
        meters_per_pixel,
        shapes,
        inside_outside,
    )
    .unwrap_or(false)
}

fn build(
    tg: &mut Tg,
    line_type: i32,
    control: &[Pt],
    mpp: f64,
    shapes: &mut Vec<Shape>,
    inside_outside: InsideOutside<'_>,
) -> Result<bool, EngineError> {
    let pt0 = control.at(0)?;
    let pt1 = control.get(1).copied().unwrap_or(pt0);
    let center = pt0;
    let f = get_numeric_fields(tg, line_type, control);
    let mut control_v = control.to_vec();
    if !add_points(tg, line_type, &mut control_v, (pt0, pt1), &f, mpp) {
        return Ok(false);
    }
    // Pixels as the points builder left them, for the later label placement.
    let saved = tg.pixels.clone();
    change1_pixels_to_shapes(tg, shapes, false)?;
    tg.pixels = saved;
    if line_type == BBS_POINT {
        let mut shape = Shape::new(shape_type::POLYLINE);
        shape.move_to(center);
        shape.line_to(Pt::new(center.x, center.y + 1.0));
        shapes.push(shape);
    }
    if line_type == PBS_RECTANGLE || line_type == PBS_SQUARE {
        pbs_fill(tg, control, &f, mpp, shapes, inside_outside)?;
    }
    Ok(true)
}

/// Dispatches to the point builder for `line_type`; false when it has none
/// or the builder fails.
fn add_points(
    tg: &mut Tg,
    line_type: i32,
    control: &mut Vec<Pt>,
    (pt0, pt1): (Pt, Pt),
    f: &NumericFields,
    mpp: f64,
) -> bool {
    let n = control.len();
    match line_type {
        t if ELLIPSES.contains(&t) => corners::ellipse(tg, pt0, f, mpp),
        t if SEGMENT_RECTANGLES.contains(&t) => {
            corners::centered_rectangle(tg, pt0, pt1, f, mpp);
        }
        t if CIRCLES.contains(&t) => corners::circle(tg, pt0, f, mpp),
        BS_ORBIT => corners::orbit(tg, pt0, pt1, f, mpp),
        BS_ROUTE | BS_TRACK => {
            if corners::legs(tg, control, line_type == BS_TRACK, mpp).is_err() {
                return false;
            }
        }
        RECTANGULAR_TARGET => {
            if corners::rectangular_target(tg, pt0, pt1, f, mpp).is_err() {
                return false;
            }
        }
        RECTANGULAR | PBS_RECTANGLE | PBS_SQUARE | CUED_ACQUISITION => {
            corners::oriented_rectangle(tg, pt0, f, mpp);
        }
        RANGE_FAN => {
            get_concentric_circles(tg, control, mpp);
            if n > 1 {
                range_fan_orientation(tg, control, mpp);
            }
        }
        RANGE_FAN_SECTOR => {
            get_sector_range_fan(tg, control, mpp);
            range_fan_orientation(tg, control, mpp);
        }
        RADAR_SEARCH | BS_RADARC | BS_CAKE | RANGE_FAN_FILL => {
            get_sector_range_fan(tg, control, mpp);
        }
        BS_POLYARC => {
            if corners::poly_arc(tg, control, f, mpp).is_err() {
                return false;
            }
        }
        _ => return false,
    }
    true
}

/// The extra fill shape of `PBS_RECTANGLE` and `PBS_SQUARE`: the outline
/// offset outward by the radius.
fn pbs_fill(
    tg: &mut Tg,
    control: &[Pt],
    f: &NumericFields,
    mpp: f64,
    shapes: &mut Vec<Shape>,
    inside_outside: InsideOutside<'_>,
) -> Result<(), EngineError> {
    let origin = control.at(0)?;
    let probe = coordinate(origin, f.radius, 45.0, mpp);
    let dist = calc_distance_double(origin, probe);
    let original = tg.pixels.clone();
    let mut pts = original.clone();
    let count = i32::try_from(pts.len()).unwrap_or(i32::MAX);
    pts.at_mut(0)?.style = dist as i32;
    get_exterior_points(&mut pts, count, tg.line_type, false, inside_outside)?;
    tg.pixels = pts.iter().map(|p| Pt::new(p.x, p.y)).collect();
    change1_pixels_to_shapes(tg, shapes, true)?;
    tg.pixels = original;
    Ok(())
}
