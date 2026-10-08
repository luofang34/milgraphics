//! The axis-of-advance branch of `GetChannel1Double`: the two edges plus the
//! arrowhead and the type's own glyph.

use super::Chan;
use crate::engine::base::{EngineError, Pt};
use crate::engine::channels::ChannelExternals;
use crate::engine::channels::axad::{AxadInput, get_axad_double};
use crate::engine::channels::point_index::{mut_i, new_pts};
use crate::engine::tactical_lines as lt;

impl Chan<'_> {
    pub(super) fn load_arrow_points<E: ChannelExternals>(
        &mut self,
        ext: &E,
    ) -> Result<(), EngineError> {
        let base = self.l_count + self.u_count;
        self.counter = match self.draw {
            lt::AAAAA => base + 19,
            lt::FRONTAL_ATTACK => base + 15,
            lt::TURNING_MOVEMENT => base + 14,
            lt::MOVEMENT_TO_CONTACT => base + 24,
            lt::CATKBYFIRE => base + 17,
            _ => base + 8,
        };
        let mut points = new_pts(self.counter)?;
        for p in &mut points {
            *p = Pt::new(self.origin.x, self.origin.y);
        }
        self.points = points;
        let input = AxadInput {
            printer: 1.0,
            lower: &mut self.lower,
            upper: &mut self.upper,
            arrow: self.arrow,
            draw_this: self.draw,
            offset_factor: self.offset_factor,
        };
        get_axad_double(input, &mut self.points)?;

        if matches!(self.draw, lt::CATK | lt::CATKBYFIRE) {
            for k in 0..self.counter {
                let p = mut_i(&mut self.points, k)?;
                if p.style != 5 {
                    p.style = 1;
                }
            }
        }
        match self.draw {
            lt::AAAAA => self.add_rotary_arrow()?,
            lt::CATKBYFIRE => self.add_fire_rotary()?,
            lt::FRONTAL_ATTACK => self.add_frontal_attack_bar()?,
            lt::TURNING_MOVEMENT => self.add_turning_movement_bar()?,
            lt::MOVEMENT_TO_CONTACT => self.add_contact_cover(ext)?,
            _ => {}
        }
        if matches!(self.draw, lt::FRONTAL_ATTACK | lt::TURNING_MOVEMENT) {
            self.add_letter_glyph()?;
        }
        Ok(())
    }
}
