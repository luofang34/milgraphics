//! The second shape switch: fill shapes for arrowheads and glyphs that
//! outlines cannot carry.

use crate::engine::arraysupport::shape_path::{line_to, move_to};
use crate::engine::arraysupport::work::{Work, get};
use crate::engine::base::{EngineError, Pt, Shape, shape_type};
use crate::engine::lineutility::saafr::get_saafr_fill_segment;
use crate::engine::tactical_lines as lt;

/// Appends the fill shapes of `w.line_type`.
pub(crate) fn build(w: &Work<'_>, shapes: &mut Vec<Shape>) -> Result<(), EngineError> {
    match w.line_type {
        lt::AC | lt::SAAFR | lt::MRR | lt::SL | lt::TC | lt::SC | lt::LLTR => saafr(w, shapes),
        lt::DIRATKAIR => bow_tie_closures(w, shapes),
        lt::OFY
        | lt::OCCLUDED
        | lt::WF
        | lt::WFG
        | lt::WFY
        | lt::CF
        | lt::CFY
        | lt::CFG
        | lt::SARA
        | lt::FERRY
        | lt::EASY
        | lt::BYDIF
        | lt::BYIMP
        | lt::FOLSP
        | lt::ATDITCHC
        | lt::ATDITCHM
        | lt::MNFLDFIX
        | lt::TURN_REVD
        | lt::TURN
        | lt::MNFLDDIS
        | lt::AREA_DEFENSE
        | lt::MOBILE_DEFENSE => style9_fills(w, shapes),
        _ => Ok(()),
    }
}

/// The arrowhead quads of the segmented corridors go first in the list.
fn saafr(w: &Work<'_>, shapes: &mut Vec<Shape>) -> Result<(), EngineError> {
    for j in 0..w.save - 1 {
        let (a, b) = (get(&w.orig, j)?, get(&w.orig, j + 1)?);
        let mut ac_points = [Pt::default(); 6];
        ac_points[0] = a;
        ac_points[1] = b;
        get_saafr_fill_segment(&mut ac_points, f64::from(a.style))?;
        let mut s = Shape::new(shape_type::FILL);
        move_to(&mut s, ac_points[0]);
        for q in &ac_points[1..4] {
            line_to(&mut s, *q)?;
        }
        shapes.insert(0, s);
    }
    Ok(())
}

/// A line closing each bow tie, which is not filled.
fn bow_tie_closures(w: &Work<'_>, shapes: &mut Vec<Shape>) -> Result<(), EngineError> {
    for k in 0..w.ac {
        let pk = get(&w.p, k)?;
        if pk.style == 10 {
            let mut s = Shape::new(shape_type::POLYLINE);
            move_to(&mut s, get(&w.p, k - 2)?);
            line_to(&mut s, pk)?;
            shapes.push(s);
        }
    }
    Ok(())
}

/// Runs of style 9 points ending in a style 10 point become fill shapes.
fn style9_fills(w: &Work<'_>, shapes: &mut Vec<Shape>) -> Result<(), EngineError> {
    let mut shape: Option<Shape> = None;
    for k in 0..w.ac {
        let pk = get(&w.p, k)?;
        let prev_is_9 = k > 0 && get(&w.p, k - 1)?.style == 9;
        if pk.style == 9 && !prev_is_9 {
            let mut s = Shape::new(shape_type::FILL);
            s.style = pk.style;
            move_to(&mut s, pk);
            shape = Some(s);
        } else if pk.style == 9 && k > 0 && prev_is_9 {
            let s = shape
                .as_mut()
                .ok_or(EngineError::Degenerate("fill run without a start"))?;
            line_to(s, pk)?;
        }
        if pk.style == 10 {
            let mut s = shape
                .clone()
                .ok_or(EngineError::Degenerate("fill run without a start"))?;
            line_to(&mut s, pk)?;
            shape = Some(s.clone());
            if w.line_type == lt::AREA_DEFENSE {
                shapes.push(s);
            } else {
                shapes.insert(0, s);
            }
        }
    }
    Ok(())
}
