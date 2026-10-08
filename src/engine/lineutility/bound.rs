//! Port of `BoundOneSegment` and its helpers from lineutility.java: trims a
//! segment to a rectangle.

use crate::engine::base::Pt;

/// Upstream `PointInBounds2`: whether `pt` lies inside the rectangle.
fn point_in_bounds2(pt: Pt, ul: Pt, lr: Pt) -> bool {
    pt.x <= lr.x && pt.x >= ul.x && pt.y <= lr.y && pt.y >= ul.y
}

/// Upstream `intersectSegment`: where the segment pt0-pt1 crosses the
/// axis-aligned side sidePt0-sidePt1, if it does.
fn intersect_segment(pt0: Pt, pt1: Pt, side_pt0: Pt, side_pt1: Pt) -> Option<Pt> {
    if pt0.x == pt1.x {
        return None;
    }
    let m = (pt1.y - pt0.y) / (pt1.x - pt0.x);
    if side_pt0.x == side_pt1.x {
        let (upper, lower) = if side_pt0.y < side_pt1.y {
            (side_pt0, side_pt1)
        } else {
            (side_pt1, side_pt0)
        };
        let dx = upper.x - pt0.x;
        let pt = Pt::new(upper.x, pt0.y + m * dx);
        let within_x = (pt0.x <= pt.x && pt.x <= pt1.x) || (pt0.x >= pt.x && pt.x >= pt1.x);
        if within_x && upper.y <= pt.y && pt.y <= lower.y {
            return Some(pt);
        }
    } else {
        let (left, right) = if side_pt0.x < side_pt1.x {
            (side_pt0, side_pt1)
        } else {
            (side_pt1, side_pt0)
        };
        let dy = left.y - pt0.y;
        let pt = Pt::new(pt0.x + dy / m, left.y);
        let within_y = (pt0.y <= pt.y && pt.y <= pt1.y) || (pt0.y >= pt.y && pt.y >= pt1.y);
        if within_y && left.x <= pt.x && pt.x <= right.x {
            return Some(pt);
        }
    }
    None
}

/// Upstream `BoundOneSegment`: the part of pt0-pt1 inside the rectangle
/// `ul`-`lr`, or `None` when the segment misses it.
pub(crate) fn bound_one_segment(pt0: Pt, pt1: Pt, ul: Pt, lr: Pt) -> Option<[Pt; 2]> {
    if (pt0.y < ul.y && pt1.y < ul.y)
        || (pt0.y > lr.y && pt1.y > lr.y)
        || (pt0.x < ul.x && pt1.x < ul.x)
        || (pt0.x > lr.x && pt1.x > lr.x)
    {
        return None;
    }
    if pt0.x == pt1.x {
        let clamp = |mut p: Pt| {
            if p.y < ul.y {
                p.y = ul.y;
            }
            if p.y > lr.y {
                p.y = lr.y;
            }
            p
        };
        return Some([clamp(pt0), clamp(pt1)]);
    }
    let ur = Pt::new(lr.x, ul.y);
    let ll = Pt::new(ul.x, lr.y);
    let sides = [(ll, ul), (ul, ur), (ur, lr), (ll, lr)];
    // A side counts as used once pt0's search has reached it, even if it
    // produced nothing, and pt1's search then skips it.
    let mut used = [false; 4];
    let mut p0 = point_in_bounds2(pt0, ul, lr).then_some(pt0);
    for ((a, b), flag) in sides.iter().zip(used.iter_mut()) {
        if p0.is_some() {
            break;
        }
        p0 = intersect_segment(pt0, pt1, *a, *b);
        *flag = true;
    }
    let mut p1 = point_in_bounds2(pt1, ul, lr).then_some(pt1);
    for ((a, b), flag) in sides.iter().zip(used.iter()) {
        if p1.is_some() {
            break;
        }
        if !*flag {
            p1 = intersect_segment(pt1, pt0, *a, *b);
        }
    }
    Some([p0?, p1?])
}
