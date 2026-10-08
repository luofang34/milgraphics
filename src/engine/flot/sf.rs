//! Port of the stationary-front builders in flot.java
//! (`GetSFCountDouble`, `GetSFPointsDouble`) for SF, USF, SFG and SFY.

use super::segment::{FlipState, get_flot_segment2};
use super::spike::{SpikeTip, push_spike_tip};
use super::{FlotStyle, budgeted_len, count, int_coords, set_xy_style, store, styled};
use crate::engine::base::{At, EngineError, Pt, idx};
use crate::engine::lineutility::basics::calc_distance_double;
use crate::engine::lineutility::extend::{
    extend_along_line_double, extend_along_line_double_style, extend_line_double,
};
use crate::engine::lineutility::slope::calc_true_slope_double;
use crate::engine::tactical_lines as tl;

/// Upstream `GetSFCountDouble`: the points a stationary front over the
/// first `num_pts` points needs (80-pixel pitch; never fewer than 25 per
/// point).
pub(crate) fn get_sf_count_double(pts: &[Pt], num_pts: i32) -> Result<i32, EngineError> {
    let n = count(num_pts)?;
    let mut total = 0_i32;
    for j in 0..n.saturating_sub(1) {
        let d = calc_distance_double(pts.at(j)?, pts.at(j + 1)?);
        let segs = (d / 80.0) as i32;
        let per = segs.max(1);
        total = total
            .wrapping_add(per.wrapping_mul(10))
            .wrapping_add(per.wrapping_mul(3))
            .wrapping_add(segs.wrapping_mul(16))
            .wrapping_add(num_pts.wrapping_mul(4));
    }
    Ok(total.max(num_pts.wrapping_mul(25)).max(num_pts))
}

/// The working arrays of `GetSFPointsDouble`.
#[derive(Debug)]
pub(crate) struct Sf<'a> {
    pub(super) style: &'a FlotStyle,
    pub(super) line: Vec<Pt>,
    pub(super) spike_size: f64,
    flot: Vec<Pt>,
    pub(super) spike: Vec<Pt>,
    seg: Vec<Pt>,
    n_flot: usize,
    pub(super) n_spike: usize,
    n_seg: usize,
    pub(super) flot_start: Vec<Pt>,
    pub(super) flot_end: Vec<Pt>,
    pub(super) spike_start: Vec<Pt>,
    pub(super) spike_end: Vec<Pt>,
}

fn put(v: &mut [Pt], i: usize, p: Pt) -> Result<(), EngineError> {
    *v.at_mut(i)? = p;
    Ok(())
}

impl Sf<'_> {
    fn is_sf_kind(&self) -> bool {
        matches!(self.style.line_type, tl::SF | tl::USF)
    }

    fn push_seg(&mut self, p: Pt, style: i32) -> Result<(), EngineError> {
        put(&mut self.seg, self.n_seg, styled(p, style))?;
        self.n_seg += 1;
        Ok(())
    }

    fn add_flots(
        &mut self,
        j: usize,
        n_flots: i32,
        vb: &[i32],
        state: &mut FlipState,
    ) -> Result<(), EngineError> {
        let n = idx(n_flots, 0)?;
        self.flot_start = vec![Pt::default(); n];
        self.flot_end = vec![Pt::default(); n];
        let mut points = vec![0_i32; budgeted_len(n_flots, 30)?];
        get_flot_segment2(self.style, vb, j, &mut points, state)?;
        let (a, b) = (self.line.at(j)?, self.line.at(j + 1)?);
        let body = if self.is_sf_kind() { 19 } else { 9 };
        let end = if self.is_sf_kind() { 5 } else { 23 };
        let mut k = 0;
        for l in 0..n * 10 {
            let nf = self.n_flot;
            let (x, y) = (points.at(k)?, points.at(k + 1)?);
            set_xy_style(&mut self.flot, nf, (x, y), body)?;
            if nf % 10 == 0 {
                let cur = self.flot.at(nf)?;
                *self.flot_start.at_mut(l / 10)? = cur;
                let (d1, d2) = (calc_distance_double(a, cur), calc_distance_double(b, cur));
                let p = if d2 > d1 {
                    extend_line_double(b, a, -d1)
                } else {
                    extend_line_double(a, b, -d2)
                };
                put(&mut self.flot, nf, styled(p, body))?;
            }
            if (nf + 1) % 10 == 0 {
                self.flot.at_mut(nf)?.style = end;
                *self.flot_end.at_mut(l / 10)? = self.flot.at(nf)?;
            }
            if l == 0 {
                self.push_seg(a, 19)?;
                self.push_seg(self.flot_start.at(l)?, 5)?;
            }
            if l == n * 10 - 1 {
                self.push_seg(b, 19)?;
                self.push_seg(self.flot_start.at(l / 10)?, 5)?;
            }
            k += 3;
            self.n_flot += 1;
        }
        Ok(())
    }

    /// One spike after flot `k` of segment `j`: base, tip and second base,
    /// plus the line type's features. Returns nothing; the spike arrays
    /// advance.
    fn add_spike(
        &mut self,
        j: usize,
        k: usize,
        not_vertical: bool,
        seg_len: f64,
        spikes: usize,
    ) -> Result<(), EngineError> {
        let (a, b) = (self.line.at(j)?, self.line.at(j + 1)?);
        let (f0, f1) = (self.flot_start.at(k)?, self.flot_start.at(k + 1)?);
        let d1 = calc_distance_double(f0, f1) / 2.0 - self.spike_size;
        let temp = extend_along_line_double_style(f0, b, d1, 0);
        let too_long = calc_distance_double(a, temp) + self.spike_size >= seg_len;
        let body = if self.is_sf_kind() { 25 } else { 9 };
        let ns = self.n_spike;
        if too_long {
            put(&mut self.spike, ns, styled(b, body))?;
        } else {
            put(&mut self.spike, ns, styled(temp, body))?;
            put(&mut self.spike_start, k, temp)?;
        }
        self.n_spike += 1;
        let pt0 =
            extend_along_line_double(self.spike.at(self.n_spike - 1)?, b, self.spike_size / 2.0);
        let tip = SpikeTip {
            a,
            b,
            pt0,
            size: self.spike_size,
            too_long,
            not_vertical,
            style: body,
        };
        push_spike_tip(&mut self.spike, &mut self.n_spike, &tip)?;
        let ns = self.n_spike;
        if too_long {
            put(&mut self.spike, ns, styled(b, 5))?;
        } else {
            let base = self.spike.at(ns - 2)?;
            let st = if self.is_sf_kind() { 5 } else { 24 };
            let p = styled(extend_along_line_double(base, b, self.spike_size), st);
            put(&mut self.spike, ns, p)?;
            put(&mut self.spike_end, k, p)?;
        }
        self.n_spike += 1;
        self.add_features(j, k, k + 2 == spikes)
    }

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

/// Upstream `GetSFPointsDouble`: replaces `pts` with the stationary front
/// through its first `num_pts` points: flot points, then spike points and
/// their features, then the segment pieces. Returns the point count.
pub(crate) fn get_sf_points_double(
    style: &FlotStyle,
    pts: &mut Vec<Pt>,
    num_pts: i32,
) -> Result<i32, EngineError> {
    let n = count(num_pts)?;
    let total = budgeted_len(get_sf_count_double(pts, num_pts)?, 1)?;
    let first = styled(pts.at(0)?, 5);
    let vb = int_coords(pts, n)?;
    let increment = style.scaled(80.0);
    let mut b = Sf {
        style,
        line: pts.iter().take(n).copied().collect(),
        spike_size: style.scaled(20.0),
        flot: vec![first; total],
        spike: vec![first; total],
        seg: vec![Pt::default(); n.saturating_sub(1) * 4],
        n_flot: 0,
        n_spike: 0,
        n_seg: 0,
        flot_start: Vec::new(),
        flot_end: Vec::new(),
        spike_start: Vec::new(),
        spike_end: Vec::new(),
    };
    let mut state = FlipState::unset();
    for j in 0..n.saturating_sub(1) {
        let (a, c) = (b.line.at(j)?, b.line.at(j + 1)?);
        let not_vertical = calc_true_slope_double(a, c).0 != 0;
        let seg_len = calc_distance_double(a, c);
        let n_flots = (seg_len / increment) as i32;
        if n_flots > 0 {
            b.add_flots(j, n_flots, &vb, &mut state)?;
        } else {
            b.push_seg(a, 0)?;
            b.push_seg(c, 5)?;
        }
        let spikes = idx(n_flots.max(0), 0)?;
        b.spike_start = vec![Pt::default(); spikes];
        b.spike_end = vec![Pt::default(); spikes];
        for k in 0..spikes.saturating_sub(1) {
            b.add_spike(j, k, not_vertical, seg_len, spikes)?;
        }
        b.end_segment_spikes(j)?;
    }
    load_output(pts, &b)
}

/// Fills `pts` with the first spike point, then the flots, spikes and
/// segment pieces, and pads the tail with the last point.
fn load_output(pts: &mut Vec<Pt>, b: &Sf<'_>) -> Result<i32, EngineError> {
    let filler = styled(b.spike.at(0)?, 5);
    for p in pts.iter_mut() {
        *p = filler;
    }
    let mut out = 0;
    for p in b
        .flot
        .iter()
        .take(b.n_flot)
        .chain(b.spike.iter().take(b.n_spike))
        .chain(b.seg.iter().take(b.n_seg))
    {
        store(pts, out, *p)?;
        out += 1;
    }
    let last = pts.at(out.wrapping_sub(1))?;
    for p in pts.iter_mut().skip(out) {
        *p = last;
    }
    i32::try_from(out).map_err(|_| EngineError::Degenerate("flot point count overflow"))
}
