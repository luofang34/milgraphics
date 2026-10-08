//! The per-spike line features of the stationary-front family in
//! `GetSFPointsDouble`: the rails joining flots and spikes for SF/USF, the
//! dots of SFG and the tick marks of SFY.

use super::sf::Sf;
use super::styled;
use crate::engine::base::{At, EngineError, Pt};
use crate::engine::lineutility::basics::calc_distance_double;
use crate::engine::lineutility::extend::{
    extend_along_line_double, extend_along_line_double_style, extend_directed_line_style,
};
use crate::engine::tactical_lines as tl;

impl Sf<'_> {
    fn push_spike(&mut self, p: Pt) -> Result<(), EngineError> {
        *self.spike.at_mut(self.n_spike)? = p;
        self.n_spike += 1;
        Ok(())
    }

    /// Dispatches the features for the line type after spike `k` of
    /// segment `j`; `last` is true for the segment's final spike.
    pub(super) fn add_features(
        &mut self,
        j: usize,
        k: usize,
        last: bool,
    ) -> Result<(), EngineError> {
        match self.style.line_type {
            tl::SF | tl::USF => self.add_rails(j, k, last),
            tl::SFG => {
                let (a, b) = (self.line.at(j)?, self.line.at(j + 1)?);
                let (s, e) = (self.spike_start.at(k)?, self.spike_end.at(k)?);
                let half = self.spike_size / 2.0;
                self.push_spike(extend_along_line_double_style(s, a, half, 22))?;
                self.push_spike(extend_along_line_double_style(e, b, half, 20))
            }
            tl::SFY => self.add_ticks(j, k),
            _ => Ok(()),
        }
    }

    fn add_rails(&mut self, j: usize, k: usize, last: bool) -> Result<(), EngineError> {
        let (a, b) = (self.line.at(j)?, self.line.at(j + 1)?);
        let (fs, fe) = (self.flot_start.at(k)?, self.flot_end.at(k)?);
        let (ss, se) = (self.spike_start.at(k)?, self.spike_end.at(k)?);
        let d1 = calc_distance_double(fs, ss);
        let half = d1 / 2.0;
        self.push_spike(styled(fs, 19))?;
        self.push_spike(extend_along_line_double_style(fs, b, half, 5))?;
        self.push_spike(styled(fe, 19))?;
        self.push_spike(extend_along_line_double_style(fe, a, half, 5))?;
        if last {
            let (fs1, fe1) = (self.flot_start.at(k + 1)?, self.flot_end.at(k + 1)?);
            self.push_spike(styled(fs1, 19))?;
            self.push_spike(extend_along_line_double_style(fs1, b, half, 5))?;
            self.push_spike(styled(fe1, 19))?;
            self.push_spike(extend_along_line_double_style(fe1, a, half, 5))?;
        }
        self.push_spike(styled(ss, 25))?;
        self.push_spike(extend_along_line_double_style(ss, a, half, 5))?;
        self.push_spike(styled(se, 25))?;
        self.push_spike(extend_along_line_double_style(se, b, half, 5))?;
        if self.style.line_type == tl::USF {
            self.push_spike(styled(fe, 19))?;
            self.push_spike(styled(fs, 5))?;
            if last {
                let (fs1, fe1) = (self.flot_start.at(k + 1)?, self.flot_end.at(k + 1)?);
                self.push_spike(styled(fe1, 19))?;
                self.push_spike(styled(fs1, 5))?;
            }
            self.push_spike(styled(se, 25))?;
            self.push_spike(styled(ss, 5))?;
        }
        Ok(())
    }

    /// Two three-quarter-length coloured stretches (blue then red) next to
    /// each flot, each with a tick across it.
    fn add_ticks(&mut self, j: usize, k: usize) -> Result<(), EngineError> {
        let (a, b) = (self.line.at(j)?, self.line.at(j + 1)?);
        let (fs, ss, se) = (
            self.flot_start.at(k)?,
            self.spike_start.at(k)?,
            self.spike_end.at(k)?,
        );
        let fe_next = self.flot_end.at(k + 1)?;
        let d1 = calc_distance_double(fs, ss);
        self.add_tick_group(ss, a, d1, (2, 3), (25, 5))?;
        let d2 = calc_distance_double(fe_next, se);
        self.add_tick_group(se, b, d2, (3, 2), (19, 5))?;
        Ok(())
    }

    /// Six points from `from` toward `toward`: blue start and end, red
    /// start and end at `d/4`, `d/2`, `d/2`, `3d/4`, then the ticks across
    /// the first and last, in the given directions with the given styles.
    fn add_tick_group(
        &mut self,
        from: Pt,
        toward: Pt,
        d: f64,
        dirs: (i32, i32),
        tick_styles: (i32, i32),
    ) -> Result<(), EngineError> {
        let ns = self.n_spike;
        let five = self.style.scaled(5.0);
        let marks = [
            (d / 4.0, 25),
            (d / 2.0, 5),
            (d / 2.0, 19),
            (3.0 * d / 4.0, 5),
        ];
        for (i, (dist, st)) in marks.iter().enumerate() {
            *self.spike.at_mut(ns + i)? =
                styled(extend_along_line_double(from, toward, *dist), *st);
        }
        let first = self.spike.at(ns)?;
        let fourth = self.spike.at(ns + 3)?;
        *self.spike.at_mut(ns + 4)? =
            extend_directed_line_style(first, toward, first, dirs.0, five, tick_styles.0);
        *self.spike.at_mut(ns + 5)? =
            extend_directed_line_style(fourth, toward, fourth, dirs.1, five, tick_styles.1);
        self.n_spike += 6;
        Ok(())
    }
}
