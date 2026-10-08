//! The mid-segment features of the warm-front variants that
//! `GetFlot2Double` collects in `style10Points`: the line pieces between
//! flots of WF, and the dots (WFG) and cross ticks (WFY).

use super::{FlotStyle, styled};
use crate::engine::base::{At, EngineError, Pt};
use crate::engine::lineutility::extend::{
    extend_along_line_double_style, extend_directed_line_style,
};
use crate::engine::tactical_lines as tl;

/// Upstream's `style10Points` array and its counter, sized by the flot
/// count; writing past the end fails as Java's array would.
#[derive(Debug)]
pub(crate) struct Style10 {
    pts: Vec<Pt>,
    len: usize,
}

impl Style10 {
    pub(crate) fn new(capacity: usize) -> Self {
        Self {
            pts: vec![Pt::default(); capacity],
            len: 0,
        }
    }

    pub(crate) fn push(&mut self, p: Pt) -> Result<(), EngineError> {
        *self.pts.at_mut(self.len)? = p;
        self.len += 1;
        Ok(())
    }

    /// The most recently pushed point.
    pub(crate) fn last(&self) -> Result<Pt, EngineError> {
        self.pts.at(self.len.wrapping_sub(1))
    }

    pub(crate) fn len(&self) -> usize {
        self.len
    }

    pub(crate) fn get(&self, i: usize) -> Result<Pt, EngineError> {
        self.pts.at(i)
    }
}

/// The `pt1`/`pt2` scratch points `GetFlot2Double` threads through its
/// loop; they keep their last values between flots.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Scratch {
    pub(crate) pt1: Pt,
    pub(crate) pt2: Pt,
}

/// The features after a flot that is neither the last nor the second to
/// last of its segment: dots for WFG, double-break with ticks for WFY.
pub(crate) fn mid_features(
    style: &FlotStyle,
    s10: &mut Style10,
    scratch: &mut Scratch,
) -> Result<(), EngineError> {
    scratch.pt2 = s10.last()?;
    let (pt2, pt1) = (scratch.pt2, scratch.pt1);
    let along =
        |size: f64, st: i32| extend_along_line_double_style(pt2, pt1, style.scaled(size), st);
    match style.line_type {
        tl::WFG => {
            s10.push(along(10.0, 5))?;
            s10.push(along(20.0, 20))?;
            s10.push(along(30.0, 0))?;
            s10.push(along(70.0, 5))?;
        }
        tl::WFY => {
            s10.push(along(10.0, 5))?;
            s10.push(along(15.0, 0))?;
            let base = s10.last()?;
            let cross1 = extend_directed_line_style(base, pt1, base, 3, style.scaled(5.0), 0);
            s10.push(along(25.0, 5))?;
            let base = s10.last()?;
            let cross2 = extend_directed_line_style(base, pt1, base, 2, style.scaled(5.0), 5);
            s10.push(cross1)?;
            s10.push(cross2)?;
            s10.push(along(30.0, 0))?;
            s10.push(along(60.0, 5))?;
        }
        _ => {}
    }
    Ok(())
}

/// The features closing a segment's last flot: a stub from the segment
/// start and the flot's own point plus the segment end.
pub(crate) fn segment_ends(
    style: &FlotStyle,
    s10: &mut Style10,
    scratch: &mut Scratch,
    ends: ([i32; 2], [i32; 2]),
    flot_point: Pt,
) -> Result<(), EngineError> {
    let (start, end) = ends;
    scratch.pt2.x = f64::from(start[0]);
    scratch.pt2.y = f64::from(start[1]);
    scratch.pt2.style = 0;
    s10.push(scratch.pt2)?;
    s10.push(extend_along_line_double_style(
        scratch.pt2,
        scratch.pt1,
        style.scaled(40.0),
        5,
    ))?;
    scratch.pt2.x = f64::from(end[0]);
    scratch.pt2.y = f64::from(end[1]);
    scratch.pt2.style = 5;
    s10.push(styled(flot_point, 0))?;
    s10.push(scratch.pt2)
}
