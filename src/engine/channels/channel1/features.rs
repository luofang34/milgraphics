//! The repeated features along the client line of the wire and fence
//! channels: ellipses for concertina wire, X marks for fences.

use super::Chan;
use crate::engine::base::{EngineError, Pt};
use crate::engine::channels::point_index::{at_i, set_i};
use crate::engine::channels::scaled_size::get_scaled_size;
use crate::engine::lineutility::basics::{
    calc_distance_double, calc_segment_angle_double, mid_point_double,
};
use crate::engine::lineutility::transform::rotate_geometry_double;
use crate::engine::tactical_lines as lt;
use std::f64::consts::PI;

/// Points of one wire ellipse, including the pen-up copy of the last point.
const ELLIPSE_POINTS: usize = 37;

/// `pt` plus the whole-pixel part of `f * (b - a) / count`.
fn along(a: f64, b: f64, f: f64, count: i32) -> f64 {
    a + f64::from((f * (b - a) / f64::from(count)) as i32)
}

impl Chan<'_> {
    /// Adds the features to `points` starting at `ellipse_counter` and
    /// returns the new point count.
    pub(super) fn add_features(&mut self, ellipse_counter: i32) -> Result<i32, EngineError> {
        let x_size: i32 = if self.shift_lines {
            self.width / 8
        } else {
            self.width / 4
        };
        let mut counter = ellipse_counter;
        let mut x_counter = 0;
        let thickness = f64::from(self.req.tg.line_thickness) / 2.0;
        let increment =
            f64::from(x_size) + get_scaled_size(2.0, thickness, self.req.tg.pattern_scale);
        for j in 0..self.u_count - 1 {
            let (p0, p1) = (at_i(&self.original, j)?, at_i(&self.original, j + 1)?);
            let d = calc_distance_double(p0, p1);
            let how_many = (d / increment) as i32;
            let remainder = d - increment * f64::from(how_many);
            let angle = calc_segment_angle_double(p0, p1) + PI / 2.0;
            for k in 0..how_many {
                let skip = if self.draw == lt::SFENCE {
                    k % 4 == 0
                } else {
                    k % 2 == 0
                };
                if skip {
                    continue;
                }
                let f = f64::from(k) * (1.0 + remainder / d);
                let center = self.feature_center(j, f, how_many)?;
                counter = match self.draw {
                    lt::SINGLEC | lt::DOUBLEC | lt::TRIPLE => {
                        self.add_ellipse(center, angle, x_size, counter)?
                    }
                    lt::HWFENCE
                    | lt::LWFENCE
                    | lt::DOUBLEA
                    | lt::UNSP
                    | lt::SFENCE
                    | lt::DFENCE => {
                        x_counter += 1;
                        let next = self.add_x(center, angle, x_size, x_counter, counter)?;
                        if x_counter == 5 {
                            x_counter = 0;
                        }
                        next
                    }
                    _ => counter,
                };
            }
            if how_many == 0 && i32::try_from(self.points.len()).unwrap_or(i32::MAX) > counter {
                set_i(&mut self.points, counter, p0)?;
                counter += 1;
                let mut end = p1;
                end.style = 5;
                set_i(&mut self.points, counter, end)?;
                counter += 1;
            }
        }
        Ok(counter)
    }

    /// Where the `f`-th of `how_many` features on segment `j` sits.
    fn feature_center(&self, j: i32, f: f64, how_many: i32) -> Result<Pt, EngineError> {
        let on = |pts: &[Pt]| -> Result<Pt, EngineError> {
            let (a, b) = (at_i(pts, j)?, at_i(pts, j + 1)?);
            Ok(Pt::new(
                along(a.x, b.x, f, how_many),
                along(a.y, b.y, f, how_many),
            ))
        };
        if !self.shift_lines {
            return on(&self.original);
        }
        if self.draw == lt::DOUBLEC {
            return on(&self.upper);
        }
        let upper = on(&self.upper)?;
        let lower = on(&self.lower)?;
        Ok(mid_point_double(upper, lower, 0))
    }

    fn add_ellipse(
        &mut self,
        center: Pt,
        angle: f64,
        x_size: i32,
        mut counter: i32,
    ) -> Result<i32, EngineError> {
        let mut ellipse = [Pt::default(); ELLIPSE_POINTS];
        for l in 1..ELLIPSE_POINTS {
            let factor = (10.0 * l as f64) * PI / 180.0;
            if let Some(p) = ellipse.get_mut(l - 1) {
                p.x = center.x + f64::from(x_size) * factor.cos();
                p.y = center.y + f64::from(x_size / 2) * factor.sin();
                p.style = 0;
            }
        }
        rotate_geometry_double(&mut ellipse, 36, angle * 180.0 / PI)?;
        let mut closing = ellipse.get(35).copied().unwrap_or_default();
        closing.style = 5;
        if let Some(slot) = ellipse.get_mut(36) {
            *slot = closing;
        }
        for p in ellipse {
            set_i(&mut self.points, counter, p)?;
            counter += 1;
        }
        Ok(counter)
    }

    fn add_x(
        &mut self,
        center: Pt,
        angle: f64,
        x_size: i32,
        x_counter: i32,
        mut counter: i32,
    ) -> Result<i32, EngineError> {
        let s = f64::from(x_size);
        let mut x = [
            Pt::styled(center.x - s, center.y - s, 0),
            Pt::styled(center.x + s, center.y + s, 5),
            Pt::styled(center.x - s, center.y + s, 0),
            Pt::styled(center.x + s, center.y - s, 5),
        ];
        rotate_geometry_double(&mut x, 4, f64::from((angle * 180.0 / PI) as i32))?;
        for mut p in x {
            let lifts = match self.draw {
                lt::SFENCE => (2..=5).contains(&x_counter),
                lt::DFENCE => (3..=5).contains(&x_counter),
                _ => false,
            };
            if lifts {
                p.style = 5;
            }
            set_i(&mut self.points, counter, p)?;
            counter += 1;
        }
        Ok(counter)
    }
}
