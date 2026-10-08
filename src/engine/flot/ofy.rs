//! Port of the OFY (occluded front, yellow) builders in flot.java
//! (`GetOFYCountDouble`, `GetOFYPointsDouble`).

use super::segment::{FlipState, get_flot_segment2};
use super::spike::{SpikeTip, push_spike_tip};
use super::{FlotStyle, budgeted_len, count, int_coords, set_xy_style, store, styled};
use crate::engine::base::{At, EngineError, Pt, idx};
use crate::engine::lineutility::basics::calc_distance_double;
use crate::engine::lineutility::extend::{
    extend_along_line_double, extend_along_line_double_style, extend_directed_line_style,
    extend_line_double, extend_line2_double,
};
use crate::engine::lineutility::slope::calc_true_slope_double;

/// Upstream `GetOFYCountDouble`: the points an OFY front needs (18 per
/// flot and 7 per spike at `interval` pixels, never fewer than 25 per
/// point).
pub(crate) fn get_ofy_count_double(
    pts: &[Pt],
    interval: f64,
    num_pts: i32,
) -> Result<i32, EngineError> {
    let n = count(num_pts)?;
    let mut total = 0_i32;
    for j in 0..n.saturating_sub(1) {
        let d = calc_distance_double(pts.at(j)?, pts.at(j + 1)?);
        let segs = ((d / interval) as i32).max(1);
        total = total
            .wrapping_add(segs.wrapping_mul(18))
            .wrapping_add(segs.wrapping_mul(7));
    }
    if total < 22_i32.wrapping_mul(num_pts) {
        total = 25_i32.wrapping_mul(num_pts);
    }
    Ok(total)
}

/// A point upstream would have built before reading it.
fn built(v: &[Option<Pt>], i: usize) -> Result<Pt, EngineError> {
    v.get(i).copied().flatten().ok_or(EngineError::Degenerate(
        "flot reads a point it did not build",
    ))
}

fn put(v: &mut [Pt], i: usize, p: Pt) -> Result<(), EngineError> {
    *v.at_mut(i)? = p;
    Ok(())
}

struct Ofy<'a> {
    style: &'a FlotStyle,
    line: Vec<Pt>,
    spike_size: f64,
    flot: Vec<Pt>,
    spike: Vec<Pt>,
    seg: Vec<Pt>,
    n_flot: usize,
    n_spike: usize,
    n_seg: usize,
    flot_start: Vec<Option<Pt>>,
    flot_end: Vec<Option<Pt>>,
    spike_start: Vec<Option<Pt>>,
    spike_end: Vec<Option<Pt>>,
    n_spike_end: usize,
    n_flot_end: usize,
}

impl Ofy<'_> {
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
        self.flot_start = vec![None; n];
        self.flot_end = vec![None; n];
        let mut points = vec![0_i32; budgeted_len(n_flots, 30)?];
        get_flot_segment2(self.style, vb, j, &mut points, state)?;
        let (a, b) = (self.line.at(j)?, self.line.at(j + 1)?);
        let mut k = 0;
        for l in 0..n * 10 {
            let nf = self.n_flot;
            let (x, y) = (points.at(k)?, points.at(k + 1)?);
            set_xy_style(&mut self.flot, nf, (x, y), 9)?;
            if nf % 10 == 0 {
                *self.flot_start.at_mut(l / 10)? = Some(self.flot.at(nf)?);
                let cur = self.flot.at(nf)?;
                let (d1, d2) = (calc_distance_double(a, cur), calc_distance_double(b, cur));
                let p = if d2 > d1 {
                    extend_line_double(b, a, -d1)
                } else {
                    extend_line_double(a, b, -d2)
                };
                put(&mut self.flot, nf, styled(p, 9))?;
            }
            if (nf + 1) % 10 == 0 {
                *self.flot_end.at_mut(l / 10)? = Some(self.flot.at(nf)?);
                self.n_flot_end += 1;
                let first = self.flot.at(nf - 9)?;
                let (d1, d2) = (
                    calc_distance_double(a, first),
                    calc_distance_double(b, first),
                );
                let p = if d2 > d1 {
                    extend_line_double(b, a, -d1 - self.spike_size)
                } else {
                    extend_line_double(a, b, -d2 + self.spike_size)
                };
                put(&mut self.flot, nf, styled(p, 10))?;
            }
            k += 3;
            self.n_flot += 1;
        }
        Ok(())
    }

    fn add_spike(
        &mut self,
        j: usize,
        k: usize,
        not_vertical: bool,
        seg_len: f64,
    ) -> Result<(), EngineError> {
        let (a, b) = (self.line.at(j)?, self.line.at(j + 1)?);
        let (e0, e1) = (built(&self.flot_end, k)?, built(&self.flot_end, k + 1)?);
        let d1 = calc_distance_double(e0, e1) / 2.0 - self.spike_size;
        let temp = extend_along_line_double_style(e0, b, d1, 0);
        let too_long = calc_distance_double(a, temp) + self.spike_size >= seg_len;
        let ns = self.n_spike;
        if too_long {
            put(&mut self.spike, ns, styled(b, 9))?;
        } else {
            put(&mut self.spike, ns, styled(temp, 9))?;
            *self.spike_start.at_mut(k)? = Some(temp);
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
            style: 9,
        };
        push_spike_tip(&mut self.spike, &mut self.n_spike, &tip)?;
        let ns = self.n_spike;
        if too_long {
            put(&mut self.spike, ns, styled(b, 5))?;
        } else {
            let base = self.spike.at(ns - 2)?;
            let (d1, d2) = (calc_distance_double(a, base), calc_distance_double(b, base));
            let p = if d1 > d2 {
                extend_line2_double(a, base, self.spike_size, 0)
            } else {
                extend_line2_double(b, base, -self.spike_size, 0)
            };
            *self.spike_end.at_mut(k)? = Some(p);
            self.n_spike_end += 1;
            put(&mut self.spike, ns, styled(p, 10))?;
        }
        self.n_spike += 1;
        Ok(())
    }

    /// The line pieces joining the segment ends to the first and last
    /// flots, and the cross ticks between flots and spikes.
    fn add_segment_pieces(&mut self, j: usize) -> Result<(), EngineError> {
        let (a, b) = (self.line.at(j)?, self.line.at(j + 1)?);
        if self.n_spike_end == 0 && self.n_flot_end == 1 {
            self.push_seg(a, 0)?;
            self.push_seg(built(&self.flot_start, 0)?, 5)?;
            self.push_seg(b, 0)?;
            self.push_seg(built(&self.flot_end, 0)?, 5)?;
        }
        let five = self.style.scaled(5.0);
        for l in 0..self.n_spike_end {
            if l == 0 {
                self.push_seg(a, 0)?;
                self.push_seg(built(&self.flot_start, 0)?, 5)?;
            }
            if l == self.n_spike_end - 1 {
                self.push_seg(b, 0)?;
                self.push_seg(built(&self.flot_end, l + 1)?, 5)?;
            }
            self.push_seg(built(&self.spike_end, l)?, 0)?;
            self.push_seg(built(&self.flot_start, l + 1)?, 5)?;
            let start = built(&self.spike_start, l)?;
            let d1 = calc_distance_double(start, built(&self.flot_end, l)?);
            let third = extend_along_line_double_style(start, a, d1 / 3.0, 0);
            let two_thirds = extend_along_line_double_style(start, a, 2.0 * d1 / 3.0, 5);
            put(&mut self.seg, self.n_seg, third)?;
            put(&mut self.seg, self.n_seg + 1, two_thirds)?;
            self.n_seg += 2;
            let t1 = extend_directed_line_style(a, b, self.seg.at(self.n_seg - 2)?, 2, five, 0);
            put(&mut self.seg, self.n_seg, t1)?;
            self.n_seg += 1;
            let t2 = extend_directed_line_style(a, b, self.seg.at(self.n_seg - 2)?, 3, five, 5);
            put(&mut self.seg, self.n_seg, t2)?;
            self.n_seg += 1;
        }
        Ok(())
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

/// Upstream `GetOFYPointsDouble`: replaces `pts` with the OFY front
/// through its first `num_pts` points: all flot points, then all spike
/// points, then the connecting line pieces and cross ticks. Returns the
/// point count.
pub(crate) fn get_ofy_points_double(
    style: &FlotStyle,
    pts: &mut Vec<Pt>,
    num_pts: i32,
) -> Result<i32, EngineError> {
    let n = count(num_pts)?;
    let increment = style.scaled(80.0);
    let total = budgeted_len(get_ofy_count_double(pts, increment, num_pts)?, 1)?;
    let first = styled(pts.at(0)?, 5);
    let vb = int_coords(pts, n)?;
    let mut b = Ofy {
        style,
        line: pts.iter().take(n).copied().collect(),
        spike_size: style.scaled(20.0),
        flot: vec![first; total],
        spike: vec![first; total],
        seg: vec![Pt::default(); total],
        n_flot: 0,
        n_spike: 0,
        n_seg: 0,
        flot_start: Vec::new(),
        flot_end: Vec::new(),
        spike_start: Vec::new(),
        spike_end: Vec::new(),
        n_spike_end: 0,
        n_flot_end: 0,
    };
    let mut state = FlipState::zeroed();
    for j in 0..n.saturating_sub(1) {
        let (a, c) = (b.line.at(j)?, b.line.at(j + 1)?);
        b.n_spike_end = 0;
        b.n_flot_end = 0;
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
        b.spike_start = vec![None; spikes];
        b.spike_end = vec![None; spikes];
        for k in 0..spikes.saturating_sub(1) {
            b.add_spike(j, k, not_vertical, seg_len)?;
        }
        b.add_segment_pieces(j)?;
        b.end_segment_spikes(j)?;
    }
    load_output(pts, &b)
}

/// Concatenates flots, spikes and segment pieces into `pts` and pads the
/// tail with the last point.
fn load_output(pts: &mut Vec<Pt>, b: &Ofy<'_>) -> Result<i32, EngineError> {
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
