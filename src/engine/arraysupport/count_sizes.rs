//! Port of the point-count helpers of countsupport.java. The counts size
//! upstream's fixed arrays; they matter here because several builders index
//! from the array's end and read its length.

use crate::engine::base::{EngineError, Pt};
use crate::engine::lineutility::basics::calc_distance_double;
use crate::engine::tactical_lines as lt;
use crate::engine::tg::Tg;
use crate::engine::visible::PixelBox;

use super::work::{MAX_LENGTH, MIN_LENGTH, get, scaled_size};

/// Java's `int += double`: the sum is taken in `double` and truncated.
fn add(count: i32, v: f64) -> i32 {
    (f64::from(count) + v) as i32
}

/// `GetReefCount`.
pub(crate) fn reef_count(pts: &[Pt], length: f64, vbl: i32) -> Result<i32, EngineError> {
    let mut count = 0;
    for j in 0..vbl - 1 {
        let d = calc_distance_double(get(pts, j)?, get(pts, j + 1)?);
        count = add(count, f64::from(5 * (d as i32)) / length);
    }
    Ok(count + 2 * vbl)
}

/// `GetRestrictedAreaCount`.
pub(crate) fn restricted_area_count(pts: &[Pt], length: f64, vbl: i32) -> Result<i32, EngineError> {
    let mut count = 0;
    for j in 0..vbl - 1 {
        let d = calc_distance_double(get(pts, j)?, get(pts, j + 1)?);
        count = add(count, f64::from(4 * (d as i32)) / length);
    }
    Ok(count + 2 * vbl)
}

/// `GetPipeCount`.
pub(crate) fn pipe_count(pts: &[Pt], length: f64, vbl: i32) -> Result<i32, EngineError> {
    let mut count = 0;
    for j in 0..vbl - 1 {
        let d = calc_distance_double(get(pts, j)?, get(pts, j + 1)?);
        count = add(count, f64::from(3 * (d as i32)) / length);
    }
    Ok(count + 2 * vbl)
}

/// `GetXPointsCount`.
pub(crate) fn x_points_count(
    pts: &[Pt],
    segment_length: f64,
    vbl: i32,
) -> Result<i32, EngineError> {
    let mut count = 0;
    for j in 0..vbl - 1 {
        let d = calc_distance_double(get(pts, j)?, get(pts, j + 1)?);
        count += 4 * (((d - segment_length / 2.0) / segment_length) as i32);
    }
    Ok(count)
}

/// `GetLVOCount`.
pub(crate) fn lvo_count(pts: &[Pt], segment_length: f64, vbl: i32) -> Result<i32, EngineError> {
    let mut count = 0;
    for j in 0..vbl - 1 {
        let d = calc_distance_double(get(pts, j)?, get(pts, j + 1)?);
        count += (((d - segment_length) / segment_length) as i32 + 1) * 37;
    }
    Ok(count)
}

/// `GetIcingCount`.
pub(crate) fn icing_count(pts: &[Pt], length: f64, vbl: i32) -> Result<i32, EngineError> {
    let mut total = 2 * vbl;
    for j in 0..vbl - 1 {
        let d = calc_distance_double(get(pts, j)?, get(pts, j + 1)?);
        total = add(total, (d / length) * 4.0);
    }
    Ok(total)
}

/// `GetITDQty`.
pub(crate) fn itd_qty(pts: &[Pt], length: f64, vbl: i32) -> Result<i32, EngineError> {
    let mut total = 0;
    for j in 0..vbl - 1 {
        let d = calc_distance_double(get(pts, j)?, get(pts, j + 1)?);
        total += (2 * ((d / length) as i32)).max(2);
    }
    Ok(total)
}

/// `GetConvergenceQty`.
pub(crate) fn convergence_qty(pts: &[Pt], length: f64, vbl: i32) -> Result<i32, EngineError> {
    let mut total = vbl;
    for j in 0..vbl - 1 {
        let d = calc_distance_double(get(pts, j)?, get(pts, j + 1)?);
        total += 4 * ((d / length) as i32);
    }
    Ok(total)
}

/// `GetDitchCountDouble`.
pub(crate) fn ditch_count(pts: &[Pt], vbl: i32, line_type: i32) -> Result<i32, EngineError> {
    let mut total = vbl;
    for j in 0..vbl - 1 {
        let how_far = calc_distance_double(get(pts, j)?, get(pts, j + 1)?);
        let per_segment = ((how_far - 1.0) / 12.0) as i32;
        if how_far > 24.0 {
            total += if line_type == lt::ATDITCHM {
                5 * per_segment + 1
            } else {
                4 * per_segment
            };
        } else {
            total += 2;
        }
    }
    Ok(total)
}

/// `GetSquallQty`.
pub(crate) fn squall_qty(
    pts: &[Pt],
    quantity: i32,
    length: f64,
    num_points: i32,
) -> Result<i32, EngineError> {
    let mut counter = 0;
    for j in 0..num_points - 1 {
        let dist = calc_distance_double(get(pts, j)?, get(pts, j + 1)?);
        let curves = (dist / length) as i32;
        counter += curves * quantity;
        if curves == 0 {
            counter += 2;
        }
    }
    Ok(counter.max(num_points))
}

/// `GetSquallSegQty`.
pub(crate) fn squall_seg_qty(start: Pt, end: Pt, quantity: i32, length: f64) -> i32 {
    let dist = calc_distance_double(start, end);
    ((dist / length) as i32) * quantity
}

/// `GetFORTLCountDouble`.
pub(crate) fn fortl_count(tg: &Tg, pts: &[Pt], vbl: i32) -> Result<i32, EngineError> {
    let increment = match tg.line_type {
        lt::UCF | lt::CF | lt::CFG | lt::CFY => scaled_size(tg, 60.0),
        _ => scaled_size(tg, 20.0),
    };
    let mut counter = 0_i32;
    for j in 0..vbl - 1 {
        let mut c = calc_distance_double(get(pts, j)?, get(pts, j + 1)?);
        c = match tg.line_type {
            lt::CFG => (c / increment) * 13.0,
            lt::CFY => (c / increment) * 17.0,
            _ => (c / increment) * 10.0,
        };
        if c < 4.0 {
            c = 4.0;
        }
        counter = (i64::from(counter) + c as i64) as i32;
    }
    Ok(counter + 10 + vbl)
}

/// [`fortl_count`] for the zones, which with a `visible` box spike only the
/// part of each segment near it.
pub(crate) fn zone_count(
    tg: &Tg,
    pts: &[Pt],
    vbl: i32,
    visible: Option<&PixelBox>,
) -> Result<i32, EngineError> {
    let Some(bx) = visible else {
        return fortl_count(tg, pts, vbl);
    };
    let increment = scaled_size(tg, 20.0);
    let mut counter = 0_i32;
    for j in 0..vbl - 1 {
        let (a, b) = (get(pts, j)?, get(pts, j + 1)?);
        let near = bx
            .span(a, b)
            .map_or(0.0, |(lo, hi)| hi - lo + 6.0 * increment);
        let c = (calc_distance_double(a, b).min(near) / increment * 10.0).max(4.0);
        counter = (i64::from(counter) + c as i64) as i32;
    }
    Ok(counter + 10 + vbl)
}

/// `GetDISMFixCountDouble` for no clip bounds.
pub(crate) fn dism_fix_count(first: Pt, last: Pt, dpi: f64) -> i32 {
    let length =
        ((last.x - first.x) * (last.x - first.x) + (last.y - first.y) * (last.y - first.y)).sqrt();
    let mut half_amp = length / 15.0;
    if half_amp > MAX_LENGTH * dpi {
        half_amp = MAX_LENGTH * dpi;
    }
    if half_amp < MIN_LENGTH * dpi {
        half_amp = MIN_LENGTH * dpi;
    }
    let half_period = half_amp / 1.5;
    let jaggies = ((length / half_period) as i32 - 3).max(0);
    20 + jaggies * 3
}
