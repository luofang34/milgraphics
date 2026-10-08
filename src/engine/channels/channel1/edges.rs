//! The "get the channel array" stage of `GetChannel1Double`: derives the
//! upper and lower channel edges from the client points, per line-type
//! family.

use super::Chan;
use crate::engine::base::{EngineError, Pt};
use crate::engine::channels::lines::{get_channel_array2_double, get_triple_count_double};
use crate::engine::channels::point_index::{at_i, set_i};
use crate::engine::lineutility::basics::{mid_point_double, reverse_points_double2};
use crate::engine::lineutility::channel_pixels::{move_channel_pixels, move_single_c_pixels};
use crate::engine::tactical_lines as lt;

/// Edge arrays are always built for one printer pixel.
const N_PRINTER: i32 = 1;

impl Chan<'_> {
    pub(super) fn build_edges(&mut self) -> Result<(), EngineError> {
        match self.draw {
            lt::MAIN
            | lt::MAIN_STRAIGHT
            | lt::SPT
            | lt::SPT_STRAIGHT
            | lt::FRONTAL_ATTACK
            | lt::TURNING_MOVEMENT
            | lt::MOVEMENT_TO_CONTACT
            | lt::CATK
            | lt::CATKBYFIRE
            | lt::TRIPLE
            | lt::DOUBLEC
            | lt::SINGLEC
            | lt::HWFENCE
            | lt::BBS_LINE
            | lt::LWFENCE
            | lt::UNSP
            | lt::DOUBLEA
            | lt::DFENCE
            | lt::SFENCE
            | lt::CHANNEL
            | lt::CHANNEL_FLARED
            | lt::CHANNEL_DASHED => self.build_offset_edges(),
            lt::LC => self.build_lc_edges(),
            lt::AAAAA | lt::AIRAOA => self.build_crossed_edges(),
            _ => Ok(()),
        }
    }

    fn channel_edges(&mut self) -> Result<(), EngineError> {
        self.upper = get_channel_array2_double(
            N_PRINTER,
            &self.upper,
            1,
            self.u_count,
            self.draw,
            self.width,
        )?;
        self.lower = get_channel_array2_double(
            N_PRINTER,
            &self.lower,
            0,
            self.l_count,
            self.draw,
            self.width,
        )?;
        Ok(())
    }

    fn build_offset_edges(&mut self) -> Result<(), EngineError> {
        self.counter = get_triple_count_double(&self.upper, self.u_count, self.draw)?;
        let mut original = Vec::new();
        for k in 0..self.u_count {
            original.push(at_i(&self.upper, k)?);
        }
        self.original = original.clone();
        if matches!(
            self.draw,
            lt::TRIPLE
                | lt::DOUBLEC
                | lt::SINGLEC
                | lt::HWFENCE
                | lt::BBS_LINE
                | lt::LWFENCE
                | lt::UNSP
                | lt::DOUBLEA
                | lt::DFENCE
                | lt::SFENCE
        ) {
            let mut lower = Vec::new();
            for k in 0..self.l_count {
                lower.push(at_i(&original, k)?);
            }
            let mut upper = Vec::new();
            for k in 0..self.u_count {
                upper.push(at_i(&original, k)?);
            }
            self.lower = lower;
            self.upper = upper;
        }
        move_single_c_pixels(self.draw, &mut self.upper)?;
        move_single_c_pixels(self.draw, &mut self.lower)?;
        move_channel_pixels(&mut self.upper)?;
        move_channel_pixels(&mut self.lower)?;
        if self.shift_lines {
            self.width = self.width.wrapping_mul(2);
        }
        self.channel_edges()?;
        if self.shift_lines {
            self.restore_shifted_edge()?;
        }
        Ok(())
    }

    /// With shifted lines one edge is the client line itself.
    fn restore_shifted_edge(&mut self) -> Result<(), EngineError> {
        match self.draw {
            lt::SINGLEC => self.lower = self.original.clone(),
            lt::DOUBLEC => {
                for j in 0..self.upper.len() {
                    let jj = i32::try_from(j).unwrap_or(i32::MAX);
                    let mid =
                        mid_point_double(at_i(&self.lower, jj)?, at_i(&self.original, jj)?, 0);
                    set_i(&mut self.upper, jj, mid)?;
                }
            }
            _ => self.upper = self.original.clone(),
        }
        Ok(())
    }

    fn build_lc_edges(&mut self) -> Result<(), EngineError> {
        if self.shift_lines {
            self.original = self.upper.clone();
            self.width = self.width.wrapping_mul(2);
        }
        self.channel_edges()?;
        if self.shift_lines {
            self.upper = self.original.clone();
        }
        let (p0, p1) = (at_i(&self.upper, 0)?, at_i(&self.upper, 1)?);
        if p0.x > p1.x && p0.y != p1.y {
            self.reverse_upper = 1;
            reverse_points_double2(&mut self.lower, self.l_count)?;
        } else if p0.x > p1.x && p0.y == p1.y {
            self.reverse_upper = 0;
            reverse_points_double2(&mut self.upper, self.u_count)?;
        } else if p0.x < p1.x || (p0.y > p1.y && p0.x == p1.x) {
            self.reverse_upper = 1;
            reverse_points_double2(&mut self.lower, self.l_count)?;
        } else if p0.y < p1.y && p0.x == p1.x {
            self.reverse_upper = 0;
            reverse_points_double2(&mut self.upper, self.u_count)?;
        }
        Ok(())
    }

    /// The helicopter and airborne arrows cross their edge ends so the
    /// arrowhead closes the channel.
    fn build_crossed_edges(&mut self) -> Result<(), EngineError> {
        self.original = self.upper.clone();
        self.channel_edges()?;
        let temp1: Pt = at_i(&self.lower, self.l_count - 1)?;
        let temp2: Pt = at_i(&self.upper, self.u_count - 1)?;
        set_i(&mut self.lower, self.l_count - 1, temp2)?;
        set_i(&mut self.upper, self.u_count - 1, temp1)
    }
}
