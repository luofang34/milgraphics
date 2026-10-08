//! `GetTrueEndPointDouble` and `ConnectTrueDouble2` of Channels.java: the
//! channel edge points at a polyline's end and at an interior vertex.

use super::ChannelPoints;
use crate::engine::base::Pt;
use crate::engine::lineutility::intersect::calc_true_intersect_double;
use crate::engine::lineutility::slope::{calc_true_intersect_double2, calc_true_lines_double};

/// Upstream `GetTrueEndPointDouble`: the edge points `n_width` either side
/// of `end` for the segment toward `next`, ordered by `last` (0..3 for
/// non-vertical segments, 4..7 for vertical ones).
pub(crate) fn get_true_end_point_double(
    n_width: i32,
    end: Pt,
    next: Pt,
    last: i32,
) -> ChannelPoints {
    let mut answers = ChannelPoints::default();
    let d_width = f64::from(n_width);
    let (vertical, r) = calc_true_lines_double(i64::from(n_width), end, next);
    let (m, upper_b, lower_b) = (r[0], r[3], r[5]);

    if vertical == 0 {
        match last {
            4 | 6 => {
                answers.line1.x = end.x - d_width;
                answers.line1.y = end.y;
                answers.line2.x = end.x + d_width;
                answers.line2.y = end.y;
            }
            5 | 7 => {
                answers.line1.x = end.x + d_width;
                answers.line1.y = end.y;
                answers.line2.x = end.x - d_width;
                answers.line2.y = end.y;
            }
            _ => {}
        }
    }

    if m == 0.0 {
        match last {
            0 | 2 => {
                answers.line1.x = end.x;
                answers.line1.y = end.y - d_width;
                answers.line2.x = end.x;
                answers.line2.y = end.y + d_width;
            }
            1 | 3 => {
                answers.line1.x = end.x;
                answers.line1.y = end.y + d_width;
                answers.line2.x = end.x;
                answers.line2.y = end.y - d_width;
            }
            _ => {}
        }
    }

    if vertical != 0 && m != 0.0 {
        let b_perpendicular = end.y + end.x / m;
        let p1 =
            calc_true_intersect_double2(m, upper_b, -1.0 / m, b_perpendicular, 1, 1, (0.0, 0.0));
        let p2 =
            calc_true_intersect_double2(m, lower_b, -1.0 / m, b_perpendicular, 1, 1, (0.0, 0.0));
        match last {
            0 | 2 => {
                if p1.y < p2.y {
                    answers.line1 = p1;
                    answers.line2 = p2;
                } else {
                    answers.line1 = p2;
                    answers.line2 = p1;
                }
            }
            1 | 3 => {
                if p1.y > p2.y {
                    answers.line1 = p1;
                    answers.line2 = p2;
                } else {
                    answers.line1 = p2;
                    answers.line2 = p1;
                }
            }
            _ => {}
        }
    }
    answers
}

/// The slope, intercept and offset intercepts of one segment, with the
/// vertical flag, as `ConnectTrueDouble2` reads them from
/// `CalcTrueLinesDouble`.
#[derive(Clone, Copy, Debug, Default)]
struct SegmentLines {
    vertical: i32,
    m: f64,
    upper_b: f64,
    lower_b: f64,
}

fn segment_lines(n_width: i32, from: Pt, to: Pt) -> SegmentLines {
    let (vertical, r) = calc_true_lines_double(i64::from(n_width), from, to);
    if vertical == 0 {
        return SegmentLines::default();
    }
    SegmentLines {
        vertical,
        m: r[0],
        upper_b: r[5],
        lower_b: r[3],
    }
}

/// Upstream `ConnectTrueDouble2`: the edge points at the vertex `p2` between
/// segments `p1`-`p2` and `p2`-`p3`, for orientation `l_orient`
/// (0 above/above, 1 above/below, 2 below/above, 3 below/below).
pub(crate) fn connect_true_double2(
    n_width: i32,
    p1: Pt,
    p2: Pt,
    p3: Pt,
    l_orient: i32,
) -> ChannelPoints {
    let d_width = f64::from(n_width);
    let s1 = segment_lines(n_width, p1, p2);
    let s2 = segment_lines(n_width, p2, p3);
    let vertical = (s1.vertical, s2.vertical);
    let intersect = |b1: f64, b2: f64, orient: i32| -> Pt {
        let (x, y) =
            calc_true_intersect_double((s1.m, b1), (s2.m, b2), p2, vertical, d_width, orient);
        Pt::new(x, y)
    };
    let mut answer = ChannelPoints::default();
    // Edge 1 and edge 2 are each (segment 1 offset, segment 2 offset,
    // orientation code); the orientations alternate with the sides.
    let edges = match l_orient {
        0 => Some(((s1.upper_b, s2.upper_b, 0), (s1.lower_b, s2.lower_b, 3))),
        1 => Some(((s1.upper_b, s2.lower_b, 1), (s1.lower_b, s2.upper_b, 2))),
        2 => Some(((s1.lower_b, s2.upper_b, 2), (s1.upper_b, s2.lower_b, 1))),
        3 => Some(((s1.lower_b, s2.lower_b, 3), (s1.upper_b, s2.upper_b, 0))),
        _ => None,
    };
    if let Some((e1, e2)) = edges {
        let a = intersect(e1.0, e1.1, e1.2);
        let b = intersect(e2.0, e2.1, e2.2);
        answer.line1.x = a.x;
        answer.line1.y = a.y;
        answer.line2.x = b.x;
        answer.line2.y = b.y;
    }
    answer
}
