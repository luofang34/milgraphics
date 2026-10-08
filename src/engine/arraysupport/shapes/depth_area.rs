//! DEPTH_AREA: a white polygon with a pale blue and a blue band along the
//! inside of its outline. Upstream builds each band as the polygon's stroke
//! (`BasicStroke`, mitred) intersected with the polygon (`Area`): a fill
//! with a hole, which the output's single-ring polygons cannot carry. The
//! same band is drawn here as a mitred line along its centre, inset by half
//! its width, as wide as the band. A polygon too small for the inset is
//! drawn with its own outline.

use crate::engine::arraysupport::work::{Work, get};
use crate::engine::base::{EngineError, Pt, Shape, shape_type};
use crate::engine::flot::get_scaled_size;
use crate::style::Rgba;

/// Appends the white, pale blue and blue shapes.
pub(super) fn build(w: &Work<'_>, shapes: &mut Vec<Shape>) -> Result<(), EngineError> {
    let control: Vec<Pt> = (0..w.save)
        .map(|k| get(&w.p, k))
        .collect::<Result<_, _>>()?;
    let mut white = Shape::new(shape_type::FILL);
    white.fill_color = Some(Rgba::opaque(255, 255, 255));
    for (k, &p) in control.iter().enumerate() {
        if k == 0 {
            white.move_to(p);
        } else {
            white.line_to(p);
        }
    }
    // `java.awt.Polygon` holds whole pixels; a closing point repeats the
    // first and adds no edge.
    let mut poly: Vec<Pt> = Vec::with_capacity(control.len());
    for p in &control {
        let q = Pt::new(f64::from(p.x as i32), f64::from(p.y as i32));
        if poly.last() != Some(&q) {
            poly.push(q);
        }
    }
    if poly.len() > 1 && poly.first() == poly.last() {
        poly.pop();
    }
    let blue_width = get_scaled_size(14.0, f64::from(w.tg.line_thickness), w.tg.pattern_scale);
    let pale = band(&poly, blue_width, Rgba::opaque(153, 204, 255));
    let blue = band(&poly, blue_width / 2.0, Rgba::opaque(30, 144, 255));

    shapes.push(white);
    shapes.push(pale);
    shapes.push(blue);
    Ok(())
}

/// A band `width` wide inside the polygon's outline, as a line along its
/// centre.
fn band(poly: &[Pt], width: f64, color: Rgba) -> Shape {
    let mut s = Shape::new(shape_type::POLYLINE);
    s.line_color = Some(color);
    s.stroke.width = width;
    if poly.len() < 3 {
        return s;
    }
    let centre = inward(poly, width / 2.0).unwrap_or_else(|| poly.to_vec());
    for (k, &p) in centre.iter().chain(centre.first()).enumerate() {
        if k == 0 {
            s.move_to(p);
        } else {
            s.line_to(p);
        }
    }
    s
}

/// Twice the signed area (positive when clockwise on screen, y down).
fn signed_area(pts: &[Pt]) -> f64 {
    let n = pts.len();
    (0..n)
        .filter_map(|i| Some((pts.get(i)?, pts.get((i + 1) % n)?)))
        .map(|(a, b)| a.x * b.y - b.x * a.y)
        .sum()
}

/// The polygon offset inward by `d` with mitred corners, or `None` when the
/// offset collapses (reverses orientation or loses its area).
fn inward(poly: &[Pt], d: f64) -> Option<Vec<Pt>> {
    let n = poly.len();
    let area = signed_area(poly);
    if area == 0.0 {
        return None;
    }
    // Inward normal side for this orientation.
    let side = if area > 0.0 { 1.0 } else { -1.0 };
    let edge = |i: usize| -> Option<(Pt, (f64, f64))> {
        let a = *poly.get(i)?;
        let b = *poly.get((i + 1) % n)?;
        let (dx, dy) = (b.x - a.x, b.y - a.y);
        let len = dx.hypot(dy);
        (len > 0.0).then(|| {
            let (ux, uy) = (dx / len, dy / len);
            let normal = (-uy * side, ux * side);
            (Pt::new(a.x + normal.0 * d, a.y + normal.1 * d), (ux, uy))
        })
    };
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let (p0, u0) = edge((i + n - 1) % n)?;
        let (p1, u1) = edge(i)?;
        let cross = u0.0 * u1.1 - u0.1 * u1.0;
        let corner = if cross.abs() < 1e-12 {
            p1
        } else {
            let t = ((p1.x - p0.x) * u1.1 - (p1.y - p0.y) * u1.0) / cross;
            Pt::new(p0.x + u0.0 * t, p0.y + u0.1 * t)
        };
        out.push(corner);
    }
    let inner = signed_area(&out);
    (inner * area > 0.0 && inner.abs() < area.abs()).then_some(out)
}
