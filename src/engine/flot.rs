//! Port of mil-sym-java JavaLineArray/flot.java: the point builders for
//! the flotation-style fronts and lines (FLOT, LC, warm/cold/occluded/
//! stationary fronts, anchorage flots).
//!
//! Upstream builders write into a caller-sized `POINT2[]` and return a
//! point count. Here the caller owns a `Vec<Pt>` that grows when a builder
//! writes past its end; builders return the count. Where Java would throw
//! (and log, leaving the graphic partly built) the builders return
//! [`EngineError`]. The `TGLight` inputs are carried by [`FlotStyle`].
//! `ref<int[]>` flip/direction state is [`segment::FlipState`].

pub(crate) mod anchorage;
pub(crate) mod angle;
pub(crate) mod flot_line;
pub(crate) mod flot_wf;
pub(crate) mod occluded;
pub(crate) mod ofy;
pub(crate) mod segment;
pub(crate) mod sf;
mod sf_features;
mod spike;
mod wf_style10;

#[cfg(test)]
mod tests;

use crate::engine::base::{At, EngineError, Pt, idx};

/// The `TGLight` values flot reads: the line type and the thickness and
/// pattern scale that scale every pattern dimension.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct FlotStyle {
    pub(crate) line_type: i32,
    pub(crate) line_thickness: f64,
    pub(crate) pattern_scale: f64,
}

impl FlotStyle {
    pub(crate) fn new(line_type: i32, line_thickness: f64, pattern_scale: f64) -> Self {
        Self {
            line_type,
            line_thickness,
            pattern_scale,
        }
    }

    /// `arraysupport.getScaledSize` with this style's thickness and scale.
    pub(crate) fn scaled(&self, size: f64) -> f64 {
        get_scaled_size(size, self.line_thickness, self.pattern_scale)
    }
}

/// `arraysupport.getScaledSize`: a pattern dimension grown with line
/// widths above the default of 3 pixels (capped at 100).
pub(crate) fn get_scaled_size(original_size: f64, line_width: f64, pattern_scale: f64) -> f64 {
    if line_width <= 3.0 {
        return original_size;
    }
    let width = if line_width > 100.0 {
        100.0
    } else {
        line_width
    };
    original_size * (1.0 + ((width - 3.0) / 2.0) * pattern_scale)
}

/// The integer pixel coordinates upstream keeps in `vbPoints` (x then y per
/// point, truncated toward zero).
pub(crate) fn int_coords(pts: &[Pt], num_pts: usize) -> Result<Vec<i32>, EngineError> {
    let mut out = Vec::with_capacity(num_pts.saturating_mul(2));
    for i in 0..num_pts {
        let p = pts.at(i)?;
        out.push(p.x as i32);
        out.push(p.y as i32);
    }
    Ok(out)
}

/// A Java `int` count as an index bound, failing when negative.
pub(crate) fn count(n: i32) -> Result<usize, EngineError> {
    idx(n, 0)
}

/// The most points any builder may grow an output array to.
pub(crate) const MAX_OUTPUT_POINTS: usize = 1 << 22;

/// The element at `i` of a caller-owned output array, growing the array
/// when the builder writes past its end.
pub(crate) fn slot(v: &mut Vec<Pt>, i: usize) -> Result<&mut Pt, EngineError> {
    if i >= MAX_OUTPUT_POINTS {
        return Err(EngineError::Index {
            index: i64::try_from(i).unwrap_or(i64::MAX),
            len: v.len(),
        });
    }
    if i >= v.len() {
        v.resize(i.saturating_add(1), Pt::default());
    }
    v.at_mut(i)
}

/// A point stored by value into a caller-owned output array.
pub(crate) fn store(v: &mut Vec<Pt>, i: usize, p: Pt) -> Result<(), EngineError> {
    *slot(v, i)? = p;
    Ok(())
}

/// A point copied with a different style, as upstream's
/// `new POINT2(p)` followed by `.style = s`.
pub(crate) fn styled(p: Pt, style: i32) -> Pt {
    Pt { style, ..p }
}

/// `n * per` as an allocation length, failing when `n` is negative or the
/// product exceeds the output budget.
pub(crate) fn budgeted_len(n: i32, per: usize) -> Result<usize, EngineError> {
    let n = idx(n, 0)?;
    n.checked_mul(per)
        .filter(|len| *len <= MAX_OUTPUT_POINTS.saturating_mul(30))
        .ok_or(EngineError::Degenerate(
            "flot output exceeds its point budget",
        ))
}

/// Sets position and style of element `i` in place, keeping its other
/// fields, as upstream assigns `.x`, `.y` and `.style` on an existing point.
pub(crate) fn set_xy_style(
    v: &mut [Pt],
    i: usize,
    xy: (i32, i32),
    style: i32,
) -> Result<(), EngineError> {
    let p = v.at_mut(i)?;
    p.x = f64::from(xy.0);
    p.y = f64::from(xy.1);
    p.style = style;
    Ok(())
}
