use super::*;
use crate::engine::settings::Settings;

fn tg_with(line_type: i32, pts: &[(f64, f64)]) -> Tg {
    let mut tg = Tg::new(&Settings::default());
    tg.line_type = line_type;
    tg.pixels = pts.iter().map(|&(x, y)| Pt::new(x, y)).collect();
    tg
}

#[test]
fn close_polygon_appends_first_point_once() {
    let mut pts = vec![Pt::new(0.0, 0.0), Pt::new(5.0, 0.0), Pt::new(5.0, 5.0)];
    close_polygon(&mut pts);
    assert_eq!(pts.len(), 4);
    assert_eq!(pts.last().copied(), Some(Pt::new(0.0, 0.0)));
    close_polygon(&mut pts);
    assert_eq!(pts.len(), 4);
}

#[test]
fn intersection_of_two_lines() {
    let mut p = Pt::default();
    calc_intersect_pt(Pt::new(0.0, 0.0), 1.0, Pt::new(4.0, 0.0), -1.0, &mut p);
    assert!((p.x - 2.0).abs() < 1e-12 && (p.y - 2.0).abs() < 1e-12);
    let mut q = Pt::new(7.0, 7.0);
    calc_intersect_pt(Pt::new(0.0, 0.0), 1.0, Pt::new(4.0, 0.0), 1.0, &mut q);
    assert_eq!(q, Pt::new(7.0, 7.0));
}

#[test]
fn order_tests() {
    let (a, b, c) = (Pt::new(0.0, 3.0), Pt::new(1.0, 2.0), Pt::new(2.0, 1.0));
    assert!(in_x_order(a, b, c));
    assert!(in_y_order(a, b, c));
    assert!(!in_x_order(a, c, b));
}

#[test]
fn reorder_reverses_all_but_last_pair() {
    let mut p = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
    reorder_pixels(&mut p).unwrap();
    assert_eq!(p, [5.0, 6.0, 3.0, 4.0, 1.0, 2.0, 7.0, 8.0]);
    let mut single = [1.0, 2.0];
    reorder_pixels(&mut single).unwrap();
    assert_eq!(single, [1.0, 2.0]);
    assert!(reorder_pixels(&mut []).is_err());
}

#[test]
fn vertical_segments_are_nudged_for_listed_types_only() {
    let mut tg = tg_with(MAIN, &[(0.0, 0.0), (0.4, 10.0), (1.4, 20.0), (-5.0, 30.0)]);
    filter_vertical_segments(&mut tg);
    let xs: Vec<f64> = tg.pixels.iter().map(|p| p.x).collect();
    // Each nudge is relative to the already moved predecessor.
    assert_eq!(xs, vec![0.0, 1.4, 2.4, -5.0]);
    let mut other = tg_with(PL, &[(0.0, 0.0), (0.4, 10.0)]);
    filter_vertical_segments(&mut other);
    assert_eq!(other.pixels.get(1).map(|p| p.x), Some(0.4));
}

#[test]
fn duplicates_are_removed_down_to_the_minimum() {
    let mut tg = tg_with(PL, &[(0.0, 0.0), (0.2, 0.2), (10.0, 0.0), (10.1, 0.1)]);
    remove_duplicate_points(&mut tg, None);
    assert_eq!(tg.pixels.len(), 2);
    let mut two = tg_with(PL, &[(0.0, 0.0), (0.1, 0.1)]);
    remove_duplicate_points(&mut two, None);
    assert_eq!(two.pixels.len(), 2);
}

#[test]
fn duplicate_removal_respects_autoshapes_and_segment_data() {
    let fixed = MsInfo {
        draw_rule: 1,
        min_points: 3,
        max_points: 3,
    };
    let pts = [(0.0, 0.0), (0.1, 0.1), (10.0, 0.0)];
    let mut auto = tg_with(PL, &pts);
    remove_duplicate_points(&mut auto, Some(fixed));
    assert_eq!(auto.pixels.len(), 3);
    let mut main = tg_with(MAIN, &pts);
    remove_duplicate_points(&mut main, None);
    assert_eq!(main.pixels.len(), 3);
    let mut boundary = tg_with(BOUNDARY, &pts);
    boundary.h = "0:FF0000,1:00FF00".to_owned();
    remove_duplicate_points(&mut boundary, None);
    assert_eq!(boundary.pixels.len(), 3);
}

#[test]
fn point_order_is_reversed_for_listed_types() {
    let mut tg = tg_with(UNSP, &[(0.0, 0.0), (1.0, 1.0)]);
    tg.symbol_id = "110325000000001400000000000000".to_owned();
    reverse_points_rev_d(&mut tg);
    assert_eq!(tg.pixels.first().map(|p| p.x), Some(1.0));
    let mut cluster_e = tg_with(CLUSTER, &[(0.0, 0.0), (1.0, 1.0)]);
    cluster_e.symbol_id = "130325000000001400000000000000".to_owned();
    reverse_points_rev_d(&mut cluster_e);
    assert_eq!(cluster_e.pixels.first().map(|p| p.x), Some(0.0));
    let mut cluster_d = tg_with(CLUSTER, &[(0.0, 0.0), (1.0, 1.0)]);
    cluster_d.symbol_id = "110325000000001400000000000000".to_owned();
    reverse_points_rev_d(&mut cluster_d);
    assert_eq!(cluster_d.pixels.first().map(|p| p.x), Some(1.0));
}
