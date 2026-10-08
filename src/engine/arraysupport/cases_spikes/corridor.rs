//! The air corridor group (SC, MRR, SL, TC, LLTR, SAAFR, AC): a centre line
//! with side lines per leg and a circle at each leg end.

use crate::engine::base::{EngineError, Pt};
use crate::engine::lineutility::circle::calc_circle_double;
use crate::engine::lineutility::saafr::get_saafr_segment;

use crate::engine::arraysupport::work::{Work, set};

/// Appends the 26 points of an arc circle ending with style 5 at `w.ac`.
fn push_circle(w: &mut Work<'_>, center: Pt, radius: f64) -> Result<(), EngineError> {
    let mut circle = [Pt::default(); 26];
    calc_circle_double(center, radius, 26, &mut circle, 0)?;
    circle[25].style = 5;
    for c in circle {
        set(&mut w.p, w.ac, c)?;
        w.ac += 1;
    }
    Ok(())
}

/// The corridor's legs and the circles at its points, sized by the style
/// each control point carries.
pub(super) fn corridor(w: &mut Work<'_>) -> Result<(), EngineError> {
    w.ac = 0;
    for j in 0..w.save - 1 {
        let ju = usize::try_from(j).map_err(|_| EngineError::Degenerate("index"))?;
        let (a, b) = (
            *w.orig.get(ju).ok_or(EngineError::Degenerate("leg"))?,
            *w.orig.get(ju + 1).ok_or(EngineError::Degenerate("leg"))?,
        );
        w.d_mbr = f64::from(a.style);
        let mut leg = [
            a,
            b,
            Pt::default(),
            Pt::default(),
            Pt::default(),
            Pt::default(),
        ];
        get_saafr_segment(&mut leg, w.d_mbr)?;
        for pt in leg {
            set(&mut w.p, w.ac, pt)?;
            w.ac += 1;
        }
    }
    let mut last_size = 0;
    let mut last_point = Pt::default();
    for j in 0..w.save {
        let ju = usize::try_from(j).map_err(|_| EngineError::Degenerate("index"))?;
        let here = *w.orig.get(ju).ok_or(EngineError::Degenerate("point"))?;
        let size = here.style;
        if j == 0 {
            last_size = size;
            last_point = here;
            continue;
        }
        if size < 0 {
            continue;
        }
        w.d_mbr = f64::from(last_size);
        push_circle(w, here, w.d_mbr)?;
        push_circle(w, last_point, w.d_mbr)?;
        last_size = size;
        last_point = here;
    }
    Ok(())
}
