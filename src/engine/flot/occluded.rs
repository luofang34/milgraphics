//! Port of the occluded-front builders in flot.java
//! (`GetOccludedCountDouble`, `GetOccludedPointsDouble`) for OCCLUDED, UOF
//! and, through the shared point layout, SF.

use super::segment::{FlipState, get_flot_segment2};
use super::spike::{SpikeTip, push_spike_tip};
use super::{FlotStyle, budgeted_len, count, int_coords, set_xy_style, store, styled};
use crate::engine::base::{At, EngineError, Pt, idx};
use crate::engine::lineutility::basics::calc_distance_double;
use crate::engine::lineutility::extend::{extend_line_double, extend_line2_double};
use crate::engine::lineutility::slope::calc_true_slope_double;
use crate::engine::tactical_lines as tl;

/// Upstream `GetOccludedCountDouble`: the points an occluded front over the
/// first `num_pts` points needs (10 per flot and 3 per spike, at a pitch of
/// 50 pixels, never fewer than 13 per point).
pub(crate) fn get_occluded_count_double(pts: &[Pt], num_pts: i32) -> Result<i32, EngineError> {
    let n = count(num_pts)?;
    let mut total = 0_i32;
    for j in 0..n.saturating_sub(1) {
        let d = calc_distance_double(pts.at(j)?, pts.at(j + 1)?);
        let segs = ((d / 50.0) as i32).max(1);
        total = total
            .wrapping_add(segs.wrapping_mul(10))
            .wrapping_add(segs.wrapping_mul(3));
    }
    Ok(total.max(num_pts.wrapping_mul(13)).max(num_pts))
}

/// The working arrays of `GetOccludedPointsDouble`.
struct Build<'a> {
    style: &'a FlotStyle,
    line: Vec<Pt>,
    spike_size: f64,
    increment: f64,
    flot: Vec<Pt>,
    spike: Vec<Pt>,
    n_flot: usize,
    n_spike: usize,
    flots: Vec<i32>,
    temp: Pt,
}

fn put(v: &mut [Pt], i: usize, p: Pt) -> Result<(), EngineError> {
    *v.at_mut(i)? = p;
    Ok(())
}

impl Build<'_> {
    fn is_occluded_kind(&self) -> bool {
        matches!(self.style.line_type, tl::OCCLUDED | tl::UOF)
    }

    /// Moves flot point `nf`, the first of its flot, onto the line.
    fn straighten_start(&mut self, j: usize, nf: usize) -> Result<(), EngineError> {
        let (a, b) = (self.line.at(j)?, self.line.at(j + 1)?);
        let cur = self.flot.at(nf)?;
        let (d1, d2) = (calc_distance_double(a, cur), calc_distance_double(b, cur));
        let mut p = if d2 > d1 {
            extend_line_double(b, a, -d1)
        } else {
            extend_line_double(a, b, -d2)
        };
        p.style = if self.style.line_type == tl::UOF {
            0
        } else {
            9
        };
        put(&mut self.flot, nf, p)
    }

    /// Moves flot point `nf`, the last of its flot, a spike length past the
    /// flot's start along the line.
    fn shape_end(&mut self, j: usize, nf: usize) -> Result<(), EngineError> {
        if self.is_occluded_kind() {
            let (a, b) = (self.line.at(j)?, self.line.at(j + 1)?);
            let first = self.flot.at(nf - 9)?;
            let (d1, d2) = (
                calc_distance_double(a, first),
                calc_distance_double(b, first),
            );
            let mut p = if d2 > d1 {
                extend_line_double(b, a, -d1 - self.spike_size)
            } else {
                extend_line_double(a, b, -d2 + self.spike_size)
            };
            p.style = match self.style.line_type {
                tl::OCCLUDED => 10,
                _ => 5,
            };
            put(&mut self.flot, nf, p)?;
        }
        if self.style.line_type == tl::SF {
            self.flot.at_mut(nf)?.style = 23;
        }
        Ok(())
    }

    fn add_flots(
        &mut self,
        j: usize,
        n_flots: i32,
        vb: &[i32],
        state: &mut FlipState,
    ) -> Result<(), EngineError> {
        let mut points = vec![0_i32; budgeted_len(n_flots, 30)?];
        get_flot_segment2(self.style, vb, j, &mut points, state)?;
        let mut k = 0;
        for _ in 0..idx(n_flots.wrapping_mul(10), 0)? {
            let nf = self.n_flot;
            let (x, y) = (points.at(k)?, points.at(k + 1)?);
            set_xy_style(&mut self.flot, nf, (x, y), 9)?;
            if nf % 10 == 0 {
                self.straighten_start(j, nf)?;
            }
            if (nf + 1) % 10 == 0 {
                self.shape_end(j, nf)?;
            }
            k += 3;
            self.n_flot += 1;
        }
        Ok(())
    }

    /// The base point of the spike after flot `k`, kept in `self.temp`.
    fn spike_base(&mut self, j: usize, k: usize) -> Result<(), EngineError> {
        let sum: i32 = self.flots.iter().take(j + 1).sum();
        let at = idx(sum, 0)? * 10 + 10 * k;
        let (a, b) = (self.line.at(j)?, self.line.at(j + 1)?);
        let base = self.flot.at(at)?;
        let (d1, d2) = (calc_distance_double(a, base), calc_distance_double(b, base));
        let half = match self.style.line_type {
            tl::OCCLUDED | tl::UOF => Some(self.increment / 2.0),
            tl::SF => Some(self.increment / 8.0),
            _ => None,
        };
        if let Some(h) = half {
            self.temp = if d2 > d1 {
                extend_line2_double(b, a, -d1 - h, 0)
            } else {
                extend_line2_double(a, b, -d2 + h, 0)
            };
        }
        Ok(())
    }

    /// One spike (base, tip, second base) after a flot, three points.
    fn add_spike(&mut self, j: usize, k: usize, not_vertical: bool) -> Result<(), EngineError> {
        let (a, b) = (self.line.at(j)?, self.line.at(j + 1)?);
        self.spike_base(j, k)?;
        let seg_len = calc_distance_double(a, b);
        let spike_len = calc_distance_double(a, self.temp);
        let too_long = spike_len + self.spike_size >= seg_len;
        let base = if too_long { b } else { self.temp };
        let ns = self.n_spike;
        put(&mut self.spike, ns, styled(base, 9))?;
        self.n_spike += 1;
        let base = self.spike.at(self.n_spike - 1)?;
        let (d1, d2) = (calc_distance_double(a, base), calc_distance_double(b, base));
        let pt0 = if d1 > d2 {
            extend_line_double(a, base, self.spike_size / 2.0)
        } else {
            extend_line_double(b, base, -self.spike_size / 2.0)
        };
        self.add_spike_tip(j, pt0, too_long, not_vertical)?;
        self.add_second_base(j, too_long)
    }

    fn add_spike_tip(
        &mut self,
        j: usize,
        pt0: Pt,
        too_long: bool,
        not_vertical: bool,
    ) -> Result<(), EngineError> {
        let tip = SpikeTip {
            a: self.line.at(j)?,
            b: self.line.at(j + 1)?,
            pt0,
            size: self.spike_size,
            too_long,
            not_vertical,
            style: 9,
        };
        push_spike_tip(&mut self.spike, &mut self.n_spike, &tip)
    }

    fn add_second_base(&mut self, j: usize, too_long: bool) -> Result<(), EngineError> {
        let (a, b) = (self.line.at(j)?, self.line.at(j + 1)?);
        let ns = self.n_spike;
        if too_long {
            return self.finish_spike(ns, styled(b, 5));
        }
        let tip = self.spike.at(ns - 2)?;
        let (d1, d2) = (calc_distance_double(a, tip), calc_distance_double(b, tip));
        let mut p = if d1 > d2 {
            extend_line2_double(a, tip, self.spike_size, 0)
        } else {
            extend_line2_double(b, tip, -self.spike_size, 0)
        };
        match self.style.line_type {
            tl::OCCLUDED => p.style = 10,
            tl::UOF => p.style = 5,
            tl::SF => p.style = 24,
            _ => {}
        }
        self.finish_spike(ns, p)
    }

    fn finish_spike(&mut self, ns: usize, p: Pt) -> Result<(), EngineError> {
        put(&mut self.spike, ns, p)?;
        self.n_spike += 1;
        Ok(())
    }

    /// The closing padding of a segment's spike points.
    fn end_segment_spikes(&mut self, j: usize) -> Result<(), EngineError> {
        if self.n_spike == 0 {
            let (a, b) = (self.line.at(j)?, self.line.at(j + 1)?);
            for p in [a, b, b] {
                put(&mut self.spike, self.n_spike, styled(p, 5))?;
                self.n_spike += 1;
            }
        } else {
            let last = styled(self.spike.at(self.n_spike - 1)?, 5);
            for _ in 0..3 {
                put(&mut self.spike, self.n_spike, last)?;
                self.n_spike += 1;
            }
        }
        Ok(())
    }
}

/// Upstream `GetOccludedPointsDouble`: replaces `pts` with the occluded
/// front through its first `num_pts` points: for each 50-pixel stretch ten
/// flot points followed by three spike points. Returns the point count.
pub(crate) fn get_occluded_points_double(
    style: &FlotStyle,
    pts: &mut Vec<Pt>,
    num_pts: i32,
) -> Result<i32, EngineError> {
    let n = count(num_pts)?;
    let total = get_occluded_count_double(pts, num_pts)?;
    let first = styled(pts.at(0)?, 5);
    let vb = int_coords(pts, n)?;
    let mut b = Build {
        style,
        line: pts.iter().take(n).copied().collect(),
        spike_size: style.scaled(20.0),
        increment: style.scaled(50.0),
        flot: vec![first; budgeted_len(total.wrapping_mul(10) / 13, 1)?],
        spike: vec![first; budgeted_len(total.wrapping_mul(3) / 13, 1)?],
        n_flot: 0,
        n_spike: 0,
        flots: vec![0; n + 1],
        temp: Pt::default(),
    };
    let mut state = FlipState::unset();
    for j in 0..n.saturating_sub(1) {
        let (a, c) = (b.line.at(j)?, b.line.at(j + 1)?);
        let not_vertical = calc_true_slope_double(a, c).0 != 0;
        let n_flots = (calc_distance_double(a, c) / b.increment) as i32;
        *b.flots.at_mut(j + 1)? = n_flots;
        if n_flots > 0 {
            b.add_flots(j, n_flots, &vb, &mut state)?;
        }
        for k in 0..idx(n_flots.max(1) - 1, 0)? {
            b.add_spike(j, k, not_vertical)?;
        }
        b.end_segment_spikes(j)?;
    }
    load_output(pts, &b, total / 13)
}

/// Writes flots and spikes interleaved (10 and 3) into `pts`, padding the
/// tail with the last point; returns the count written.
fn load_output(pts: &mut Vec<Pt>, b: &Build<'_>, groups: i32) -> Result<i32, EngineError> {
    let groups = idx(groups, 0)?;
    let needed = groups * 13;
    if pts.len() < needed {
        pts.resize(needed, Pt::default());
    }
    let filler = b.spike.at(0)?;
    for p in pts.iter_mut() {
        *p = styled(filler, 5);
    }
    let mut out = 0;
    for g in 0..groups {
        for k in 0..10 {
            store(pts, out, b.flot.at(g * 10 + k)?)?;
            out += 1;
        }
        for k in 0..3 {
            store(pts, out, b.spike.at(g * 3 + k)?)?;
            out += 1;
        }
    }
    let last = pts.at(out.wrapping_sub(1))?;
    for p in pts.iter_mut().skip(out) {
        *p = last;
    }
    i32::try_from(out).map_err(|_| EngineError::Degenerate("flot point count overflow"))
}
