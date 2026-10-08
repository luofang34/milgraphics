//! `ConnectArrayTrueDouble` of Channels.java: walks a polyline and, at each
//! vertex, picks which side of each segment the two channel edges lie on so
//! the edges stay continuous through turns.
//!
//! `n_last` records the previous segment: 0 left to right with edge 1 above,
//! 1 left to right with edge 1 below, 2 right to left with edge 1 above,
//! 3 right to left with edge 1 below, 4 and 5 vertical upward with edge 1
//! to the left and right, 6 and 7 vertical downward with edge 1 to the left
//! and right.

use super::ChannelPoints;
use super::point_index::at_i;
use super::true_points::{connect_true_double2, get_true_end_point_double};
use crate::engine::base::{EngineError, Pt};

fn sgn(x: f64) -> Option<i8> {
    if x > 0.0 {
        Some(1)
    } else if x < 0.0 {
        Some(-1)
    } else if x == 0.0 {
        Some(0)
    } else {
        None
    }
}

/// The `nLast` that begins the walk, from the first segment's direction.
fn initial_last(d1x: f64, d1y: f64) -> i32 {
    let mut last = 0;
    if d1x == 0.0 {
        if d1y > 0.0 {
            last = 6;
        }
        if d1y < 0.0 {
            last = 4;
        }
    }
    if d1y == 0.0 {
        if d1x > 0.0 {
            last = 0;
        }
        if d1x < 0.0 {
            last = 2;
        }
    }
    if d1x < 0.0 && d1y > 0.0 {
        last = 3;
    }
    if d1x > 0.0 && d1y > 0.0 {
        last = 0;
    }
    if d1x < 0.0 && d1y < 0.0 {
        last = 3;
    }
    if d1x > 0.0 && d1y < 0.0 {
        last = 0;
    }
    last
}

/// Whether the previous segment left edge 1 on the "above" family of codes.
fn previous_above(last: i32) -> bool {
    matches!(last, 0 | 3 | 4 | 7)
}

/// Picks `if dy > 0 { when_pos } else if dy < 0 { when_neg }`, keeping
/// `current` when `dy` is zero.
fn by_sign(dy: f64, when_pos: i32, when_neg: i32, current: i32) -> i32 {
    if dy > 0.0 {
        when_pos
    } else if dy < 0.0 {
        when_neg
    } else {
        current
    }
}

/// The orientation (0 above/above, 1 above/below, 2 below/above, 3 below/below)
/// at a vertex from the previous state and the two segment deltas; keeps
/// `current` where upstream assigns nothing.
fn vertex_orientation(last: i32, d: [f64; 4], current: i32) -> i32 {
    let [d1x, d2x, d1y, d2y] = d;
    let above = previous_above(last);
    match (sgn(d1x), sgn(d2x)) {
        (Some(1), Some(1)) => {
            if above {
                0
            } else {
                3
            }
        }
        (Some(1), Some(0)) => {
            if above {
                by_sign(d2y, 1, 0, current)
            } else {
                by_sign(d2y, 2, 3, current)
            }
        }
        (Some(-1), Some(0)) => {
            if above {
                by_sign(d2y, 3, 2, current)
            } else {
                by_sign(d2y, 0, 1, current)
            }
        }
        (Some(0), Some(1)) => {
            if above {
                by_sign(d1y, 2, 0, current)
            } else {
                by_sign(d1y, 1, 3, current)
            }
        }
        (Some(0), Some(-1)) => {
            if above {
                by_sign(d1y, 3, 1, current)
            } else {
                by_sign(d1y, 0, 2, current)
            }
        }
        (Some(-1), Some(-1)) => {
            if above {
                3
            } else {
                0
            }
        }
        (Some(1), Some(-1)) => {
            if above {
                1
            } else {
                2
            }
        }
        (Some(-1), Some(1)) => {
            if above {
                2
            } else {
                1
            }
        }
        (Some(0), Some(0)) => match last {
            4 if d2y < 0.0 => 0,
            6 if d2y > 0.0 => 0,
            5 if d2y < 0.0 => 3,
            7 if d2y > 0.0 => 3,
            _ => current,
        },
        _ => current,
    }
}

/// The `nLast` after drawing a segment with orientation `orient` and
/// deltas `d2x`, `d2y`; keeps `last` where upstream assigns nothing.
fn next_last(orient: i32, d2x: f64, d2y: f64, last: i32) -> i32 {
    let mut n = last;
    if d2x == 0.0 {
        let (down, up) = match orient {
            0 | 2 => (6, 4),
            1 | 3 => (7, 5),
            _ => return n,
        };
        if d2y > 0.0 {
            n = down;
        }
        if d2y < 0.0 {
            n = up;
        }
    }
    if d2x > 0.0 {
        n = match orient {
            0 | 2 => 0,
            1 | 3 => 1,
            _ => n,
        };
    }
    if d2x < 0.0 {
        n = match orient {
            0 | 2 => 2,
            1 | 3 => 3,
            _ => n,
        };
    }
    n
}

/// Upstream `ConnectArrayTrueDouble`: the channel edge points at each of the
/// `n_counter + 1` vertices of `pts`, `n_width` either side of the line.
pub(crate) fn connect_array_true_double(
    n_width: i32,
    n_counter: i32,
    pts: &[Pt],
) -> Result<Vec<ChannelPoints>, EngineError> {
    let first = at_i(pts, 0)?;
    let second = at_i(pts, 1)?;
    let mut last = initial_last(second.x - first.x, second.y - first.y);
    let mut out =
        vec![ChannelPoints::default(); usize::try_from(n_counter.max(0)).unwrap_or(0) + 1];
    *out.first_mut()
        .ok_or(EngineError::Degenerate("no channel points"))? =
        get_true_end_point_double(n_width, first, second, last);

    let mut orient = 0;
    for k in 1..n_counter {
        let p1 = at_i(pts, k - 1)?;
        let p2 = at_i(pts, k)?;
        let p3 = at_i(pts, k + 1)?;
        let (d1x, d2x) = (p2.x - p1.x, p3.x - p2.x);
        let (d1y, d2y) = (p2.y - p1.y, p3.y - p2.y);
        orient = vertex_orientation(last, [d1x, d2x, d1y, d2y], orient);
        let slot = out
            .get_mut(usize::try_from(k).unwrap_or(usize::MAX))
            .ok_or(EngineError::Degenerate("channel point slot"))?;
        *slot = connect_true_double2(n_width, p1, p2, p3, orient);
        last = next_last(orient, d2x, d2y, last);
    }

    let end = at_i(pts, n_counter)?;
    let before_end = at_i(pts, n_counter - 1)?;
    let slot = out
        .get_mut(usize::try_from(n_counter).unwrap_or(usize::MAX))
        .ok_or(EngineError::Degenerate("channel point slot"))?;
    *slot = get_true_end_point_double(n_width, end, before_end, last);
    Ok(out)
}
