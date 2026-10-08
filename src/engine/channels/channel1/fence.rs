//! The wire, fence and plain channel branch of `GetChannel1Double`: loads the
//! two edges into one point list, then adds the repeated ellipses or X
//! features along the client line.

use super::Chan;
use crate::engine::base::{EngineError, Pt};
use crate::engine::channels::lines::fence_type;
use crate::engine::channels::point_index::{at_i, mut_i, new_pts, set_i};
use crate::engine::lineutility::basics::resize_array;
use crate::engine::tactical_lines as lt;

impl Chan<'_> {
    pub(super) fn load_fence_points(&mut self) -> Result<(), EngineError> {
        let mut points = new_pts(self.counter)?;
        for p in &mut points {
            *p = Pt::new(self.origin.x, self.origin.y);
        }
        self.points = points;
        self.load_first_edge()?;
        mut_i(&mut self.points, self.l_count - 1)?.style = 5;
        self.load_second_edge()?;
        mut_i(&mut self.points, self.l_count + self.u_count - 1)?.style = 5;

        let mut ellipse_counter = self.l_count + self.u_count;
        if !matches!(
            self.draw,
            lt::BBS_LINE
                | lt::CHANNEL
                | lt::CHANNEL_DASHED
                | lt::CHANNEL_FLARED
                | lt::SPT_STRAIGHT
                | lt::MAIN_STRAIGHT
        ) {
            ellipse_counter = self.add_features(ellipse_counter)?;
            self.points = resize_array(&self.points, ellipse_counter)?;
            self.counter = i32::try_from(self.points.len()).unwrap_or(i32::MAX);
        }
        if fence_type(self.draw) == 1 {
            self.settle_fence_styles(ellipse_counter)?;
        }
        Ok(())
    }

    /// The first `l_count` points: the lower edge or the client line,
    /// depending on the type.
    fn load_first_edge(&mut self) -> Result<(), EngineError> {
        for k in 0..self.l_count {
            let mut p = match self.draw {
                lt::TRIPLE
                | lt::HWFENCE
                | lt::CHANNEL
                | lt::CHANNEL_FLARED
                | lt::CHANNEL_DASHED
                | lt::SINGLEC => at_i(&self.lower, k)?,
                lt::DOUBLEC => {
                    if at_i(&self.original, 0)?.x < at_i(&self.original, 1)?.x {
                        at_i(&self.original, k)?
                    } else {
                        at_i(&self.upper, k)?
                    }
                }
                _ => at_i(&self.original, k)?,
            };
            if matches!(self.draw, lt::LWFENCE | lt::UNSP) {
                p.style = 5;
            }
            set_i(&mut self.points, k, p)?;
        }
        Ok(())
    }

    /// The next `u_count` points: the upper edge or the client line.
    fn load_second_edge(&mut self) -> Result<(), EngineError> {
        for k in 0..self.u_count {
            let mut p = match self.draw {
                lt::TRIPLE
                | lt::HWFENCE
                | lt::CHANNEL
                | lt::CHANNEL_FLARED
                | lt::CHANNEL_DASHED
                | lt::LWFENCE => at_i(&self.upper, k)?,
                lt::DOUBLEC => {
                    if at_i(&self.original, 0)?.x < at_i(&self.original, 1)?.x {
                        at_i(&self.upper, k)?
                    } else {
                        at_i(&self.original, k)?
                    }
                }
                lt::SINGLEC => at_i(&self.lower, k)?,
                _ => at_i(&self.original, k)?,
            };
            if self.draw == lt::UNSP {
                p.style = 5;
            }
            set_i(&mut self.points, self.l_count + k, p)?;
        }
        Ok(())
    }

    /// If no segment was long enough to carry features the line is drawn
    /// solid; otherwise the unused tail closes with a pen lift.
    fn settle_fence_styles(&mut self, ellipse_counter: i32) -> Result<(), EngineError> {
        if ellipse_counter <= self.l_count + self.u_count {
            for k in 0..self.l_count + self.u_count {
                let p = mut_i(&mut self.points, k)?;
                if p.style != 5 {
                    p.style = 0;
                }
            }
        } else {
            let len = i32::try_from(self.points.len()).unwrap_or(i32::MAX);
            for k in ellipse_counter - 1..len {
                mut_i(&mut self.points, k)?.style = 5;
            }
        }
        Ok(())
    }

    /// The bounding-box line: lower edge, upper edge reversed, then the
    /// first point again.
    pub(super) fn load_bbs_points(&mut self) -> Result<(), EngineError> {
        let mut points = new_pts(self.l_count + self.u_count + 1)?;
        for j in 0..self.l_count {
            set_i(&mut points, j, at_i(&self.lower, j)?)?;
        }
        for j in 0..self.u_count {
            set_i(
                &mut points,
                j + self.l_count,
                at_i(&self.upper, self.u_count - 1 - j)?,
            )?;
        }
        let first = at_i(&points, 0)?;
        let last = i32::try_from(points.len()).unwrap_or(i32::MAX) - 1;
        set_i(&mut points, last, first)?;
        self.points = points;
        Ok(())
    }
}
