//! MOBILE_DEFENSE: the RIP drawing plus two filled triangles on the arc and
//! two on the sides.

use crate::engine::arraysupport::work::{Work, get, set};
use crate::engine::base::{EngineError, Pt};
use crate::engine::dism::rip::get_dism_rip_double;
use crate::engine::lineutility::EXTEND_ABOVE;
use crate::engine::lineutility::EXTEND_BELOW;
use crate::engine::lineutility::basics::{calc_distance_double, mid_point_double};
use crate::engine::lineutility::extend::{
    extend_along_line_double_style, extend_along_line_double2, extend_directed_line,
};
use crate::engine::lineutility::relative::point_relative_to_line_at;

/// Appends the fill triangle on the arc point `c`, pushed out to `tip`.
fn arc_spike(tri: &mut Vec<Pt>, p: &[Pt], c: i32, tip: Pt) -> Result<(), EngineError> {
    let filled = |pt: Pt, style: i32| Pt { style, ..pt };
    tri.push(filled(get(p, c - 1)?, 9));
    tri.push(filled(tip, 9));
    tri.push(filled(get(p, c + 1)?, 9));
    tri.push(filled(get(p, c)?, 9));
    tri.push(filled(get(p, c - 1)?, 10));
    Ok(())
}

/// The tip of the spike on arc point `c`, truncated to whole pixels.
fn spike_tip(p: &[Pt], c: i32, center: Pt, radius: f64, length: f64) -> Result<Pt, EngineError> {
    let pc = get(p, c)?;
    let k = radius / length;
    Ok(Pt::new(
        center.x - ((k * (center.x - pc.x)) as i64) as f64,
        center.y - ((k * (center.y - pc.y)) as i64) as f64,
    ))
}

/// The side triangle built on the line `from`-`to` of the control box.
fn side_triangle(
    tri: &mut Vec<Pt>,
    from: Pt,
    to: Pt,
    side: (f64, f64, f64, i32),
) -> Result<(), EngineError> {
    let (offset, base, height, direction) = side;
    let a = Pt {
        style: 9,
        ..extend_along_line_double_style(from, to, offset, 9)
    };
    let b = Pt {
        style: 9,
        ..extend_along_line_double2(a, to, base)
    };
    let mid = mid_point_double(a, b, 0);
    let c = Pt {
        style: 9,
        ..extend_directed_line(a, b, mid, direction, height)
    };
    tri.extend([a, b, c, Pt { style: 10, ..a }]);
    Ok(())
}

/// MOBILE_DEFENSE.
pub(crate) fn mobile_defense(w: &mut Work<'_>) -> Result<(), EngineError> {
    let (pt0, pt1, pt2) = (w.pt0, w.pt1, w.pt2);
    set(&mut w.p, 2, point_relative_to_line_at(pt0, pt1, pt1, pt2))?;
    set(&mut w.p, 3, point_relative_to_line_at(pt0, pt1, pt0, pt2))?;
    w.ac = get_dism_rip_double(&mut w.p, w.line_type, w.settings)?;
    let (p1, p2) = (get(&w.p, 1)?, get(&w.p, 2)?);
    let radius = calc_distance_double(p1, p2) / 2.0;
    let center = mid_point_double(p1, p2, 0);
    let mut length = (radius - 20.0).abs();
    if radius < 40.0 {
        length = radius / 1.5;
    }
    if radius > 100.0 {
        length = 0.8 * radius;
    }
    let mut tri: Vec<Pt> = Vec::with_capacity(18);
    for c in [10, 22] {
        let tip = spike_tip(&w.p, c, center, radius, length)?;
        arc_spike(&mut tri, &w.p, c, tip)?;
    }
    let (t0, t1, t2, t3) = (get(&tri, 0)?, get(&tri, 1)?, get(&tri, 2)?, get(&tri, 3)?);
    let base = calc_distance_double(t0, t2);
    let height = calc_distance_double(t1, t3);
    let eighth = calc_distance_double(pt0, pt1) / 8.0;
    let (p0, p1, p2, p3) = (get(&w.p, 0)?, get(&w.p, 1)?, get(&w.p, 2)?, get(&w.p, 3)?);
    side_triangle(&mut tri, p3, p2, (eighth, base, height, EXTEND_ABOVE))?;
    side_triangle(&mut tri, p0, p1, (eighth, base, height, EXTEND_BELOW))?;
    for t in tri {
        set(&mut w.p, w.ac, t)?;
        w.ac += 1;
    }
    Ok(())
}
