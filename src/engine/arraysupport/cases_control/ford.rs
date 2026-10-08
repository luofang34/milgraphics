//! FORDIF and the anti-tank ditches.

use crate::engine::arraysupport::work::{Work, get, set, set_style};
use crate::engine::base::{EngineError, Pt};
use crate::engine::lineutility::basics::{
    calc_distance_double, mid_point_double, reverse_points_double2,
};
use crate::engine::lineutility::ditch::get_ditch_spike_double;
use crate::engine::lineutility::extend::extend_line2_double;
use crate::engine::lineutility::relative::line_relative_to_line;
use crate::engine::tactical_lines as lt;

/// FORDIF: two parallel banks with a zigzag across the middle.
pub(crate) fn ford_difficult(w: &mut Work<'_>) -> Result<(), EngineError> {
    let (p0, p1, p2) = (get(&w.p, 0)?, get(&w.p, 1)?, get(&w.p, 2)?);
    let mut pt4 = p0;
    let mut pt5 = p0;
    line_relative_to_line(p0, p1, p2, &mut pt4, &mut pt5);
    set(&mut w.p, 2, pt5)?;
    set(&mut w.p, 3, pt4)?;
    for j in 0..w.vbl {
        set_style(&mut w.p, j, 1)?;
    }
    let pt0 = mid_point_double(get(&w.p, 0)?, get(&w.p, 1)?, 0);
    let pt1 = mid_point_double(get(&w.p, 2)?, get(&w.p, 3)?, 0);
    let mid = mid_point_double(pt0, pt1, 0);
    if calc_distance_double(mid, pt0) > calc_distance_double(mid, pt1) {
        line_relative_to_line(get(&w.p, 2)?, get(&w.p, 3)?, pt0, &mut pt4, &mut pt5);
        set(&mut w.p, 0, Pt::styled(pt5.x, pt5.y, 1))?;
        set(&mut w.p, 1, Pt::styled(pt4.x, pt4.y, 1))?;
    } else {
        line_relative_to_line(get(&w.p, 0)?, get(&w.p, 1)?, pt1, &mut pt4, &mut pt5);
        set(&mut w.p, 2, Pt::styled(pt5.x, pt5.y, 1))?;
        set(&mut w.p, 3, Pt::styled(pt4.x, pt4.y, 1))?;
    }
    let spike = w.scaled(10.0);
    let (q0, q3) = (get(&w.p, 0)?, get(&w.p, 3)?);
    let pt2 = extend_line2_double(q0, pt0, -spike, 0);
    let pt3 = extend_line2_double(q3, pt1, -spike, 0);
    let pt4 = extend_line2_double(q0, pt0, spike, 0);
    let pt5 = extend_line2_double(q3, pt1, spike, 0);
    let width = calc_distance_double(pt0, pt1);
    let mut count = 4;
    set(&mut w.p, count, Pt { style: 0, ..pt0 })?;
    count += 1;
    let mut n = 1;
    let mut extend = 0.0;
    while extend < width - spike {
        extend = f64::from(n) * spike / 2.0;
        set(
            &mut w.p,
            count,
            extend_line2_double(pt2, pt3, extend - width, 0),
        )?;
        count += 1;
        n += 1;
        extend = f64::from(n) * spike / 2.0;
        set(
            &mut w.p,
            count,
            extend_line2_double(pt4, pt5, extend - width, 0),
        )?;
        count += 1;
        if count >= i32::try_from(w.p.len()).unwrap_or(i32::MAX) - 1 {
            break;
        }
        n += 1;
    }
    set(&mut w.p, count, Pt { style: 5, ..pt1 })?;
    w.ac = count + 1;
    Ok(())
}

/// ATDITCH, ATDITCHC, ATDITCHM: spikes along the line.
pub(crate) fn ditch(w: &mut Work<'_>) -> Result<(), EngineError> {
    if w.line_type == lt::ATDITCHM {
        reverse_points_double2(&mut w.p, w.save)?;
    }
    if w.line_type != lt::ATDITCH {
        set_style(&mut w.p, 0, 9)?;
    }
    let (spike, radius) = (w.scaled(12.0), w.scaled(4.0));
    w.ac = get_ditch_spike_double(w.line_type, spike, radius, &mut w.p, w.save, 0)?;
    Ok(())
}
