//! ANCHORAGE_AREA, ANCHORAGE_LINE, UNDERCAST and MVFR: flots on one side of
//! the line, with the points reversed so the flots face the right way.

use super::super::inside_outside::get_inside_outside_double2;
use super::super::work::{Work, at_mut, get, set, set_style};
use crate::engine::base::{EngineError, Pt};
use crate::engine::flot::anchorage::{get_anchorage_count_double, get_anchorage_flot_segment};
use crate::engine::flot::flot_line::get_flot_double;
use crate::engine::flot::int_coords;
use crate::engine::flot::segment::FlipState;
use crate::engine::lineutility::basics::{
    calc_distance_double, get_quadrant_double, reverse_points_double2,
};
use crate::engine::lineutility::extend::extend_along_line_double;

/// Copies the flot points of one segment into `p`, ending each tenth flot
/// with the remembered tick point; returns the new point count.
fn emit_flots(
    p: &mut [Pt],
    points: &[i32],
    start: i32,
    flot_count: i32,
    pt: &mut Pt,
) -> Result<i32, EngineError> {
    let mut counter = start;
    let mut k = 0_usize;
    let pc = |i: usize| {
        points
            .get(i)
            .copied()
            .map(f64::from)
            .ok_or(EngineError::Index {
                index: i64::try_from(i).unwrap_or(0),
                len: points.len(),
            })
    };
    for j in 0..points.len() / 3 {
        let (x1, y1) = (pc(k)?, pc(k + 1)?);
        k += 3;
        if j % 10 == 0 {
            *pt = Pt::styled(x1, y1, 5);
        } else if (j + 1) % 10 == 0 {
            if counter < flot_count {
                let slot = at_mut(p, counter)?;
                slot.x = x1;
                slot.y = y1;
                set(p, counter + 1, *pt)?;
                counter += 2;
                continue;
            }
            break;
        }
        if counter < flot_count {
            let slot = at_mut(p, counter)?;
            slot.x = x1;
            slot.y = y1;
            counter += 1;
        } else {
            break;
        }
    }
    Ok(counter)
}

/// Upstream `GetAnchorageDouble`: flots with a tick every tenth one; returns
/// the number of points written into `p`.
fn get_anchorage_double(
    p: &mut [Pt],
    float_diameter: f64,
    num_pts: i32,
) -> Result<i32, EngineError> {
    let flot_count = get_anchorage_count_double(p, float_diameter, num_pts)?;
    let n = usize::try_from(num_pts).map_err(|_| EngineError::Degenerate("negative count"))?;
    let vb = int_coords(p, n)?;
    let mut state = FlipState::zeroed();
    let mut pt = Pt::default();
    let mut counter = 0;
    for l in 0..num_pts - 1 {
        let li = usize::try_from(l).unwrap_or(0);
        let coord = |i: usize| {
            vb.get(i).copied().map(f64::from).ok_or(EngineError::Index {
                index: i64::try_from(i).unwrap_or(i64::MAX),
                len: vb.len(),
            })
        };
        let mut pt1 = Pt::new(coord(2 * li)?, coord(2 * li + 1)?);
        let pt2 = Pt::new(coord(2 * li + 2)?, coord(2 * li + 3)?);
        if l > 0 {
            pt1 = extend_along_line_double(pt1, pt2, float_diameter);
        }
        let num_segs = (calc_distance_double(pt1, pt2) / float_diameter) as i32;
        if num_segs <= 0 {
            if counter < flot_count {
                let slot = at_mut(p, counter)?;
                slot.x = coord(2 * li)?;
                slot.y = coord(2 * li + 1)?;
                counter += 1;
            }
            continue;
        }
        let mut points2 = vec![0_i32; usize::try_from(num_segs).unwrap_or(0) * 32];
        let ends = (pt1.x as i32, pt1.y as i32, pt2.x as i32, pt2.y as i32);
        let num_seg_pts =
            get_anchorage_flot_segment(&vb, ends, li, float_diameter, &mut points2, &mut state)?;
        let points: Vec<i32> = points2.iter().copied().take(num_seg_pts).collect();
        if points.len() != num_seg_pts {
            return Err(EngineError::Index {
                index: i64::try_from(num_seg_pts).unwrap_or(0),
                len: points.len(),
            });
        }
        counter = emit_flots(p, &points, counter, flot_count, &mut pt)?;
    }
    for j in counter - 1..flot_count {
        set_style(p, j, 5)?;
    }
    Ok(counter)
}

/// Reverses the points when the first segment's quadrant and the side of
/// the inside disagree with the direction the flots are drawn in. `keep`
/// lists, per quadrant 1 to 4, the two direction codes (of 0 to 3) that
/// keep the order; the other two reverse it.
fn orient(w: &mut Work<'_>, keep: [[i32; 2]; 4]) -> Result<(), EngineError> {
    let (p0, p1) = (get(&w.p, 0)?, get(&w.p, 1)?);
    let n = get_inside_outside_double2(p0, p1, &w.p, w.save, 0, w.line_type)?;
    let quadrant = get_quadrant_double(p0, p1);
    if let Some(keeps) = usize::try_from(quadrant - 1).ok().and_then(|q| keep.get(q)) {
        if (0..4).contains(&n) && !keeps.contains(&n) {
            reverse_points_double2(&mut w.p, w.save)?;
        }
    }
    Ok(())
}

pub(super) fn anchorage_area(w: &mut Work<'_>) -> Result<(), EngineError> {
    orient(w, [[1, 3], [0, 3], [0, 2], [1, 2]])?;
    let d = w.scaled(20.0);
    w.ac = get_anchorage_double(&mut w.p, d, w.save)?;
    Ok(())
}

pub(super) fn anchorage_line(w: &mut Work<'_>) -> Result<(), EngineError> {
    reverse_points_double2(&mut w.p, w.save)?;
    let d = w.scaled(20.0);
    w.ac = get_anchorage_double(&mut w.p, d, w.save)?;
    Ok(())
}

pub(super) fn undercast(w: &mut Work<'_>) -> Result<(), EngineError> {
    if get(&w.p, 0)?.x < get(&w.p, 1)?.x {
        reverse_points_double2(&mut w.p, w.save)?;
    }
    let d = w.scaled(20.0);
    w.ac = get_flot_double(&mut w.p, d, w.save)?;
    Ok(())
}

pub(super) fn mvfr(w: &mut Work<'_>) -> Result<(), EngineError> {
    orient(w, [[0, 2], [1, 2], [1, 3], [0, 3]])?;
    let d = w.scaled(20.0);
    w.ac = get_flot_double(&mut w.p, d, w.save)?;
    Ok(())
}
