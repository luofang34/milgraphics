//! TROUGH, CABLE, SQUALL, ITD and CONVERGENCE: waves and ticks along the line.

use super::super::count_sizes::{squall_qty, squall_seg_qty};
use super::super::work::{Work, at_mut, get, set, set_style};
use crate::engine::base::{EngineError, Pt};
use crate::engine::lineutility::basics::calc_distance_double;
use crate::engine::lineutility::extend::{
    extend_along_line_double, extend_along_line_double_style, extend_along_line_double2,
    extend_directed_line_style,
};
use crate::engine::lineutility::squall::get_squall_segment;

/// Upstream `GetSquallDouble`: returns the number of points.
pub(super) fn get_squall_double(
    p: &mut [Pt],
    amplitude: f64,
    quantity: i32,
    length: f64,
    num_points: i32,
) -> Result<i32, EngineError> {
    let save1 = get(p, 0)?;
    let save2 = get(p, num_points - 1)?;
    let total = squall_qty(p, quantity, length, num_points)?;
    if total == 0 {
        return Ok(0);
    }
    let mut squall = vec![Pt::default(); usize::try_from(total).unwrap_or(0)];
    let mut counter = 0;
    for j in 0..num_points - 1 {
        let (start, end) = (get(p, j)?, get(p, j + 1)?);
        let seg_qty = squall_seg_qty(start, end, quantity, length);
        if seg_qty <= 0 {
            for q in [start, end] {
                let slot = at_mut(&mut squall, counter)?;
                slot.x = q.x;
                slot.y = q.y;
                counter += 1;
            }
            continue;
        }
        let mut seg = vec![Pt::default(); usize::try_from(seg_qty).unwrap_or(0)];
        let mut sign = -1;
        get_squall_segment(start, end, &mut seg, &mut sign, amplitude, quantity, length)?;
        for k in 0..seg_qty {
            let s = get(&seg, k)?;
            let slot = at_mut(&mut squall, counter)?;
            slot.x = s.x;
            slot.y = s.y;
            if k == 0 {
                *slot = get(p, j)?;
            }
            if k == seg_qty - 1 {
                *slot = get(p, j + 1)?;
            }
            slot.style = 0;
            counter += 1;
        }
    }
    for j in 0..counter {
        if j < total {
            let s = get(&squall, j)?;
            let slot = at_mut(p, j)?;
            slot.x = s.x;
            slot.y = s.y;
            if j == 0 {
                *slot = save1;
            }
            if j == counter - 1 {
                *slot = save2;
            }
            slot.style = s.style;
        }
    }
    if counter == 0 {
        for (j, slot) in p.iter_mut().enumerate() {
            *slot = if j == 0 { save1 } else { save2 };
        }
        counter = i32::try_from(p.len()).unwrap_or(i32::MAX);
    }
    Ok(counter)
}

/// Upstream `GetSevereSquall`: returns the number of points.
pub(super) fn get_severe_squall(
    p: &mut [Pt],
    length: f64,
    num_points: i32,
) -> Result<i32, EngineError> {
    let total = squall_qty(p, 5, length, num_points)? + 2 * num_points;
    let mut sq: Vec<Pt> = Vec::new();
    for j in 0..num_points - 1 {
        let (a, b) = (get(p, j)?, get(p, j + 1)?);
        let dist = calc_distance_double(a, b);
        let seg_qty = (dist / length) as i32;
        for k in 0..seg_qty {
            let off = f64::from(k) * length;
            let pt0 = extend_along_line_double2(a, b, off);
            let mut pt1 = extend_along_line_double(a, b, off + length / 6.0 * 4.0);
            pt1.style = 5;
            sq.push(pt0);
            sq.push(pt1);
            let pt5 = extend_along_line_double(a, b, off + length / 6.0 * 5.0);
            let pt6 = extend_along_line_double(a, b, off + length);
            let pt2 = extend_directed_line_style(pt0, pt1, pt1, 2, length / 6.0, 0);
            let pt3 = extend_directed_line_style(pt0, pt5, pt5, 3, length / 6.0, 0);
            let mut pt4 = extend_directed_line_style(pt0, pt6, pt6, 2, length / 6.0, 5);
            pt4.style = 5;
            sq.extend([pt2, pt3, pt4]);
        }
        sq.push(b);
        let mut tail = extend_along_line_double(b, a, dist - f64::from(seg_qty) * length);
        tail.style = 5;
        sq.push(tail);
    }
    let mut l = i32::try_from(sq.len()).unwrap_or(i32::MAX);
    if l > total {
        return Err(EngineError::Index {
            index: i64::from(total),
            len: sq.len(),
        });
    }
    l = l.min(i32::try_from(p.len()).unwrap_or(i32::MAX));
    for (j, q) in sq.iter().enumerate().take(usize::try_from(l).unwrap_or(0)) {
        set(p, i32::try_from(j).unwrap_or(0), *q)?;
    }
    Ok(l)
}

/// Upstream `GetConvergencePointsDouble`.
pub(super) fn get_convergence_points_double(
    p: &mut [Pt],
    length: f64,
    vbl: i32,
) -> Result<i32, EngineError> {
    let temp: Vec<Pt> = (0..vbl).map(|j| get(p, j)).collect::<Result<_, _>>()?;
    let mut counter = vbl;
    set_style(p, vbl - 1, 5)?;
    for j in 0..vbl - 1 {
        let (pt0, pt1) = (get(&temp, j)?, get(&temp, j + 1)?);
        let d = calc_distance_double(pt0, pt1);
        let mut jags = (d / length) as i32;
        if d - f64::from(jags) * length < 5.0 {
            jags -= 1;
        }
        for k in 0..jags {
            let mut t =
                extend_along_line_double_style(pt0, pt1, f64::from(k) * length + length / 2.0, 0);
            set(p, counter, t)?;
            t = extend_along_line_double(t, pt1, length / 2.0);
            t = extend_directed_line_style(pt0, t, t, 2, length / 2.0, 5);
            set(p, counter + 1, t)?;
            t = extend_along_line_double_style(pt0, pt1, f64::from(k + 1) * length, 0);
            set(p, counter + 2, t)?;
            t = extend_along_line_double(t, pt1, length / 2.0);
            t = extend_directed_line_style(pt0, t, t, 3, length / 2.0, 5);
            set(p, counter + 3, t)?;
            counter += 4;
        }
    }
    Ok(counter)
}

/// Upstream `GetITDPointsDouble`: dashes alternating between two styles.
pub(super) fn get_itd_points_double(
    p: &mut [Pt],
    length: f64,
    vbl: i32,
) -> Result<i32, EngineError> {
    let temp: Vec<Pt> = (0..vbl).map(|j| get(p, j)).collect::<Result<_, _>>()?;
    let mut counter = 0;
    let mut line_style = 19;
    for j in 0..vbl - 1 {
        let (mut pt0, mut pt1) = (get(&temp, j)?, get(&temp, j + 1)?);
        let d = calc_distance_double(pt0, pt1);
        let mut jags = (d / length) as i32;
        if d - f64::from(jags) * length / 3.0 * 2.0 < length / 3.0 {
            jags -= 1;
        }
        if jags == 0 {
            pt0.style = 19;
            set(p, counter, pt0)?;
            pt1.style = 5;
            set(p, counter + 1, pt1)?;
            counter += 2;
        }
        for k in 0..jags {
            let t = extend_along_line_double_style(
                pt0,
                pt1,
                f64::from(k) * length + length / 3.0,
                line_style,
            );
            set(p, counter, t)?;
            let end = if k < jags - 1 {
                extend_along_line_double_style(t, pt1, length * 2.0 / 3.0, 5)
            } else {
                Pt {
                    style: 5,
                    ..get(&temp, j + 1)?
                }
            };
            set(p, counter + 1, end)?;
            counter += 2;
            line_style = if line_style == 19 { 25 } else { 19 };
        }
    }
    Ok(counter)
}

/// TROUGH and CABLE: a squall wave of the given amplitude and length.
pub(super) fn trough(w: &mut Work<'_>, amplitude: f64, length: f64) -> Result<(), EngineError> {
    let (amplitude, length) = (w.scaled(amplitude), w.scaled(length));
    w.vbl = get_squall_double(&mut w.p, amplitude, 6, length, w.save)?;
    w.ac = w.vbl;
    Ok(())
}

pub(super) fn severe_squall(w: &mut Work<'_>) -> Result<(), EngineError> {
    let length = w.scaled(30.0);
    w.vbl = get_severe_squall(&mut w.p, length, w.save)?;
    w.ac = w.vbl;
    Ok(())
}

pub(super) fn itd(w: &mut Work<'_>) -> Result<(), EngineError> {
    let length = w.scaled(15.0);
    w.ac = get_itd_points_double(&mut w.p, length, w.save)?;
    Ok(())
}

/// The ticks alternate sides every half `length`. MIL-STD-2525E change 1
/// (TABLE M-II) spaces them about three times as far apart as upstream,
/// with ticks as long; the buffer sized for upstream's spacing holds them.
pub(super) fn convergence(w: &mut Work<'_>) -> Result<(), EngineError> {
    let version = crate::engine::modifier::center_label::symbol_version(&w.tg.symbol_id);
    let length = w.scaled(if version == Some(15) { 30.0 } else { 10.0 });
    w.ac = get_convergence_points_double(&mut w.p, length, w.save)?;
    Ok(())
}
