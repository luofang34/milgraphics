//! GAP, ASLTXING, MNFLDDIS, LINTGT, LINTGTS, FPF, BLOCK, MNFLDBLK and
//! PAA_RECTANGULAR from DISMSupport.java.

use super::support::{MAX_LENGTH, MIN_LENGTH, clamp_size, draw_endpiece_deltas_double, put};
use crate::engine::base::{At, EngineError, Pt};
use crate::engine::lineutility::basics::{calc_distance_double, mid_point_double};
use crate::engine::lineutility::bounds::mbr_distance;
use crate::engine::lineutility::extend::{extend_line_double, extend_true_line_perp_double};
use crate::engine::settings::Settings;
use crate::engine::tactical_lines as lt;
use std::f64::consts::PI;

fn styled(p: Pt, style: i32) -> Pt {
    let mut q = p;
    q.style = style;
    q
}

fn push_segment(points: &mut Vec<Pt>, counter: &mut usize, a: Pt, b: Pt) {
    put(points, *counter, styled(a, 0));
    put(points, *counter + 1, styled(b, 5));
    *counter += 2;
}

/// Upstream `GetDISMGapDouble`: GAP and ASLTXING, 12 points (the return
/// value is always 12).
pub(crate) fn get_dism_gap_double(
    points: &mut Vec<Pt>,
    settings: &Settings,
) -> Result<i32, EngineError> {
    let mut mbr = mbr_distance(points, 4)?;
    let sp = [points.at(0)?, points.at(1)?, points.at(2)?, points.at(3)?];
    let dpi = settings.dpi_scale_factor();
    if mbr / 10.0 > MAX_LENGTH * dpi {
        mbr = 10.0 * MAX_LENGTH * dpi;
    }
    if mbr / 10.0 < MIN_LENGTH * dpi {
        mbr = 10.0 * MIN_LENGTH * dpi;
    }
    put(points, 0, styled(sp[0], 0));
    put(points, 1, styled(sp[1], 5));
    put(points, 2, styled(sp[2], 0));
    put(points, 3, styled(sp[3], 5));
    let mut dist = mbr / 10.0;
    if dist > 20.0 * dpi {
        dist = 20.0 * dpi;
    }
    let dist2 = dist;
    // Each corner gets a tick from the corner to the middle of the two
    // points that sit `dist` along the edges leaving it.
    let ticks = [
        (0, [(1, 0, dist), (2, 0, dist2)]),
        (1, [(0, 1, dist), (3, 1, dist2)]),
        (2, [(0, 2, dist2), (3, 2, dist)]),
        (3, [(1, 3, dist2), (2, 3, dist)]),
    ];
    for (k, (corner, [a, b])) in ticks.iter().enumerate() {
        let p0 = extend_line_double(sp.at(a.0)?, sp.at(a.1)?, a.2);
        let p1 = extend_line_double(sp.at(b.0)?, sp.at(b.1)?, b.2);
        put(points, 4 + 2 * k, styled(sp.at(*corner)?, 0));
        put(points, 5 + 2 * k, mid_point_double(p0, p1, 5));
    }
    Ok(12)
}

/// Upstream `GetDISMMinefieldDisruptDouble`: MNFLDDIS, 22 points.
pub(crate) fn get_dism_minefield_disrupt_double(
    points: &mut Vec<Pt>,
    settings: &Settings,
) -> Result<i32, EngineError> {
    let sp = [points.at(0)?, points.at(1)?, points.at(2)?];
    let mut counter = 0;
    push_segment(points, &mut counter, sp[0], sp[1]);
    let ctr = Pt::new((sp[0].x + sp[1].x) / 2.0, (sp[0].y + sp[1].y) / 2.0);
    let (ex, ey) = (sp[2].x - sp[0].x, sp[2].y - sp[0].y);
    let arrow = [
        sp[2],
        Pt::new(ctr.x + ex * 4.0 / 5.0, ctr.y + ey * 4.0 / 5.0),
        Pt::new(sp[1].x + ex * 3.0 / 5.0, sp[1].y + ey * 3.0 / 5.0),
    ];
    push_segment(points, &mut counter, sp[1], arrow[2]);
    push_segment(points, &mut counter, ctr, arrow[1]);
    let dpi = settings.dpi_scale_factor();
    let mut d = calc_distance_double(sp[2], sp[0]);
    if d > 5.0 * MAX_LENGTH * dpi {
        d = 5.0 * MAX_LENGTH * dpi;
    }
    if d < 5.0 * MIN_LENGTH * dpi {
        d = 5.0 * MIN_LENGTH * dpi;
    }
    let tail_end = Pt::new(ctr.x - ex / 5.0, ctr.y - ey / 5.0);
    push_segment(
        points,
        &mut counter,
        ctr,
        extend_line_double(tail_end, ctr, -d / 5.0),
    );
    push_segment(points, &mut counter, sp[0], arrow[0]);

    let height = ((sp[1].x - sp[0].x) * (sp[1].x - sp[0].x)
        + (sp[1].y - sp[0].y) * (sp[1].y - sp[0].y))
        .sqrt();
    let length = ((sp[2].x - sp[1].x) * (sp[2].x - sp[1].x)
        + (sp[2].y - sp[1].y) * (sp[2].y - sp[1].y))
        .sqrt();
    let diag = clamp_size((height + length) / 15.0, dpi);
    let angle = (sp[0].y - sp[2].y).atan2(sp[0].x - sp[2].x);
    let d1 = diag * (angle - PI / 6.0).cos();
    let d2 = diag * (angle - PI / 6.0).sin();
    let d3 = diag * (angle + PI / 6.0).cos();
    let d4 = diag * (angle + PI / 6.0).sin();
    for tip in arrow {
        let b = draw_endpiece_deltas_double(tip, d1, d2, d3, d4);
        for (p, style) in [(b[1], 9), (b[0], 9), (b[3], 9), (b[3], 10)] {
            put(points, counter, styled(p, style));
            counter += 1;
        }
    }
    Ok(counter as i32)
}

/// Upstream `GetDISMLinearTargetDouble`: LINTGT, LINTGTS and FPF. The
/// `vbl_counter - 4` leading points are the polyline; four end-tick points
/// follow it.
pub(crate) fn get_dism_linear_target_double(
    points: &mut Vec<Pt>,
    line_type: i32,
    vbl_counter: i32,
    settings: &Settings,
) -> Result<i32, EngineError> {
    let mut mbr = mbr_distance(points, vbl_counter - 4)?;
    let dpi = settings.dpi_scale_factor();
    if mbr / 20.0 > MAX_LENGTH * dpi {
        mbr = 20.0 * MAX_LENGTH * dpi;
    }
    if mbr / 20.0 < MIN_LENGTH * dpi {
        mbr = 20.0 * MIN_LENGTH * dpi;
    }
    if mbr < 150.0 * dpi {
        mbr = 150.0 * dpi;
    }
    if mbr > 250.0 * dpi {
        mbr = 250.0 * dpi;
    }
    let count = usize::try_from(vbl_counter - 4)
        .map_err(|_| EngineError::Degenerate("linear target needs its end-tick slots"))?;
    for j in 0..count {
        points.at_mut(j)?.style = 0;
    }
    let last = count
        .checked_sub(1)
        .ok_or(EngineError::Degenerate("linear target has no points"))?;
    points.at_mut(last)?.style = 5;
    let mut counter = count;
    let (first, second) = (points.at(0)?, points.at(1)?);
    let step = mbr / 20.0;
    put(
        points,
        counter,
        extend_true_line_perp_double(first, second, first, step, 0)?,
    );
    put(
        points,
        counter + 1,
        extend_true_line_perp_double(first, second, first, -step, 5)?,
    );
    let end = idx_from_end(points, vbl_counter, 5)?;
    let before = idx_from_end(points, vbl_counter, 6)?;
    put(
        points,
        counter + 2,
        extend_true_line_perp_double(end, before, end, step, 0)?,
    );
    put(
        points,
        counter + 3,
        extend_true_line_perp_double(end, before, end, -step, 5)?,
    );
    counter += 4;
    if line_type == lt::FPF {
        points.at_mut(0)?.style = 6;
    }
    Ok(counter as i32)
}

fn idx_from_end(points: &[Pt], vbl_counter: i32, back: i32) -> Result<Pt, EngineError> {
    let i = crate::engine::base::idx(vbl_counter - back, points.len())?;
    points.at(i)
}

/// Upstream `GetDISMBlockDouble2`: BLOCK and MNFLDBLK, 4 points.
pub(crate) fn get_dism_block_double2(
    points: &mut Vec<Pt>,
    line_type: i32,
) -> Result<(), EngineError> {
    let relative = points.at(2)?;
    let mid = mid_point_double(points.at(0)?, points.at(1)?, 0);
    points.at_mut(0)?.style = 0;
    points.at_mut(1)?.style = 5;
    put(points, 2, mid);
    put(points, 3, relative);
    if line_type == lt::BLOCK {
        points.at_mut(2)?.style = 14;
    }
    if line_type == lt::FPF {
        points.at_mut(2)?.style = 6;
    }
    Ok(())
}
