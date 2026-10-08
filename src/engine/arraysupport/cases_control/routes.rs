//! MSR, ASR and traffic route lines: the route with direction arrows
//! beside each long segment.

use crate::engine::arraysupport::supply_route_arrow_side;
use crate::engine::arraysupport::work::{MIN_LENGTH, Work, get, set, set_style};
use crate::engine::base::{EngineError, Pt};
use crate::engine::lineutility::arrow::get_arrow_head4_double;
use crate::engine::lineutility::basics::calc_distance_double;
use crate::engine::lineutility::extend::{extend_directed_line, extend_line2_double};
use crate::engine::tactical_lines as lt;

/// Appends the three points of an arrowhead from `from` to `to`.
fn push_arrow(
    w: &mut Work<'_>,
    n: &mut i32,
    from: Pt,
    to: Pt,
    size: i32,
) -> Result<(), EngineError> {
    let mut arrow = [Pt::default(); 3];
    get_arrow_head4_double(from, to, size, size, &mut arrow, 0)?;
    for a in arrow {
        set(&mut w.p, *n, a)?;
        *n += 1;
    }
    Ok(())
}

/// The route types: the control points stay, arrows are appended.
pub(crate) fn supply_route(w: &mut Work<'_>) -> Result<(), EngineError> {
    let mut n = w.save;
    set_style(&mut w.p, w.save - 1, 5)?;
    for j in 0..w.save - 1 {
        let (a, b) = (get(&w.p, j)?, get(&w.p, j + 1)?);
        let d = calc_distance_double(a, b);
        if d < 20.0 {
            continue;
        }
        let mut dist_from_line = 10.0 * w.dpi;
        let direction = supply_route_arrow_side(a, b);
        let mut pt2 = extend_line2_double(a, b, -3.0 * d / 4.0, 0);
        let mut pt3 = extend_line2_double(a, b, -d / 4.0, 5);
        pt2 = extend_directed_line(a, b, pt2, direction, dist_from_line);
        pt3 = extend_directed_line(a, b, pt3, direction, dist_from_line);
        set(&mut w.p, n, pt2)?;
        set(&mut w.p, n + 1, pt3)?;
        n += 2;
        let mut size = dist_from_line;
        if w.d_mbr / 20.0 < MIN_LENGTH * w.dpi {
            size = 5.0 * w.dpi;
        }
        let size = size as i32;
        push_arrow(w, &mut n, pt2, pt3, size)?;
        if matches!(
            w.line_type,
            lt::MSR_ALT | lt::ASR_ALT | lt::TRAFFIC_ROUTE_ALT
        ) {
            push_arrow(w, &mut n, pt3, pt2, size)?;
        }
        if matches!(w.line_type, lt::MSR_TWOWAY | lt::ASR_TWOWAY) {
            dist_from_line = 15.0 * w.dpi;
            pt2 = extend_directed_line(a, b, pt2, direction, dist_from_line);
            pt3 = extend_directed_line(a, b, pt3, direction, dist_from_line);
            set(&mut w.p, n, pt2)?;
            set(&mut w.p, n + 1, pt3)?;
            n += 2;
            push_arrow(w, &mut n, pt3, pt2, size)?;
        }
    }
    w.ac = n;
    Ok(())
}
