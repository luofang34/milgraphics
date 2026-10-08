//! The glyphs of the frontal attack, turning movement and movement to
//! contact arrows in `GetChannel1Double`.

use super::Chan;
use crate::engine::base::{At, EngineError, Pt};
use crate::engine::channels::ChannelExternals;
use crate::engine::channels::point_index::{at_i, set_i};
use crate::engine::lineutility::basics::{calc_distance_double, mid_point_double};
use crate::engine::lineutility::extend::{
    extend_along_line_double, extend_along_line_double2, extend_directed_line,
    extend_directed_line_style,
};
use crate::engine::lineutility::relative::{closest_point_on_line, point_relative_to_line_at};
use crate::engine::lineutility::slope::{calc_direction_from_line, reverse_direction};
use crate::engine::lineutility::{EXTEND_ABOVE, EXTEND_BELOW};

impl Chan<'_> {
    fn base(&self) -> i32 {
        self.l_count + self.u_count
    }

    fn put_styled(&mut self, offset: i32, mut p: Pt, style: i32) -> Result<(), EngineError> {
        p.style = style;
        let base = self.base();
        set_i(&mut self.points, base + offset, p)
    }

    /// The bar across the frontal attack arrowhead, perpendicular to the
    /// arrow's axis.
    pub(super) fn add_frontal_attack_bar(&mut self) -> Result<(), EngineError> {
        let base = self.base();
        let left = at_i(&self.points, base + 1)?;
        let tip = at_i(&self.points, base + 6)?;
        let right = at_i(&self.points, base + 5)?;
        let mid = mid_point_double(left, right, 0);
        let width = f64::from(self.width);
        let pt0 = extend_along_line_double(right, mid, width);
        let pt1 = extend_along_line_double(pt0, mid, width);
        let first = point_relative_to_line_at(pt0, pt1, pt0, tip);
        self.put_styled(8, first, 0)?;
        let second = point_relative_to_line_at(pt0, pt1, pt1, tip);
        self.put_styled(9, second, 5)
    }

    /// The bar across the turning movement arrow's last segment.
    pub(super) fn add_turning_movement_bar(&mut self) -> Result<(), EngineError> {
        let px = &self.req.tg.pixels;
        let n = px.len();
        let (pt0, pt1) = if n == 3 {
            (
                px.at(1)?,
                closest_point_on_line(px.at(0)?, px.at(1)?, px.at(2)?),
            )
        } else {
            (
                px.at(n
                    .checked_sub(2)
                    .ok_or(EngineError::Degenerate("no pixels"))?)?,
                px.at(n
                    .checked_sub(3)
                    .ok_or(EngineError::Degenerate("no pixels"))?)?,
            )
        };
        let mid = mid_point_double(pt0, pt1, 0);
        let half = f64::from(self.width / 2);
        let above = extend_directed_line_style(pt0, pt1, mid, EXTEND_ABOVE, half, 0);
        let below = extend_directed_line_style(pt0, pt1, mid, EXTEND_BELOW, half, 5);
        let base = self.base();
        set_i(&mut self.points, base + 8, above)?;
        set_i(&mut self.points, base + 9, below)
    }

    /// The cover glyph on the movement to contact arrowhead.
    pub(super) fn add_contact_cover<E: ChannelExternals>(
        &mut self,
        ext: &E,
    ) -> Result<(), EngineError> {
        let base = self.base();
        let pt0 = at_i(&self.points, base + 1)?;
        let tip = at_i(&self.points, base + 6)?;
        let pt1 = at_i(&self.points, base + 5)?;
        let direction1 = reverse_direction(calc_direction_from_line(pt0, tip, pt1));
        let direction2 = reverse_direction(calc_direction_from_line(pt1, tip, pt0));
        let eighth = f64::from(self.width) / 8.0;
        let mid1 = mid_point_double(pt0, tip, 0);
        let mid1 = extend_directed_line(pt0, tip, mid1, direction1, eighth);
        let mid2 = mid_point_double(pt1, tip, 0);
        let mid2 = extend_directed_line(pt1, tip, mid2, direction2, eighth);

        let width = f64::from(self.width);
        let mut cover = vec![Pt::default(); 16];
        cover[0] = extend_directed_line(pt0, tip, mid1, direction1, width);
        cover[1] = mid1;
        cover[2] = mid2;
        cover[3] = extend_directed_line(pt1, tip, mid2, direction2, width);
        let count = ext.dism_cover_rev_c(&mut cover, self.draw, 4, self.req.settings)?;
        for i in 0..count {
            set_i(&mut self.points, base + 8 + i, at_i(&cover, i)?)?;
        }
        Ok(())
    }

    /// The "A" (frontal attack) or "T" (turning movement) drawn inside the
    /// arrowhead.
    pub(super) fn add_letter_glyph(&mut self) -> Result<(), EngineError> {
        let base = self.base();
        let [mut pt0, mut pt1, mut pt2, mut pt3] = self.glyph_box()?;
        if pt0.y > pt2.y && pt1.y > pt3.y {
            std::mem::swap(&mut pt0, &mut pt2);
            std::mem::swap(&mut pt1, &mut pt3);
        }
        if pt0.x > pt1.x && pt2.x > pt3.x {
            std::mem::swap(&mut pt0, &mut pt1);
            std::mem::swap(&mut pt2, &mut pt3);
        }
        if self.draw == crate::engine::tactical_lines::FRONTAL_ATTACK {
            self.put_styled(10, pt2, 0)?;
            let apex = mid_point_double(pt0, pt1, 0);
            self.put_styled(11, apex, 0)?;
            self.put_styled(12, pt3, 5)?;
            let left_mid = mid_point_double(
                at_i(&self.points, base + 10)?,
                at_i(&self.points, base + 11)?,
                0,
            );
            self.put_styled(13, left_mid, 0)?;
            let right_mid = mid_point_double(
                at_i(&self.points, base + 11)?,
                at_i(&self.points, base + 12)?,
                5,
            );
            self.put_styled(14, right_mid, 5)
        } else {
            self.put_styled(10, mid_point_double(pt0, pt1, 0), 0)?;
            self.put_styled(11, mid_point_double(pt2, pt3, 5), 5)?;
            self.put_styled(12, pt0, 0)?;
            self.put_styled(13, pt1, 5)
        }
    }

    /// The glyph's corner points (top left, top right, bottom left, bottom
    /// right) shrunk to leave spacing between the arrow lines and to a
    /// height twice the width.
    fn glyph_box(&self) -> Result<[Pt; 4], EngineError> {
        let base = self.base();
        let mut pt0 = at_i(&self.points, base + 2)?;
        let mut pt1 = mid_point_double(
            at_i(&self.points, base + 1)?,
            at_i(&self.points, base + 6)?,
            0,
        );
        let mut pt2 = at_i(&self.points, base + 4)?;
        let mut pt3 = mid_point_double(
            at_i(&self.points, base + 5)?,
            at_i(&self.points, base + 6)?,
            0,
        );
        let mut dist = calc_distance_double(pt0, pt2) / 4.0;
        pt0 = extend_along_line_double2(pt0, pt2, dist);
        pt1 = extend_along_line_double2(pt1, pt3, dist);
        pt2 = extend_along_line_double2(pt2, pt0, dist);
        pt3 = extend_along_line_double2(pt3, pt1, dist);

        if calc_distance_double(pt0, pt2) > 2.0 * calc_distance_double(pt0, pt1) {
            dist = calc_distance_double(pt0, pt1);
            let mid = mid_point_double(pt0, pt2, 0);
            pt0 = extend_along_line_double2(mid, pt0, dist);
            pt2 = extend_along_line_double2(mid, pt2, dist);
            let mid = mid_point_double(pt1, pt3, 0);
            pt1 = extend_along_line_double2(mid, pt1, dist);
            pt3 = extend_along_line_double2(mid, pt3, dist);
        } else if 2.0 * calc_distance_double(pt0, pt1) > calc_distance_double(pt0, pt2) {
            dist = calc_distance_double(pt0, pt2) / 2.0;
            pt1 = extend_along_line_double2(pt0, pt1, dist);
            pt3 = extend_along_line_double2(pt2, pt3, dist);
        }
        Ok([pt0, pt1, pt2, pt3])
    }
}
