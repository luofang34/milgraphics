//! Port of `GetFORTLPointsDouble` and `GetATWallPointsDouble2`: the glyph
//! lines of FORTL and of the anti-tank wall / line types.

use crate::engine::base::{EngineError, Pt, java_round};
use crate::engine::lineutility::basics::calc_distance_double;
use crate::engine::lineutility::extend::{
    extend_directed_line, extend_line_double, extend_line2_double,
};
use crate::engine::tg::Tg;

use crate::engine::arraysupport::work::{get, scaled_size, set};

/// The element `back` slots before the end of a growing scratch array.
fn from_end(v: &[Pt], back: usize) -> Result<Pt, EngineError> {
    let i = v
        .len()
        .checked_sub(back)
        .ok_or(EngineError::Degenerate("spike array too short"))?;
    get(
        v,
        i32::try_from(i).map_err(|_| EngineError::Degenerate("index"))?,
    )
}

/// Copies the scratch points over the first slots of the point array.
pub(super) fn store_all(p: &mut [Pt], spikes: &[Pt]) -> Result<i32, EngineError> {
    for (j, s) in spikes.iter().enumerate() {
        let j = i32::try_from(j).map_err(|_| EngineError::Degenerate("index"))?;
        set(p, j, *s)?;
    }
    i32::try_from(spikes.len()).map_err(|_| EngineError::Degenerate("index"))
}

/// Upstream `GetFORTLPointsDouble`: spikes on alternating segments of
/// `p[0..save]`; returns the point count.
pub(super) fn fortl_points(tg: &Tg, p: &mut [Pt], save: i32) -> Result<i32, EngineError> {
    let increment = scaled_size(tg, 20.0);
    let glyph = increment / 2.0;
    let mut spikes: Vec<Pt> = Vec::new();
    for j in 0..save - 1 {
        let (a, b) = (get(p, j)?, get(p, j + 1)?);
        let length = calc_distance_double(a, b);
        if length / increment < 1.0 {
            spikes.push(a);
            spikes.push(b);
            continue;
        }
        let glyphs = (length / increment) as i32;
        let seg = length / f64::from(glyphs);
        for k in 0..glyphs {
            let kf = f64::from(k);
            spikes.push(extend_line2_double(b, a, -kf * seg, 0));
            let mid = extend_line2_double(b, a, -kf * seg - seg / 2.0, 0);
            spikes.push(mid);
            let pt0 = mid;
            let pt1 = extend_line_double(a, mid, seg / 2.0);
            let direction = if a.x > b.x {
                Some(3)
            } else if a.x < b.x {
                Some(2)
            } else if a.y < b.y {
                Some(1)
            } else if a.y > b.y {
                Some(0)
            } else {
                None
            };
            if let Some(d) = direction {
                spikes.push(extend_directed_line(a, b, pt0, d, glyph));
                spikes.push(extend_directed_line(a, b, pt1, d, glyph));
            }
            let base = from_end(&spikes, 3)?;
            spikes.push(extend_line2_double(a, base, seg / 2.0, 0));
        }
        spikes.push(b);
    }
    store_all(p, &spikes)
}

/// Upstream `GetATWallPointsDouble2`: the wall's spikes along `p[0..save]`.
pub(super) fn at_wall_points2(tg: &Tg, p: &mut [Pt], save: i32) -> Result<i32, EngineError> {
    let mut spikes: Vec<Pt> = vec![get(p, 0)?];
    for j in 0..save - 1 {
        let (a, b) = (get(p, j)?, get(p, j + 1)?);
        let length = calc_distance_double(a, b);
        let spike = scaled_size(tg, 10.0);
        let mut increment = 2.0 * spike;
        let num_spikes = java_round((length - spike) / increment) as i32;
        increment = length / f64::from(num_spikes);
        let limit = num_spikes - 1;
        for k in -1..limit {
            let kf = f64::from(k);
            spikes.push(extend_line2_double(b, a, -kf * increment - spike * 3.0, 0));
            let prev = from_end(&spikes, 1)?;
            let pt0 = extend_line_double(a, prev, spike / 2.0);
            let tip = if a.x > b.x {
                Some(extend_directed_line(a, prev, pt0, 2, spike))
            } else if a.x < b.x {
                Some(extend_directed_line(a, prev, pt0, 3, spike))
            } else {
                None
            };
            spikes.push(match tip {
                Some(t) => t,
                None => {
                    let mut t = pt0;
                    t.x = if a.y < b.y {
                        pt0.x - spike
                    } else {
                        pt0.x + spike
                    };
                    t
                }
            });
            let back = from_end(&spikes, 2)?;
            spikes.push(extend_line2_double(a, back, spike, 0));
        }
        spikes.push(Pt { style: 0, ..b });
    }
    store_all(p, &spikes)
}
