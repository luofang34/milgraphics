//! Port of `GetDitchSpikeDouble` from lineutility.java: the spikes (and, for
//! mined ditches, circles) drawn along an anti-tank ditch.

use super::basics::{calc_distance_double, mid_point_double};
use super::extend::extend_line_double;
use super::slope::{calc_true_intersect_double2, calc_true_lines_double, calc_true_slope_double};
use crate::engine::base::{At, EngineError, Pt, idx};
use crate::engine::tactical_lines as lt;

/// Truncates both operands to `i32`, adds them as Java `int`s and widens.
fn trunc_add(a: f64, b: f64) -> f64 {
    f64::from((a as i32).wrapping_add(b as i32))
}

/// Truncates both operands to `i32`, subtracts them as Java `int`s and widens.
fn trunc_sub(a: f64, b: f64) -> f64 {
    f64::from((a as i32).wrapping_sub(b as i32))
}

/// Working state that upstream keeps in locals across segments and spikes.
#[derive(Debug)]
struct Ditch {
    linetype: i32,
    spike_length: f64,
    upper: Pt,
    lower1: Pt,
    lower2: Pt,
    average: Pt,
    last_average: Pt,
    loc1: (f64, f64),
    loc2: (f64, f64),
    side: f64,
    base_points: Vec<Pt>,
    circle_points: Vec<Pt>,
}

/// Geometry of the segment currently being decorated.
#[derive(Clone, Copy, Debug)]
struct Seg {
    a: Pt,
    b: Pt,
    m: f64,
    vertical: i32,
    length: f64,
    count: i32,
}

impl Ditch {
    fn is_mined(&self) -> bool {
        self.linetype == lt::ATDITCHC || self.linetype == lt::ATDITCHM
    }

    /// Where the spike's tip goes for spike `j` of the segment.
    fn place_apex(&mut self, first: Pt, s: Seg) {
        if s.m != 0.0 && s.vertical != 0 {
            let bint =
                (self.loc1.1 + self.loc2.1) / 2.0 + (1.0 / s.m) * (self.loc1.0 + self.loc2.0) / 2.0;
            self.upper = calc_true_intersect_double2(
                s.m,
                self.side,
                -1.0 / s.m,
                bint,
                1,
                1,
                (first.x, first.y),
            );
        }
        let step = (s.length / f64::from(s.count)) as i32;
        let half = (s.length / f64::from(s.count) / 2.0) as i32;
        let (step, half) = (f64::from(step), f64::from(half));
        if s.vertical == 0 {
            self.upper.y = if self.loc1.1 < self.loc2.1 {
                trunc_add(self.loc1.1, half)
            } else {
                trunc_sub(self.loc1.1, half)
            };
            self.upper.x = if s.a.y < s.b.y {
                trunc_add(self.loc1.0, step)
            } else {
                trunc_sub(self.loc1.0, step)
            };
        }
        if s.m == 0.0 && s.vertical != 0 {
            self.upper.x = if self.loc1.0 < self.loc2.0 {
                trunc_add(self.loc1.0, half)
            } else {
                trunc_sub(self.loc1.0, half)
            };
            self.upper.y = if s.b.x < s.a.x {
                trunc_add(self.loc1.1, step)
            } else {
                trunc_sub(self.loc1.1, step)
            };
        }
    }

    /// Appends the three spike points (base, tip, base) with their styles.
    fn emit_spike(
        &self,
        out: &mut Vec<Pt>,
        n_spike: &mut usize,
        j: i32,
    ) -> Result<(), EngineError> {
        let alternate = j % 2 == 1 && self.linetype == lt::ATDITCHM;
        for (p, mined_style) in [(self.lower1, 9), (self.upper, 9), (self.lower2, 10)] {
            let mut q = p;
            if self.is_mined() {
                q.style = mined_style;
            }
            if alternate {
                q.style = 5;
            }
            *slot(out, *n_spike)? = q;
            *n_spike += 1;
        }
        Ok(())
    }

    /// Centre of the mine circle belonging to spike `j` of a mined ditch.
    fn update_average(&mut self, j: i32) {
        if self.linetype != lt::ATDITCHM {
            return;
        }
        if j % 2 == 0 {
            self.average = mid_point_double(self.lower1, self.lower2, 0);
            self.average = mid_point_double(self.average, self.upper, 0);
        } else if j == 1 {
            self.average = extend_line_double(self.lower2, self.lower1, 5.0);
            self.average = mid_point_double(self.average, self.upper, 0);
        }
    }

    /// Records the base line point for spike `j` and the circle centres.
    fn finish_spike(&mut self, j: i32, s: Seg) {
        if j > 1 && j < s.count {
            self.base_points.push(self.lower1);
        } else if j == 1 {
            self.base_points.push(s.a);
        } else if j == s.count {
            let mut end = s.b;
            end.style = 5;
            self.base_points.push(end);
        }
        if self.linetype == lt::ATDITCHM && j > 1 && j % 2 == 0 {
            self.circle_points
                .push(mid_point_double(self.average, self.last_average, 20));
        }
        if j < s.count && self.linetype == lt::ATDITCHM && (j == 1 || j % 2 == 0) {
            self.last_average = self.average;
        }
    }

    /// Lays all spikes along one segment that is long enough to carry them.
    fn spikes_along(
        &mut self,
        out: &mut Vec<Pt>,
        n_spike: &mut usize,
        first: Pt,
        s: Seg,
    ) -> Result<(), EngineError> {
        for j in 1..=s.count {
            let k = f64::from(j);
            let sl = self.spike_length;
            let along = |off: f64| (k * sl + off) / s.length * (s.b.x - s.a.x);
            let along_y = |off: f64| (k * sl + off) / s.length * (s.b.y - s.a.y);
            if j > 1 {
                self.loc1 = self.loc2;
            } else {
                self.loc1 = (s.a.x + along(-sl / 2.0), s.a.y + along_y(-sl / 2.0));
            }
            self.loc2 = (s.a.x + along(sl / 2.0), s.a.y + along_y(sl / 2.0));
            self.place_apex(first, s);
            self.lower1.x = self.loc1.0;
            self.lower1.y = self.loc1.1;
            self.lower2.x = self.loc2.0;
            self.lower2.y = self.loc2.1;
            self.emit_spike(out, n_spike, j)?;
            self.update_average(j);
            self.finish_spike(j, s);
        }
        Ok(())
    }
}

/// The element at `i`, growing the vector with default points as needed
/// (upstream's arrays arrive pre-sized for the worst case).
fn slot(v: &mut Vec<Pt>, i: usize) -> Result<&mut Pt, EngineError> {
    if v.len() <= i {
        v.resize(i + 1, Pt::default());
    }
    v.at_mut(i)
}

/// Upstream `GetDitchSpikeDouble`: decorates the `n_old_counter` line points
/// in `pts` with spikes along every segment, then replaces the line points
/// with the reversed original line and appends the spike base line and (for
/// mined ditches) the circle centres. Returns the number of points used.
///
/// `spike_length` and `radius` are upstream's `getScaledSize(12, ...)` and
/// `getScaledSize(4, ...)` for the graphic's line thickness and pattern scale;
/// `b_way_is` chooses the side of the line the spikes point to.
pub(crate) fn get_ditch_spike_double(
    linetype: i32,
    spike_length: f64,
    radius: f64,
    pts: &mut Vec<Pt>,
    n_old_counter: i32,
    b_way_is: i32,
) -> Result<i32, EngineError> {
    let first = pts.at(0)?;
    let n_old = idx(n_old_counter, pts.len())?;
    let temp: Vec<Pt> = pts.iter().take(n_old).copied().collect();
    if temp.len() != n_old {
        return Err(EngineError::Index {
            index: i64::from(n_old_counter),
            len: pts.len(),
        });
    }
    let mut d = Ditch {
        linetype,
        spike_length,
        upper: first,
        lower1: first,
        lower2: first,
        average: Pt::default(),
        last_average: Pt::default(),
        loc1: (0.0, 0.0),
        loc2: (0.0, 0.0),
        side: 0.0,
        base_points: Vec::new(),
        circle_points: Vec::new(),
    };
    let spike_height = spike_length * 1.25;
    let mut min_length = 2.0 * spike_length;
    let mut n_spike = n_old;
    for i in 0..n_old.saturating_sub(1) {
        if linetype == lt::ATDITCHM && i == 0 {
            min_length = spike_length * 2.5 + radius * 2.0;
        }
        let (a, b) = (pts.at(i)?, pts.at(i + 1)?);
        let (_, lines) = calc_true_lines_double(spike_height as i64, a, b);
        let (r, s) = (lines[3], lines[5]);
        let length = calc_distance_double(a, b);
        let (vertical, m) = calc_true_slope_double(a, b);
        let count = ((length - 1.0) / spike_length) as i32;
        if length > min_length {
            if b_way_is != 0 {
                if a.x <= b.x {
                    d.side = r;
                }
                if a.x >= b.x {
                    d.side = s;
                }
            } else {
                if a.x <= b.x {
                    d.side = s;
                }
                if a.x >= b.x {
                    d.side = r;
                }
            }
            let seg = Seg {
                a,
                b,
                m,
                vertical,
                length,
                count,
            };
            d.spikes_along(pts, &mut n_spike, first, seg)?;
        } else {
            let p = slot(pts, n_spike)?;
            p.x = a.x;
            p.y = a.y;
            p.style = 0;
            n_spike += 1;
            let p = slot(pts, n_spike)?;
            p.x = b.x;
            p.y = b.y;
            p.style = 5;
            n_spike += 1;
        }
    }
    finish_ditch(pts, &temp, &mut d, n_spike)
}

/// Writes the reversed original line, the base line and the circle centres
/// after the spikes; returns the final point count.
fn finish_ditch(
    pts: &mut Vec<Pt>,
    temp: &[Pt],
    d: &mut Ditch,
    spike_count: usize,
) -> Result<i32, EngineError> {
    let mut n_spike = spike_count;
    for (j, orig) in temp.iter().rev().enumerate() {
        let p = slot(pts, j)?;
        *p = *orig;
        p.style = 5;
    }
    let last = n_spike
        .checked_sub(1)
        .ok_or(EngineError::Degenerate("ditch has no points"))?;
    let tail = slot(pts, last)?;
    if tail.style == 0 {
        tail.style = 5;
    }
    for bp in &d.base_points {
        let p = slot(pts, n_spike)?;
        *p = *bp;
        if p.style != 5 {
            p.style = 0;
        }
        n_spike += 1;
    }
    if d.linetype == lt::ATDITCHM {
        let last = n_spike
            .checked_sub(1)
            .ok_or(EngineError::Degenerate("ditch has no points"))?;
        slot(pts, last)?.style = 5;
        for c in &d.circle_points {
            let p = slot(pts, n_spike)?;
            *p = *c;
            p.style = 20;
            n_spike += 1;
        }
    }
    i32::try_from(n_spike).map_err(|_| EngineError::Degenerate("ditch point count overflows"))
}
