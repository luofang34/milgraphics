use super::*;
use crate::engine::settings::Settings;

fn lc_with(points: &[(f64, f64)]) -> Tg {
    let mut tg = Tg::new(&Settings::default());
    tg.line_type = lt::LC;
    tg.pixels = points.iter().map(|&(x, y)| Pt::new(x, y)).collect();
    tg
}

fn xy(tg: &Tg) -> Vec<(f64, f64)> {
    tg.pixels.iter().map(|p| (p.x, p.y)).collect()
}

#[test]
fn reverse_for_friendly_in_quadrants_two_and_three() {
    // Down-right: quadrant 2, reversed for a non-hostile line.
    let mut tg = lc_with(&[(0.0, 0.0), (10.0, 10.0), (20.0, 0.0)]);
    reverse_usas_lc_points_by_quadrant(&mut tg).unwrap();
    assert_eq!(xy(&tg), vec![(20.0, 0.0), (10.0, 10.0), (0.0, 0.0)]);
    // Up-right: quadrant 1, kept.
    let mut tg = lc_with(&[(0.0, 10.0), (10.0, 0.0)]);
    reverse_usas_lc_points_by_quadrant(&mut tg).unwrap();
    assert_eq!(xy(&tg), vec![(0.0, 10.0), (10.0, 0.0)]);
}

#[test]
fn reverse_for_hostile_in_quadrants_one_and_four() {
    let mut tg = lc_with(&[(0.0, 10.0), (10.0, 0.0)]);
    tg.standard_identity = "06".to_owned();
    reverse_usas_lc_points_by_quadrant(&mut tg).unwrap();
    assert_eq!(xy(&tg), vec![(10.0, 0.0), (0.0, 10.0)]);
}

#[test]
fn other_line_types_are_not_reversed() {
    let mut tg = lc_with(&[(0.0, 0.0), (10.0, 10.0)]);
    tg.line_type = lt::PL;
    reverse_usas_lc_points_by_quadrant(&mut tg).unwrap();
    assert_eq!(xy(&tg), vec![(0.0, 0.0), (10.0, 10.0)]);
}

#[test]
fn tight_angle_gets_a_split_point_on_the_longer_arm() {
    // BC is the shorter arm, so the point goes on BA at 100 from B.
    let mut tg = lc_with(&[(0.0, 10.0), (100.0, 0.0), (0.0, 0.0)]);
    segment_lc_points(&mut tg).unwrap();
    assert_eq!(tg.pixels.len(), 4);
    let p = tg.pixels[1];
    let len = (100.0_f64 * 100.0 + 100.0).sqrt();
    assert!((p.x - (100.0 - 100.0 * 100.0 / len)).abs() < 1e-9);
    assert!((p.y - 100.0 * 10.0 / len).abs() < 1e-9);
}

#[test]
fn open_angle_adds_nothing() {
    let mut tg = lc_with(&[(0.0, 0.0), (100.0, 0.0), (200.0, 0.0)]);
    segment_lc_points(&mut tg).unwrap();
    assert_eq!(tg.pixels.len(), 3);
}
