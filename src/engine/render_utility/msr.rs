//! Ports of `clsRenderer2.getMSRShapes` and `getAutoshapeFillShape`.

use crate::engine::base::{At, EngineError, Pt, Shape, shape_type};
use crate::engine::lineutility::basics::calc_distance_double;
use crate::engine::tactical_lines::*;
use crate::engine::tg::Tg;
use crate::engine::tg_utility::segment_colors::msr_segment_colors;
use crate::engine::tg_utility::shape_properties::get_line_stroke;

/// Upstream `getMSRShapes`: the shapes of a main/alternate supply route or
/// traffic route. Segments with a colour in modifier H are drawn as their
/// own shapes in that colour; the rest are one shape in the line colour in
/// which close points are thinned to 10 px spacing.
///
/// Upstream compares segment colours by reference and each colour entry is a
/// separate object, so a coloured segment always starts a new shape; only two
/// colourless entries in a row would reuse the previous one, and with none
/// yet it fails, ending the function early with the shapes added so far.
pub(crate) fn get_msr_shapes(tg: &Tg, shapes: &mut Vec<Shape>) {
    if !matches!(tg.line_type, MSR | ASR | TRAFFIC_ROUTE) {
        return;
    }
    build_msr(tg, shapes).ok();
}

fn build_msr(tg: &Tg, shapes: &mut Vec<Shape>) -> Result<(), EngineError> {
    let colors = msr_segment_colors(tg).unwrap_or_default();
    let stroke = get_line_stroke(tg.line_thickness, tg.line_style);
    let mut shape = Shape::new(shape_type::POLYLINE);
    shape.line_color = tg.line_color;
    shape.stroke = stroke.clone();
    let mut seg_shape: Option<Shape> = None;
    let mut last_was_none = true;
    let mut last_pt: Option<Pt> = None;
    let n = i32::try_from(tg.pixels.len()).unwrap_or(i32::MAX);
    for j in 0..n.saturating_sub(1) {
        let idx = usize::try_from(j).unwrap_or(0);
        let (pt0, pt1) = (tg.pixels.at(idx)?, tg.pixels.at(idx + 1)?);
        if let Some(color) = colors.get(&j) {
            if !(color.is_none() && last_was_none) {
                if let Some(done) = seg_shape.take() {
                    shapes.push(done);
                }
                let mut s = Shape::new(shape_type::POLYLINE);
                s.line_color = *color;
                s.style = tg.line_style;
                s.stroke = stroke.clone();
                seg_shape = Some(s);
            }
            let s = seg_shape
                .as_mut()
                .ok_or(EngineError::Degenerate("no segment shape"))?;
            s.move_to(pt0);
            s.line_to(pt1);
            last_was_none = color.is_none();
        } else if colors.contains_key(&(j + 1)) || colors.contains_key(&(j - 1)) {
            shape.move_to(pt0);
            shape.line_to(pt1);
            last_pt = Some(pt1);
        } else if j == n - 2 {
            shape.move_to(pt0);
            shape.line_to(pt1);
        } else {
            let last = *last_pt.get_or_insert_with(|| {
                shape.move_to(pt0);
                pt0
            });
            if calc_distance_double(pt0, pt1) > 10.0 || calc_distance_double(last, pt1) > 10.0 {
                shape.line_to(pt1);
                last_pt = Some(pt1);
            }
        }
    }
    if let Some(done) = seg_shape {
        shapes.push(done);
    }
    shapes.push(shape);
    Ok(())
}

/// Upstream `getAutoshapeFillShape`: for the autoshape types the fill is a
/// separate shape over the closed outline, inserted first, and the existing
/// shapes lose their fill. A failure leaves what was already changed, as
/// upstream's catch does.
pub(crate) fn get_autoshape_fill_shape(tg: &Tg, shapes: &mut Vec<Shape>) {
    if shapes.is_empty() || tg.pixels.is_empty() || tg.fill_color.is_none() {
        return;
    }
    let range = match tg.line_type {
        RETAIN => (1, 26),
        SECURE | CONTROL | LOCATE | OCCUPY => (1, tg.pixels.len().saturating_sub(3)),
        CONVOY | HCONVOY => (1, tg.pixels.len()),
        CORDONSEARCH | CORDONKNOCK | ISOLATE | DENY => (26, 47),
        _ => return,
    };
    for s in shapes.iter_mut() {
        s.fill_color = None;
    }
    autoshape_fill(tg, shapes, range).ok();
}

fn autoshape_fill(
    tg: &Tg,
    shapes: &mut Vec<Shape>,
    (from, to): (usize, usize),
) -> Result<(), EngineError> {
    let mut shape = Shape::new(shape_type::FILL);
    shape.fill_color = tg.fill_color;
    shape.line_color = None;
    let first = tg.pixels.at(0)?;
    shape.move_to(first);
    for j in from..to {
        shape.line_to(tg.pixels.at(j)?);
    }
    if matches!(tg.line_type, CORDONSEARCH | CORDONKNOCK | ISOLATE | DENY) {
        for j in [23, 24, 25] {
            shape.line_to(tg.pixels.at(j)?);
        }
    }
    shape.line_to(first);
    shapes.insert(0, shape);
    Ok(())
}
