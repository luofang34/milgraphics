//! Ports of `ReverseUSASLCPointsByQuadrant` and `SegmentLCPoints` from
//! mil-sym-java JavaTacticalRenderer/clsUtility.java, in pixel form.

use crate::engine::base::{EngineError, Pt};
use crate::engine::channels::scaled_size::get_scaled_size;
use crate::engine::lineutility::basics::{calc_distance_double, get_quadrant_double};
use crate::engine::lineutility::extend::extend_along_line_double2;
use crate::engine::tactical_lines as lt;
use crate::engine::tg::Tg;
use std::f64::consts::PI;

/// Upstream `ReverseUSASLCPointsByQuadrant`: USAS wants a left-to-right
/// orientation for the enemy line of contact, so the points are reversed for
/// two of the four quadrants (1 and 4 when hostile, 2 and 3 otherwise).
pub(crate) fn reverse_usas_lc_points_by_quadrant(tg: &mut Tg) -> Result<(), EngineError> {
    if tg.pixels.len() < 2 || tg.line_type != lt::LC {
        return Ok(());
    }
    let first = tg.pixels.first().copied().unwrap_or_default();
    let second = tg.pixels.get(1).copied().unwrap_or_default();
    let quadrant = get_quadrant_double(first, second);
    let reverse = if tg.is_hostile() {
        matches!(quadrant, 1 | 4)
    } else {
        matches!(quadrant, 2 | 3)
    };
    if reverse {
        tg.pixels.reverse();
    }
    Ok(())
}

/// Upstream `SegmentLCPoints`: for a line of contact, adds a point on the
/// longer arm of each acute angle where the channel cannot fit, so
/// `get_lc_partitions` can split there.
pub(crate) fn segment_lc_points(tg: &mut Tg) -> Result<(), EngineError> {
    if tg.line_type != lt::LC {
        return Ok(());
    }
    let channel_width = get_scaled_size(40.0, f64::from(tg.line_thickness), tg.pattern_scale);
    let mut i = 0usize;
    while i + 2 < tg.pixels.len() {
        let (Some(a), Some(b), Some(c)) = (
            tg.pixels.get(i).copied(),
            tg.pixels.get(i + 1).copied(),
            tg.pixels.get(i + 2).copied(),
        ) else {
            break;
        };
        let (pt_a, pt_b, pt_c) = (Pt::new(a.x, a.y), Pt::new(b.x, b.y), Pt::new(c.x, c.y));
        let angle1 = (pt_b.y - pt_a.y).atan2(pt_b.x - pt_a.x);
        let angle2 = (pt_b.y - pt_c.y).atan2(pt_b.x - pt_c.x);
        let angle = angle1 - angle2;
        let mut degrees = angle * 180.0 / PI;
        if angle < 0.0 {
            degrees += 360.0;
        }
        if degrees > 270.0 {
            let (insert_at, new_pt, far) =
                if calc_distance_double(pt_b, pt_a) < calc_distance_double(pt_b, pt_c) {
                    let n = extend_along_line_double2(pt_b, pt_c, calc_distance_double(pt_b, pt_a));
                    (i + 2, n, pt_a)
                } else {
                    let n = extend_along_line_double2(pt_b, pt_a, calc_distance_double(pt_b, pt_c));
                    (i + 1, n, pt_c)
                };
            if calc_distance_double(far, new_pt) < channel_width {
                tg.pixels.insert(insert_at, Pt::new(new_pt.x, new_pt.y));
                i += 1;
            }
        }
        i += 1;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
