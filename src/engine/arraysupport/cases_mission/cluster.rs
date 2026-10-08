//! CLUSTER and NAVIGATION.

use crate::engine::arraysupport::work::{Work, get, set, set_style};
use crate::engine::base::{EngineError, Pt};
use crate::engine::lineutility::arc::arc_array_double;
use crate::engine::lineutility::basics::calc_distance_double;
use crate::engine::lineutility::extend::{
    extend_line_double, extend_line2_double, extend_true_line_perp_double,
};
use crate::engine::lineutility::slope::calc_true_slope_double;

/// The point on the perpendicular bisector at `radius` from `mid`, on the
/// side the cluster's arc bulges to.
fn bulge_point(
    w: &Work<'_>,
    pt0: Pt,
    pt1: Pt,
    mid: &mut Pt,
    radius: f64,
) -> Result<Pt, EngineError> {
    let (vertical, m) = calc_true_slope_double(pt0, pt1);
    let (p0, p1) = (get(&w.p, 0)?, get(&w.p, 1)?);
    let mut pt2 = w.pt2;
    if vertical != 0 && m != 0.0 {
        let b = mid.y + (1.0 / m) * mid.x;
        let y_intercept = Pt { x: 0.0, y: b, ..p0 };
        pt2 = extend_line_double(y_intercept, *mid, radius);
        if p0.x <= p1.x {
            if pt2.y >= mid.y {
                pt2 = extend_line_double(y_intercept, *mid, -radius);
            }
        } else if pt2.y <= mid.y {
            pt2 = extend_line_double(y_intercept, *mid, -radius);
        }
    }
    if vertical != 0 && m == 0.0 {
        // Upstream aliases the midpoint, so moving the point moves it too.
        mid.y = if p0.x <= p1.x {
            mid.y - radius
        } else {
            mid.y + radius
        };
        pt2 = *mid;
    }
    if vertical == 0 {
        mid.x = if p0.y <= p1.y {
            mid.x + radius
        } else {
            mid.x - radius
        };
        pt2 = *mid;
    }
    Ok(pt2)
}

/// CLUSTER: the two control points followed by a 26-point arc.
pub(crate) fn cluster(w: &mut Work<'_>) -> Result<(), EngineError> {
    let pt0 = w.pt0;
    let mut pt1 = w.pt1;
    // The two points can sit on top of one another.
    if (pt0.y - pt1.y).abs() < 1.0 {
        pt1.y = pt0.y + 1.0;
    }
    let radius = calc_distance_double(pt0, pt1) / 2.0;
    let mut mid = get(&w.p, 0)?;
    mid.x = (pt1.x + pt0.x) / 2.0;
    mid.y = (pt1.y + pt0.y) / 2.0;
    let pt2 = bulge_point(w, pt0, pt1, &mut mid, radius)?;
    let pt1 = extend_line_double(mid, pt2, 100.0);
    let mut arc = vec![Pt::default(); 26];
    set(&mut arc, 0, pt2)?;
    set(&mut arc, 1, pt1)?;
    arc_array_double(&mut arc, radius, w.line_type)?;
    set_style(&mut w.p, 0, 1)?;
    set_style(&mut w.p, 1, 5)?;
    for (j, a) in (0_i32..).zip(arc) {
        set(&mut w.p, 2 + j, Pt { style: 1, ..a })?;
    }
    w.ac = 28;
    Ok(())
}

/// NAVIGATION: the line with a corner mark beyond each end.
pub(crate) fn navigation(w: &mut Work<'_>) -> Result<(), EngineError> {
    let (pt0, pt1) = (w.pt0, w.pt1);
    let ext = w.scaled(10.0);
    let pt3 = extend_line2_double(pt1, pt0, -ext, 0);
    let pt4 = extend_line2_double(pt0, pt1, -ext, 0);
    let pt5 = extend_true_line_perp_double(pt0, pt1, pt3, ext, 0)?;
    let pt6 = extend_true_line_perp_double(pt0, pt1, pt3, -ext, 0)?;
    let pt7 = extend_true_line_perp_double(pt0, pt1, pt4, ext, 0)?;
    let pt8 = extend_true_line_perp_double(pt0, pt1, pt4, -ext, 0)?;
    set(&mut w.p, 0, if pt5.y < pt6.y { pt5 } else { pt6 })?;
    set(&mut w.p, 3, if pt7.y > pt8.y { pt7 } else { pt8 })?;
    set(&mut w.p, 1, pt0)?;
    set(&mut w.p, 2, pt1)?;
    w.ac = 4;
    Ok(())
}
