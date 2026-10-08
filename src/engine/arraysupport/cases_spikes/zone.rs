//! Port of `GetZONEPointsDouble2`: the spiked outline of zones, obstacle
//! areas, strongpoints, forts and encirclements.

use crate::engine::base::{EngineError, Pt};
use crate::engine::lineutility::basics::calc_distance_double;
use crate::engine::lineutility::extend::{
    extend_directed_line, extend_line_double, extend_line2_double,
};
use crate::engine::lineutility::slope::reverse_direction;
use crate::engine::tactical_lines as lt;
use crate::engine::tg::Tg;

use super::fortl::store_all;
use crate::engine::arraysupport::inside_outside::get_inside_outside_double2;
use crate::engine::arraysupport::work::{get, scaled_size, set_style};

/// Index from the end of a scratch array.
fn back(v: &[Pt], n: usize) -> Result<Pt, EngineError> {
    let i = v
        .len()
        .checked_sub(n)
        .ok_or(EngineError::Degenerate("spike array too short"))?;
    get(
        v,
        i32::try_from(i).map_err(|_| EngineError::Degenerate("index"))?,
    )
}

/// Upstream `GetZONEPointsDouble2`: returns the point count.
pub(super) fn zone_points2(tg: &Tg, p: &mut [Pt], save: i32) -> Result<i32, EngineError> {
    let line_type = tg.line_type;
    let increment = scaled_size(tg, 20.0);
    let mut pt0 = get(p, 0)?;
    let mut spikes: Vec<Pt> = Vec::new();
    for j in 0..save - 1 {
        let (pt1, pt2) = (get(p, j)?, get(p, j + 1)?);
        let mut direction = get_inside_outside_double2(pt1, pt2, p, save, j, line_type)?;
        let length = calc_distance_double(pt1, pt2);
        if length < increment {
            spikes.push(pt1);
            spikes.push(pt2);
            continue;
        }
        if matches!(line_type, lt::OBSAREA | lt::OBSFAREA) {
            direction = reverse_direction(direction);
        }
        let n = (length / increment) as i32;
        let remainder = length - f64::from(n) * increment;
        for k in 0..n {
            let kf = f64::from(k);
            if k > 0 {
                spikes.push(extend_line2_double(
                    pt2,
                    pt1,
                    -kf * increment - remainder / 2.0,
                    0,
                ));
                spikes.push(extend_line2_double(
                    pt2,
                    pt1,
                    -kf * increment - increment / 2.0 - remainder / 2.0,
                    0,
                ));
            } else {
                spikes.push(extend_line2_double(pt2, pt1, -kf * increment, 0));
                spikes.push(extend_line2_double(
                    pt2,
                    pt1,
                    -kf * increment - increment / 2.0,
                    0,
                ));
            }
            match line_type {
                lt::OBSAREA | lt::OBSFAREA | lt::ZONE | lt::ENCIRCLE => {
                    pt0 = extend_line_double(pt1, back(&spikes, 1)?, increment / 4.0);
                }
                lt::STRONG | lt::FORT_REVD | lt::FORT => pt0 = back(&spikes, 1)?,
                _ => {}
            }
            spikes.push(extend_directed_line(
                pt1,
                pt2,
                pt0,
                direction,
                increment / 2.0,
            ));
            add_closing(&mut spikes, line_type, (pt1, pt2), direction, increment)?;
        }
        spikes.push(pt2);
    }
    finish(p, spikes, line_type)
}

/// The points after the spike tip: they differ by line type.
fn add_closing(
    spikes: &mut Vec<Pt>,
    line_type: i32,
    seg: (Pt, Pt),
    direction: i32,
    increment: f64,
) -> Result<(), EngineError> {
    let (pt1, pt2) = seg;
    let base = back(spikes, 2)?;
    match line_type {
        lt::OBSAREA | lt::OBSFAREA | lt::ZONE | lt::ENCIRCLE => {
            spikes.push(extend_line2_double(pt1, base, increment / 2.0, 0));
        }
        lt::STRONG => spikes.push(base),
        lt::FORT_REVD | lt::FORT => {
            let pt3 = extend_line2_double(pt1, base, increment / 2.0, 0);
            spikes.push(extend_directed_line(
                pt1,
                pt2,
                pt3,
                direction,
                increment / 2.0,
            ));
            spikes.push(pt3);
        }
        _ => spikes.push(Pt::default()),
    }
    if line_type == lt::ENCIRCLE {
        let c = back(spikes, 3)?;
        spikes.push(c);
    }
    Ok(())
}

/// Styles the outline's end and copies it into the point array.
fn finish(p: &mut [Pt], mut spikes: Vec<Pt>, line_type: i32) -> Result<i32, EngineError> {
    if line_type == lt::OBSAREA {
        for s in &mut spikes {
            s.style = 11;
        }
        if let Some(last) = spikes.last_mut() {
            last.style = 12;
        }
    } else if let Some(last) = spikes.last_mut() {
        last.style = 5;
    }
    let count = store_all(p, &spikes)?;
    if count > 0 && line_type != lt::OBSAREA {
        set_style(p, count - 1, 5)?;
    }
    Ok(count)
}
