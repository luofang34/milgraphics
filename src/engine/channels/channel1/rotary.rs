//! The rotary-wing glyphs of `GetChannel1Double`: the attack helicopter arrow
//! (AAAAA) and the counterattack by fire rotary feature.

use super::Chan;
use crate::engine::base::{EngineError, Pt};
use crate::engine::channels::axad::CATKBYFIRE_SHIFT;
use crate::engine::channels::point_index::{at_i, mut_i, set_i};
use crate::engine::channels::{MAX_LENGTH, MIN_LENGTH};
use crate::engine::lineutility::arrow::get_arrow_head4_double;
use crate::engine::lineutility::basics::{calc_distance_double, mid_point_double};
use crate::engine::lineutility::extend::{
    extend_directed_line, extend_line_double, extend_line2_double, extend_true_line_perp_double,
};
use crate::engine::lineutility::slope::{calc_true_intersect_double2, calc_true_slope_double};

impl Chan<'_> {
    fn put(&mut self, offset: i32, p: Pt) -> Result<(), EngineError> {
        set_i(&mut self.points, self.l_count + self.u_count + offset, p)
    }

    /// The rotary symbol of the attack helicopter arrow: a small arrow
    /// across the channel and four side lines to the arrow's midpoint.
    pub(super) fn add_rotary_arrow(&mut self) -> Result<(), EngineError> {
        let (l, u) = (self.l_count, self.u_count);
        let mut too_short = false;

        let mut pt0 = at_i(&self.lower, l - 2)?;
        let mut pt1 = at_i(&self.lower, l - 1)?;
        let dist1 = calc_distance_double(pt0, pt1);
        let (vertical_lower, m_lower) = calc_true_slope_double(pt0, pt1);
        let b_lower = pt0.y - m_lower * pt0.x;

        pt0 = at_i(&self.upper, u - 2)?;
        pt1 = at_i(&self.upper, u - 1)?;
        let (vertical_upper, m_upper) = calc_true_slope_double(pt0, pt1);
        let b_upper = pt0.y - m_upper * pt0.x;
        let dist2 = calc_distance_double(pt0, pt1);

        let mut mid1 = calc_true_intersect_double2(
            m_lower,
            b_lower,
            m_upper,
            b_upper,
            vertical_lower,
            vertical_upper,
            (pt0.x, pt0.y),
        );
        let width = f64::from(self.width);
        if dist1 <= width || dist2 <= width {
            too_short = true;
            mid1 = mid_point_double(pt0, pt1, 0);
        }
        let a = calc_distance_double(pt0, pt1);
        let b = if a < 90.0 { a / 3.0 } else { 30.0 };

        let pt3 = at_i(&self.original, u - 2)?;
        let pt4 = at_i(&self.original, u - 1)?;
        let dpi = self.req.settings.dpi_scale_factor();
        let mut d = f64::from(self.width / 4);
        if d > MAX_LENGTH * dpi {
            d = MAX_LENGTH * dpi;
        }
        if d < MIN_LENGTH * dpi {
            d = MIN_LENGTH * dpi;
        }

        let (dir0, dir1) = if pt3.x != pt4.x { (3, 2) } else { (1, 0) };
        pt0 = extend_directed_line(pt3, pt4, mid1, dir0, 2.0 * d);
        pt0.style = 0;
        self.put(8, pt0)?;
        pt1 = extend_directed_line(pt3, pt4, mid1, dir1, 2.0 * d);
        pt1.style = 5;
        self.put(9, pt1)?;
        if pt3.x == pt4.x {
            mid1 = mid_point_double(pt0, pt1, 0);
        }

        let mut arrow_pts = [Pt::default(); 3];
        get_arrow_head4_double(pt0, pt1, d as i32, d as i32, &mut arrow_pts, 0)?;
        for (k, p) in (10..13).zip(arrow_pts) {
            self.put(k, p)?;
        }
        self.points_style(12, 5)?;

        let base_a = extend_true_line_perp_double(pt0, pt1, pt0, d / 2.0, 0)?;
        let base_b = extend_true_line_perp_double(pt0, pt1, pt0, -d / 2.0, 0)?;
        self.put(13, base_a)?;
        self.put(14, base_b)?;
        self.points_style(14, 5)?;

        let (lo0, lo1) = (at_i(&self.lower, l - 2)?, at_i(&self.lower, l - 1)?);
        let (up0, up1) = (at_i(&self.upper, l - 2)?, at_i(&self.upper, l - 1)?);
        self.put(15, extend_line2_double(lo0, mid1, b, 0))?;
        self.put(16, extend_line2_double(up0, mid1, b, 5))?;
        self.put(17, extend_line2_double(lo1, mid1, b, 0))?;
        self.put(18, extend_line2_double(up1, mid1, b, 5))?;
        if too_short {
            for k in l + u + 14..l + l + 19 {
                mut_i(&mut self.points, k)?.style = 5;
            }
        }
        Ok(())
    }

    fn points_style(&mut self, offset: i32, style: i32) -> Result<(), EngineError> {
        mut_i(&mut self.points, self.l_count + self.u_count + offset)?.style = style;
        Ok(())
    }

    /// The two-sided rotary feature at the back of the counterattack by fire
    /// arrow, with the arrow that closes it.
    pub(super) fn add_fire_rotary(&mut self) -> Result<(), EngineError> {
        let (l, u) = (self.l_count, self.u_count);
        let n = self.counter;
        let dist2 = calc_distance_double(self.next_to_last, self.last_point);
        let mut dist = self.dist;
        if dist2 > CATKBYFIRE_SHIFT {
            dist -= CATKBYFIRE_SHIFT;
        }
        let (up_a, up_b) = (at_i(&self.upper, u - 2)?, at_i(&self.upper, u - 1)?);
        let (lo_a, lo_b) = (at_i(&self.lower, l - 2)?, at_i(&self.lower, l - 1)?);
        let back = |extra: f64| -> (Pt, Pt) {
            if dist2 > 20.0 {
                (
                    extend_line_double(up_a, up_b, extra + dist),
                    extend_line_double(lo_a, lo_b, extra + dist),
                )
            } else {
                (
                    extend_line_double(up_a, up_b, -50.0),
                    extend_line_double(lo_a, lo_b, -50.0),
                )
            }
        };
        let (pt1, pt2) = back(5.0);
        let reach = 10.0 + (dist / 2.0).abs();
        let pt3 = extend_line2_double(pt2, pt1, reach, 18);
        let pt4 = extend_line2_double(pt1, pt2, reach, 5);
        let mut mid1 = mid_point_double(pt1, pt2, 17);
        set_i(&mut self.points, n - 9, pt3)?;
        set_i(&mut self.points, n - 6, pt4)?;

        let (pt1, pt2) = back(15.0);
        let half = (dist / 2.0).abs();
        let pt3 = extend_line2_double(pt2, pt1, half, 18);
        let pt4 = extend_line2_double(pt1, pt2, half, 18);
        let mut mid2 = mid_point_double(pt1, pt2, 18);
        set_i(&mut self.points, n - 8, pt3)?;
        set_i(&mut self.points, n - 7, pt4)?;
        set_i(&mut self.points, n - 5, mid2)?;
        if mid1.x == mid2.x && mid1.y == mid2.y {
            (mid1, mid2) = self.short_rotary(dist2)?;
        }
        let feature = if dist2 > 30.0 { 30.0 } else { 10.0 };
        let tip = extend_line2_double(mid1, mid2, feature, feature as i32);
        set_i(&mut self.points, n - 4, tip)?;
        let mut arrow_pts = [Pt::default(); 3];
        let size = feature as i32 / 2;
        get_arrow_head4_double(mid2, tip, size, size, &mut arrow_pts, 18)?;
        for (k, p) in (0..3).zip(arrow_pts) {
            let mut p = p;
            p.style = 18;
            set_i(&mut self.points, n - k - 1, p)?;
        }
        Ok(())
    }

    /// The rotary feature when the last segment is too short for the
    /// regular layout; returns the two midpoints.
    fn short_rotary(&mut self, dist2: f64) -> Result<(Pt, Pt), EngineError> {
        let n = self.counter;
        let mut feature = 15.0;
        let mid1 = extend_line2_double(self.next_to_last, self.arrow, 10.0, 17);
        let pt1 = extend_true_line_perp_double(self.last_point, mid1, mid1, feature, 18)?;
        let pt2 = extend_true_line_perp_double(self.last_point, mid1, mid1, -feature, 5)?;
        set_i(&mut self.points, n - 9, pt1)?;
        set_i(&mut self.points, n - 6, pt2)?;
        let mid2 = if dist2 > 30.0 {
            extend_line2_double(self.next_to_last, self.arrow, 20.0, 17)
        } else {
            extend_line2_double(self.next_to_last, self.arrow, feature, 17)
        };
        feature -= 10.0;
        let pt1 = extend_true_line_perp_double(self.last_point, mid2, mid2, feature, 18)?;
        let pt2 = extend_true_line_perp_double(self.last_point, mid2, mid2, -feature, 18)?;
        set_i(&mut self.points, n - 8, pt1)?;
        set_i(&mut self.points, n - 7, pt2)?;
        set_i(&mut self.points, n - 5, mid2)?;
        Ok((mid1, mid2))
    }
}
