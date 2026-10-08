//! Port of the point-to-line relations of lineutility.java: closest points,
//! projections onto a line and polygon-edge intersection.

use super::basics::mid_point_double;
use super::extend::extend_line_double;
use super::line_types as lt;
use super::slope::{calc_true_intersect_double2, calc_true_slope_double};
use crate::engine::base::{At, EngineError, Pt};

/// Upstream `ClosestPointOnLine`: the point of the segment pt0-pt1 closest to
/// `pt_relative`.
pub(crate) fn closest_point_on_line(pt0: Pt, pt1: Pt, pt_relative: Pt) -> Pt {
    if pt0.x == pt_relative.x && pt0.y == pt_relative.y {
        return pt0;
    }
    if pt1.x == pt_relative.x && pt1.y == pt_relative.y {
        return pt1;
    }
    if pt0.x == pt1.x && pt0.y == pt1.y {
        return pt0;
    }
    let atob = Pt::new(pt1.x - pt0.x, pt1.y - pt0.y);
    let atop = Pt::new(pt_relative.x - pt0.x, pt_relative.y - pt0.y);
    let len = atob.x * atob.x + atob.y * atob.y;
    let dot = atop.x * atob.x + atop.y * atob.y;
    let t = (dot / len).clamp(0.0, 1.0);
    Pt::new(pt0.x + atob.x * t, pt0.y + atob.y * t)
}

/// Upstream `getIntersectionPoint`: where segments pt1-pt2 and pt3-pt4 cross.
fn get_intersection_point(pt1: Pt, pt2: Pt, pt3: Pt, pt4: Pt) -> Option<Pt> {
    let denom = (pt4.y - pt3.y) * (pt2.x - pt1.x) - (pt4.x - pt3.x) * (pt2.y - pt1.y);
    if denom == 0.0 {
        return None;
    }
    let ua = ((pt4.x - pt3.x) * (pt1.y - pt3.y) - (pt4.y - pt3.y) * (pt1.x - pt3.x)) / denom;
    let ub = ((pt2.x - pt1.x) * (pt1.y - pt3.y) - (pt2.y - pt1.y) * (pt1.x - pt3.x)) / denom;
    if (0.0..=1.0).contains(&ua) && (0.0..=1.0).contains(&ub) {
        return Some(Pt::new(
            pt1.x + ua * (pt2.x - pt1.x),
            pt1.y + ua * (pt2.y - pt1.y),
        ));
    }
    None
}

/// Upstream `intersectPolygon`: the first crossing of segment pt0-pt1 with a
/// polygon edge, in edge order.
pub(crate) fn intersect_polygon(poly_pts: &[Pt], pt0: Pt, pt1: Pt) -> Option<Pt> {
    let n = poly_pts.len();
    (0..n).find_map(|i| {
        let a = poly_pts.get(i).copied()?;
        let b = poly_pts.get((i + 1) % n).copied()?;
        get_intersection_point(a, b, pt0, pt1)
    })
}

/// The projection of `pt_relative` onto the perpendicular to pt0-pt1 through
/// `at`, shared by both `PointRelativeToLine` overloads.
fn project_onto_perpendicular(pt0: Pt, pt1: Pt, at: Pt, pt_relative: Pt) -> Pt {
    let mut result = pt0;
    let (vertical, m) = calc_true_slope_double(pt0, pt1);
    if vertical == 0 {
        result.x = pt_relative.x;
        result.y = at.y;
    }
    if vertical != 0 && m == 0.0 {
        result.x = at.x;
        result.y = pt_relative.y;
    }
    if vertical != 0 && m != 0.0 {
        let b1 = at.y + (1.0 / m) * at.x;
        let b2 = pt_relative.y - m * pt_relative.x;
        result = calc_true_intersect_double2(-1.0 / m, b1, m, b2, 1, 1, (0.0, 0.0));
    }
    result
}

/// Upstream `PointRelativeToLine(pt0, pt1, ptRelative)`: where the
/// perpendicular to pt0-pt1 through its midpoint meets the line parallel to
/// pt0-pt1 through `pt_relative`.
pub(crate) fn point_relative_to_line(pt0: Pt, pt1: Pt, pt_relative: Pt) -> Pt {
    let mid = mid_point_double(pt0, pt1, 0);
    project_onto_perpendicular(pt0, pt1, mid, pt_relative)
}

/// Upstream `PointRelativeToLine(pt0, pt1, atPoint, ptRelative)`: as
/// [`point_relative_to_line`] with the perpendicular taken through `at_point`.
pub(crate) fn point_relative_to_line_at(pt0: Pt, pt1: Pt, at_point: Pt, pt_relative: Pt) -> Pt {
    project_onto_perpendicular(pt0, pt1, at_point, pt_relative)
}

/// Upstream `adjustCATKBYFIREControlPoint`: pulls the last (control) point
/// in so it lies `dist` pixels from the projection of the line's first leg.
/// Other line types and short legs are left unchanged.
pub(crate) fn adjust_catkbyfire_control_point(
    linetype: i32,
    pts: &mut [Pt],
    dist: f64,
) -> Result<(), EngineError> {
    if linetype != lt::CATKBYFIRE {
        return Ok(());
    }
    let dist2 = super::basics::calc_distance_double(pts.at(0)?, pts.at(1)?);
    if dist2 <= dist {
        return Ok(());
    }
    let pt0 = pts.at(0)?;
    let pt1 = pts.at(1)?;
    let last = pts
        .len()
        .checked_sub(1)
        .ok_or(EngineError::Degenerate("no control point"))?;
    let control = pts.at(last)?;
    let pt4 = point_relative_to_line_at(pt0, pt1, pt1, control);
    *pts.at_mut(last)? = extend_line_double(pt4, control, dist);
    Ok(())
}

/// Upstream `LineRelativeToLine`: sets the x/y of `pt2` and `pt3` to the feet
/// of the perpendiculars from pt0 and pt1 onto the parallel through
/// `pt_relative`.
pub(crate) fn line_relative_to_line(pt0: Pt, pt1: Pt, pt_relative: Pt, pt2: &mut Pt, pt3: &mut Pt) {
    let (vertical, m) = calc_true_slope_double(pt0, pt1);
    if vertical == 0 {
        pt2.x = pt_relative.x;
        pt2.y = pt0.y;
        pt3.x = pt_relative.x;
        pt3.y = pt1.y;
    }
    if vertical != 0 && m == 0.0 {
        pt2.x = pt0.x;
        pt2.y = pt_relative.y;
        pt3.x = pt1.x;
        pt3.y = pt_relative.y;
    }
    if vertical != 0 && m != 0.0 {
        let b2 = pt_relative.y - m * pt_relative.x;
        let b1 = pt0.y + (1.0 / m) * pt0.x;
        let t2 = calc_true_intersect_double2(-1.0 / m, b1, m, b2, 1, 1, (0.0, 0.0));
        let b1 = pt1.y + (1.0 / m) * pt1.x;
        let t3 = calc_true_intersect_double2(-1.0 / m, b1, m, b2, 1, 1, (0.0, 0.0));
        pt2.x = t2.x;
        pt2.y = t2.y;
        pt3.x = t3.x;
        pt3.y = t3.y;
    }
}

/// Upstream `FindClosestPointOnLine`: the projection of `p3` onto the
/// infinite line through p1 and p2.
pub(crate) fn find_closest_point_on_line(p1: Pt, p2: Pt, p3: Pt) -> Pt {
    let dx_line = p2.x - p1.x;
    let dy_line = p2.y - p1.y;
    let dx3 = p3.x - p1.x;
    let dy3 = p3.y - p1.y;
    let uv = dx3 * dx_line + dy3 * dy_line;
    let uu = dx_line * dx_line + dy_line * dy_line;
    let scalar = uv / uu;
    Pt::new(p1.x + scalar * dx_line, p1.y + scalar * dy_line)
}
