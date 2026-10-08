use super::*;
use crate::engine::settings::Settings;

fn axis(points: &[(f64, f64)]) -> Tg {
    let mut tg = Tg::new(&Settings::default());
    tg.line_type = lt::MAIN;
    tg.pixels = points.iter().map(|&(x, y)| Pt::new(x, y)).collect();
    tg
}

fn xy(tg: &Tg) -> Vec<(f64, f64)> {
    tg.pixels.iter().map(|p| (p.x, p.y)).collect()
}

#[test]
fn second_point_too_close_is_pushed_out() {
    // The control point is 5 from the first segment's perpendicular, so
    // the second point must be 10 from the first.
    let mut tg = axis(&[(0.0, 0.0), (2.0, 0.0), (5.0, 50.0)]);
    filter_axad_points(&mut tg).unwrap();
    assert_eq!(xy(&tg), vec![(0.0, 0.0), (10.0, 0.0), (5.0, 50.0)]);
}

#[test]
fn good_points_pass_through_and_close_points_are_dropped() {
    let mut tg = axis(&[
        (0.0, 0.0),
        (100.0, 0.0),
        (102.0, 0.0),
        (200.0, 0.0),
        (0.0, 30.0),
    ]);
    filter_axad_points(&mut tg).unwrap();
    // The control point is on the first point's perpendicular, so the
    // second point is already far enough out; (102, 0) is within 5 pixels of
    // (100, 0).
    assert_eq!(
        xy(&tg),
        vec![(0.0, 0.0), (100.0, 0.0), (200.0, 0.0), (0.0, 30.0)]
    );
}

#[test]
fn other_line_types_are_untouched() {
    let mut tg = axis(&[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0)]);
    tg.line_type = lt::PL;
    filter_axad_points(&mut tg).unwrap();
    assert_eq!(tg.pixels.len(), 3);
}
