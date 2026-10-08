use super::*;
use crate::engine::base::{Pt, shape_type};
use crate::family::ported::ORIGIN_PX;

fn line(points: &[(f64, f64)]) -> Shape {
    let mut s = Shape::new(shape_type::POLYLINE);
    for (i, &(x, y)) in points.iter().enumerate() {
        let p = Pt::new(ORIGIN_PX + x, ORIGIN_PX + y);
        if i == 0 { s.move_to(p) } else { s.line_to(p) }
    }
    s
}

#[test]
fn proportional_shapes_are_geographic_and_pixel_sized_ones_are_not() {
    // Run b is drawn at half the metres per pixel (twice the pixels).
    let skeleton_a = line(&[(0.0, 0.0), (100.0, 0.0)]);
    let skeleton_b = line(&[(0.0, 0.0), (200.0, 0.0)]);
    // A 10 px tick stays 10 px at both sizes.
    let tick_a = line(&[(50.0, 0.0), (50.0, 10.0)]);
    let tick_b = line(&[(100.0, 0.0), (100.0, 10.0)]);
    let geo = geographic(
        &[skeleton_a, tick_a],
        2.0,
        &[skeleton_b, tick_b],
        1.0,
        200.0,
    );
    assert_eq!(geo, [true, false]);
}

#[test]
fn unpaired_runs_stay_in_pixels() {
    let a = [line(&[(0.0, 0.0), (1.0, 0.0)])];
    assert_eq!(geographic(&a, 1.0, &[], 1.0, 10.0), [false]);
}
