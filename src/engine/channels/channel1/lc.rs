//! The line of contact branch of `GetChannel1Double`: the two edges become
//! flot lines in the enemy and friendly line styles (25 and 26).

use super::Chan;
use crate::engine::base::{EngineError, Pt};
use crate::engine::channels::ChannelExternals;
use crate::engine::channels::point_index::{at_i, mut_i, new_pts, set_i};

/// Line style of the flot on the side drawn in the enemy colour.
const STYLE_ENEMY_SIDE: i32 = 25;
/// Line style of the flot on the other side.
const STYLE_OTHER_SIDE: i32 = 26;

impl Chan<'_> {
    /// Flots both edges. False when either flot has no points.
    pub(super) fn load_lc_points<E: ChannelExternals>(
        &mut self,
        ext: &E,
    ) -> Result<bool, EngineError> {
        let segment = self.scaled(20.0);
        let upper_count = ext.flot_count(&self.upper, segment, self.u_count)?;
        let lower_count = ext.flot_count(&self.lower, segment, self.l_count)?;
        if upper_count <= 0 || lower_count <= 0 {
            return Ok(false);
        }
        let mut upper_flot = new_pts(self.u_count.max(upper_count))?;
        for k in 0..self.u_count {
            set_i(&mut upper_flot, k, at_i(&self.upper, k)?)?;
        }
        let mut lower_flot = new_pts(self.l_count.max(lower_count))?;
        for k in 0..self.l_count {
            set_i(&mut lower_flot, k, at_i(&self.lower, k)?)?;
        }
        let upper_count = ext.flot(&mut upper_flot, segment, self.u_count)?;
        let lower_count = ext.flot(&mut lower_flot, segment, self.l_count)?;
        let mut points = new_pts(upper_count + lower_count)?;
        self.counter = lower_count + upper_count;
        let (upper_style, lower_style) = if self.reverse_upper == 1 {
            (STYLE_ENEMY_SIDE, STYLE_OTHER_SIDE)
        } else {
            (STYLE_OTHER_SIDE, STYLE_ENEMY_SIDE)
        };
        copy_flot(&upper_flot, &mut points, 0, upper_count, upper_style)?;
        copy_flot(
            &lower_flot,
            &mut points,
            upper_count,
            lower_count,
            lower_style,
        )?;
        self.points = points;
        Ok(true)
    }
}

/// Copies `count` flot points to `dest` at `offset` with `style`, ending the
/// run with style 5.
fn copy_flot(
    flot: &[Pt],
    dest: &mut [Pt],
    offset: i32,
    count: i32,
    style: i32,
) -> Result<(), EngineError> {
    for k in 0..count {
        let mut p = at_i(flot, k)?;
        p.style = style;
        set_i(dest, offset + k, p)?;
    }
    if count > 0 {
        mut_i(dest, offset + count - 1)?.style = 5;
    }
    Ok(())
}
