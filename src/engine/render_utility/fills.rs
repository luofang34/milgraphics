//! Ports of the unclipped parts of `clsClipPolygon2` (`LinesWithFill`,
//! `fillDMA`, `closeAreaTG`, `addAbatisFill`) and of
//! `clsRenderer.resolvePostClippedShapes`.

use crate::engine::base::{At, EngineError, Pt, Shape, shape_type};
use crate::engine::tactical_lines::*;
use crate::engine::tg::Tg;
use crate::engine::tg_utility::line_classes::lines_with_fill;

/// Upstream `closeAreaTG`: repeats the first point at the end unless the
/// ring is already closed.
fn close_area(pixels: &mut Vec<Pt>) {
    if let (Some(first), Some(last)) = (pixels.first().copied(), pixels.last().copied()) {
        if first.x != last.x || first.y != last.y {
            pixels.push(first);
        }
    }
}

/// One fill shape over the closed outline of `pixels`; `None` for no points.
fn closed_fill(tg: &Tg, pixels: &[Pt]) -> Option<Shape> {
    let mut ring = pixels.to_vec();
    close_area(&mut ring);
    let (first, rest) = ring.split_first()?;
    let mut shape = Shape::new(shape_type::FILL);
    shape.fill_color = tg.fill_color;
    shape.move_to(*first);
    for p in rest {
        shape.line_to(*p);
    }
    Some(shape)
}

/// Upstream `fillDMA` (without clipping): the fill of the area types whose
/// outline carries a feint, as a separate shape.
pub(crate) fn fill_dma(tg: &Tg) -> Vec<Shape> {
    if !matches!(
        tg.line_type,
        OBSFAREA | OBSAREA | STRONG | ZONE | FORT_REVD | FORT | ENCIRCLE | ATDITCHC | ATDITCHM
    ) {
        return Vec::new();
    }
    closed_fill(tg, &tg.pixels).into_iter().collect()
}

/// Upstream `LinesWithFill` (without clipping): for lines with glyphs the
/// fill is its own shape over the control points, drawn before the line.
/// `None` stands for upstream's null (no separate fill). Call it with the
/// graphic's control points in `tg.pixels`, before they are modified.
pub(crate) fn lines_with_fill_shapes(tg: &Tg) -> Option<Vec<Shape>> {
    let fill = tg.fill_color?;
    if fill.a <= 1 || tg.pixels.is_empty() {
        return None;
    }
    match tg.line_type {
        ABATIS | SPT | FRONTAL_ATTACK | TURNING_MOVEMENT | MOVEMENT_TO_CONTACT | MAIN | AAAAA
        | AIRAOA | CATK | CATKBYFIRE | CORDONSEARCH | CORDONKNOCK | DENY | SECURE | CONTROL
        | LOCATE | OCCUPY | RETAIN | ISOLATE | AREA_DEFENSE | MOBILE_DEFENSE | CONVOY | HCONVOY => {
            return None;
        }
        PAA_RECTANGULAR | RECTANGULAR_TARGET => return None,
        OBSFAREA | OBSAREA | STRONG | ZONE | FORT_REVD | FORT | ENCIRCLE | ATDITCHC | ATDITCHM => {
            return Some(fill_dma(tg));
        }
        _ => {}
    }
    if !lines_with_fill(tg.line_type) {
        return None;
    }
    closed_fill(tg, &tg.pixels).map(|s| vec![s])
}

/// Upstream `addAbatisFill`: the fill of `MSDZ` and `ABATIS`, built from the
/// processed points and inserted first. For `MSDZ` with fewer than 300 points
/// the inserted shape is empty, as upstream's is.
pub(crate) fn add_abatis_fill(tg: &Tg, shapes: &mut Vec<Shape>) {
    let Some(fill) = tg.fill_color else {
        return;
    };
    if tg.pixels.len() < 2 || fill.a < 2 {
        return;
    }
    let shape = match tg.line_type {
        MSDZ => msdz_fill(tg),
        ABATIS => abatis_fill(tg),
        _ => return,
    };
    if let Ok(shape) = shape {
        shapes.insert(0, shape);
    }
}

fn msdz_fill(tg: &Tg) -> Result<Shape, EngineError> {
    let mut shape = Shape::new(shape_type::POLYLINE);
    shape.fill_color = tg.fill_color;
    if tg.pixels.len() >= 300 {
        let span = |a: usize, b: usize| -> Result<f64, EngineError> {
            Ok((tg.pixels.at(a)?.x - tg.pixels.at(b)?.x).abs())
        };
        let (d0, d1, d2) = (span(0, 50)?, span(100, 150)?, span(200, 250)?);
        let start = if d0 >= d1 && d0 >= d2 {
            0
        } else if d1 >= d0 && d1 >= d2 {
            100
        } else {
            200
        };
        shape.move_to(tg.pixels.at(start)?);
        for j in start..=start + 99 {
            shape.line_to(tg.pixels.at(j)?);
        }
    }
    Ok(shape)
}

fn abatis_fill(tg: &Tg) -> Result<Shape, EngineError> {
    let mut shape = Shape::new(shape_type::POLYLINE);
    shape.fill_color = tg.fill_color;
    let n = tg.pixels.len();
    if n > 2 {
        let (a, b, c) = (
            tg.pixels.at(n - 3)?,
            tg.pixels.at(n - 2)?,
            tg.pixels.at(n - 1)?,
        );
        shape.move_to(a);
        shape.line_to(b);
        shape.line_to(c);
        shape.line_to(a);
    }
    Ok(shape)
}

/// Upstream `resolvePostClippedShapes`: the point and rectangle buffer
/// graphics keep their fill on the first shape and their hatch style on the
/// second. A graphic with fewer than two shapes keeps the first change, as
/// upstream's catch does.
pub(crate) fn resolve_post_clipped_shapes(tg: &Tg, shapes: &mut [Shape]) {
    if !matches!(
        tg.line_type,
        BBS_RECTANGLE | BBS_POINT | BBS_LINE | BBS_AREA | PBS_RECTANGLE | PBS_SQUARE
    ) {
        return;
    }
    let Some((first, rest)) = shapes.split_first_mut() else {
        return;
    };
    first.fill_color = tg.fill_color;
    if let Some(second) = rest.first_mut() {
        second.fill_color = None;
        first.fill_style = 0;
        second.fill_style = tg.fill_style;
    }
}
