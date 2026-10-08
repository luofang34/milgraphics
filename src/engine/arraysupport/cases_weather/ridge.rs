//! RIDGE: spikes along the line.

use super::super::count_sizes::fortl_count;
use super::super::work::{Work, get, scaled_size, set};
use crate::engine::base::{EngineError, Pt};
use crate::engine::lineutility::basics::calc_distance_double;
use crate::engine::lineutility::extend::{
    extend_directed_line, extend_line_double, extend_line2_double,
};
use crate::engine::lineutility::slope::calc_true_slope_double;
use crate::engine::tg::Tg;

/// Upstream `GetRidgePointsDouble`: returns the number of points.
fn get_ridge_points_double(tg: &Tg, p: &mut [Pt], save: i32) -> Result<i32, EngineError> {
    let increment = scaled_size(tg, 20.0);
    let spike = scaled_size(tg, 20.0);
    let l_count = fortl_count(tg, p, save)?;
    let mut sp: Vec<Pt> = Vec::new();
    for j in 0..save - 1 {
        let (a, b) = (get(p, j)?, get(p, j + 1)?);
        let (vertical, _) = calc_true_slope_double(a, b);
        let limit = (calc_distance_double(a, b) / increment) as i32;
        if limit < 1 {
            sp.push(a);
            sp.push(b);
            continue;
        }
        for k in 0..limit {
            let base = extend_line2_double(b, a, -f64::from(k) * increment, 0);
            sp.push(base);
            let d = calc_distance_double(a, base);
            let pt0 = extend_line_double(b, a, -d - spike / 2.0);
            let tip = if vertical != 0 {
                let dir = if a.x < b.x { 2 } else { 3 };
                extend_directed_line(a, b, pt0, dir, spike)
            } else {
                let dir = if b.y < a.y { 0 } else { 1 };
                extend_directed_line(a, b, pt0, dir, spike)
            };
            sp.push(tip);
            sp.push(extend_line2_double(b, a, -d - spike, 0));
        }
        sp.push(b);
    }
    for (j, q) in sp.iter().enumerate() {
        set(p, i32::try_from(j).unwrap_or(0), *q)?;
    }
    let n = i32::try_from(sp.len()).unwrap_or(i32::MAX);
    let last = get(&sp, n - 1)?;
    for j in n..l_count {
        set(p, j, last)?;
    }
    Ok(n)
}

pub(super) fn ridge(w: &mut Work<'_>) -> Result<(), EngineError> {
    w.vbl = get_ridge_points_double(w.tg, &mut w.p, w.save)?;
    w.ac = w.vbl;
    Ok(())
}
