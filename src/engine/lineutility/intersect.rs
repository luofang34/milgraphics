//! Port of `CalcTrueIntersectDouble` and `CalcDistance2` from lineutility.java.

use crate::engine::base::Pt;

/// Upstream `CalcTrueIntersectDouble`: where the offset line `1` meets the
/// offset line `2` around the joint `p2`, as `(x, y)`.
///
/// `l1` and `l2` are the `(m, b)` slope/intercept pairs; the vertical flags
/// are 0 for a vertical line; `d_width` is the offset (its sign is ignored);
/// `l_orient` selects which side of the segments the offset line lies on.
pub(crate) fn calc_true_intersect_double(
    l1: (f64, f64),
    l2: (f64, f64),
    p2: Pt,
    vertical: (i32, i32),
    d_width: f64,
    l_orient: i32,
) -> (f64, f64) {
    let (mut m1, mut b1) = l1;
    let (m2, b2) = l2;
    let (v1, v2) = vertical;
    let w = d_width.abs();
    // Upstream's Double.MIN_VALUE is the smallest subnormal, not
    // f64::MIN_POSITIVE.
    let tiny = f64::from_bits(1);
    if m1 != m2 && (m1 - m2).abs() <= tiny {
        m1 = m2;
    }
    if b1 != b2 && (b1 - b2).abs() <= tiny {
        b1 = b2;
    }
    if b1 == b2 && m1 + b1 == m2 + b2 {
        m1 = m2;
    }
    let at_p2 = (p2.x, p2.y);
    if v1 == 0 && v2 == 0 {
        return match l_orient {
            0 => (p2.x - w, p2.y),
            3 => (p2.x + w, p2.y),
            _ => at_p2,
        };
    }
    if v1 == 0 && v2 != 0 {
        return match l_orient {
            0 | 1 => (p2.x - w, m2 * (p2.x - w) + b2),
            2 | 3 => (p2.x + w, m2 * (p2.x + w) + b2),
            _ => at_p2,
        };
    }
    if v2 == 0 && v1 != 0 {
        return match l_orient {
            0 | 2 => (p2.x - w, m1 * (p2.x - w) + b1),
            1 | 3 => (p2.x + w, m1 * (p2.x + w) + b1),
            _ => at_p2,
        };
    }
    if m1 == m2 && m1 != 0.0 {
        if b1 != b2 {
            return at_p2;
        }
        let m = -1.0 / m1;
        let b = p2.y - m * p2.x;
        let x = (b2 - b) / (m - m2);
        return (x, m1 * x + b1);
    }
    if m1 == m2 && m1 == 0.0 {
        return match l_orient {
            0 | 1 => (p2.x, p2.y - w),
            2 | 3 => (p2.x, p2.y + w),
            _ => at_p2,
        };
    }
    if m1 == m2 && b1 == b2 && v1 != 0 && v2 != 0 {
        return parallel_offset(m1, p2, d_width, l_orient);
    }
    let x = (b2 - b1) / (m1 - m2);
    (x, m1 * x + b1)
}

/// The same-slope, same-intercept branch of [`calc_true_intersect_double`]:
/// the joint offset perpendicular to the line by `d_width`.
fn parallel_offset(m1: f64, p2: Pt, d_width: f64, l_orient: i32) -> (f64, f64) {
    let root = (1.0 + m1 * m1).sqrt();
    let (mut x, mut y) = (0.0, 0.0);
    match l_orient {
        0 => {
            if m1 < 0.0 {
                let dy = m1 * d_width / root;
                x = p2.x + dy / m1;
                y = p2.y + dy;
            }
            if m1 > 0.0 {
                let dy = -m1 * d_width / root;
                x = p2.x + -dy / m1;
                y = p2.y + dy;
            }
        }
        3 => {
            if m1 <= 0.0 {
                let dy = -m1 * d_width / root;
                x = p2.x + dy / m1;
                y = p2.y + dy;
            } else {
                let dy = m1 * d_width / root;
                x = p2.x + -dy / m1;
                y = p2.y + dy;
            }
        }
        _ => {
            x = p2.x;
            y = p2.y;
        }
    }
    (x, y)
}

/// Upstream `CalcDistance2`: distance between integer points, with the same
/// zero/infinite fallback as `CalcDistanceDouble`.
pub(crate) fn calc_distance2(x1: i64, y1: i64, x2: i64, y2: i64) -> f64 {
    let dx = x1.wrapping_sub(x2);
    let dy = y1.wrapping_sub(y2);
    let mut r = (dx.wrapping_mul(dx).wrapping_add(dy.wrapping_mul(dy))) as f64;
    r = r.sqrt();
    let xdist = dx.unsigned_abs() as f64;
    let ydist = dy.unsigned_abs() as f64;
    let max = if ydist > xdist { ydist } else { xdist };
    if (r == 0.0 || r.is_infinite()) && max > 0.0 {
        r = max;
    }
    r
}
