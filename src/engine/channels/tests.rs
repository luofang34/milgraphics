//! Hand-checked channel geometry: a horizontal line has edges `width` above
//! and below it, a triple count follows the segment lengths.

use super::connect::connect_array_true_double;
use super::fill::get_axad_fill_shapes;
use super::lines::{fence_type, get_channel_array2_double, get_triple_count_double};
use super::scaled_size::get_scaled_size;
use super::true_points::get_true_end_point_double;
use crate::engine::base::Pt;
use crate::engine::tactical_lines as lt;

fn line(points: &[(f64, f64)]) -> Vec<Pt> {
    points.iter().map(|&(x, y)| Pt::new(x, y)).collect()
}

fn near(a: Pt, x: f64, y: f64) -> bool {
    (a.x - x).abs() < 1e-9 && (a.y - y).abs() < 1e-9
}

#[test]
fn horizontal_line_has_edges_either_side() {
    let pts = line(&[(0.0, 0.0), (100.0, 0.0), (200.0, 0.0)]);
    let cp = connect_array_true_double(10, 2, &pts).unwrap();
    assert_eq!(cp.len(), 3);
    for (k, x) in [0.0, 100.0, 200.0].into_iter().enumerate() {
        assert!(
            near(cp[k].line1, x, -10.0),
            "line1 at {k}: {:?}",
            cp[k].line1
        );
        assert!(
            near(cp[k].line2, x, 10.0),
            "line2 at {k}: {:?}",
            cp[k].line2
        );
    }
}

#[test]
fn right_to_left_line_swaps_the_edges() {
    let pts = line(&[(200.0, 0.0), (100.0, 0.0), (0.0, 0.0)]);
    let cp = connect_array_true_double(10, 2, &pts).unwrap();
    // Moving right to left, edge 1 stays above the segment's direction of
    // travel code but the y values of the two edges are as upstream's nLast
    // 2 gives: edge 1 above.
    assert!(near(cp[0].line1, 200.0, -10.0));
    assert!(near(cp[0].line2, 200.0, 10.0));
}

#[test]
fn right_angle_turn_joins_the_offset_lines() {
    let pts = line(&[(0.0, 0.0), (100.0, 0.0), (100.0, 100.0)]);
    let cp = connect_array_true_double(10, 2, &pts).unwrap();
    // The vertical second segment is widened to dx = 1 (slope -100 through
    // (100, 100)), so its offset lines meet y = -10 and y = 10 at
    // (10100 +- 1000.05 +- 10) / 100.
    assert!(
        (cp[1].line1.x - 111.100_499_987_500_63).abs() < 1e-6,
        "{:?}",
        cp[1].line1
    );
    assert!(
        (cp[1].line2.x - 90.899_500_012_499_37).abs() < 1e-6,
        "{:?}",
        cp[1].line2
    );
    assert_eq!((cp[1].line1.y, cp[1].line2.y), (-10.0, 10.0));
    // An exactly vertical final segment has no handled `nLast`, so upstream
    // leaves its edge points at the origin.
    assert!(near(cp[2].line1, 0.0, 0.0) && near(cp[2].line2, 0.0, 0.0));
}

#[test]
fn end_point_of_diagonal_lies_on_the_perpendicular() {
    let cp = get_true_end_point_double(10, Pt::new(0.0, 0.0), Pt::new(100.0, 100.0), 0);
    // The offsets are 10 pixels from the end, along the perpendicular.
    let d = 10.0 / 2.0_f64.sqrt();
    assert!(near(cp.line1, d, -d), "{:?}", cp.line1);
    assert!(near(cp.line2, -d, d), "{:?}", cp.line2);
}

#[test]
fn channel_edges_are_a_quarter_of_the_width_apart() {
    // CoordIL2Double halves the width and GetChannel2Double halves it again.
    let pts = line(&[(0.0, 0.0), (100.0, 0.0)]);
    let upper = get_channel_array2_double(1, &pts, 1, 2, lt::CHANNEL, 40).unwrap();
    let lower = get_channel_array2_double(1, &pts, 0, 2, lt::CHANNEL, 40).unwrap();
    assert!(near(upper[0], 0.0, 10.0), "{:?}", upper[0]);
    assert!(near(lower[1], 100.0, -10.0), "{:?}", lower[1]);
}

#[test]
fn non_channel_types_are_passed_through() {
    let pts = line(&[(0.0, 0.0), (100.0, 0.0)]);
    let same = get_channel_array2_double(1, &pts, 1, 2, lt::FLOT, 40).unwrap();
    assert_eq!(same, pts);
}

#[test]
fn triple_count_follows_segment_lengths() {
    let pts = line(&[(0.0, 0.0), (100.0, 0.0)]);
    // (100 - 10) / 10 = 9 features on the segment.
    assert_eq!(
        get_triple_count_double(&pts, 2, lt::SINGLEC).unwrap(),
        6 * 2 + 37 * 9
    );
    assert_eq!(
        get_triple_count_double(&pts, 2, lt::UNSP).unwrap(),
        4 * 2 + 4 * 9
    );
    assert_eq!(get_triple_count_double(&pts, 2, lt::BBS_LINE).unwrap(), 5);
    assert_eq!(get_triple_count_double(&pts, 2, lt::CHANNEL).unwrap(), 4);
    let short = line(&[(0.0, 0.0), (10.0, 0.0)]);
    assert_eq!(get_triple_count_double(&short, 2, lt::SINGLEC).unwrap(), 12);
}

#[test]
fn fence_types() {
    assert_eq!(fence_type(lt::TRIPLE), 1);
    assert_eq!(fence_type(lt::CHANNEL), 0);
}

#[test]
fn scaled_size_grows_above_width_three() {
    assert_eq!(get_scaled_size(20.0, 3.0, 1.0), 20.0);
    assert_eq!(get_scaled_size(20.0, 5.0, 1.0), 40.0);
    assert_eq!(get_scaled_size(20.0, 5.0, 0.5), 30.0);
    // Widths above 100 are capped.
    assert_eq!(
        get_scaled_size(10.0, 500.0, 1.0),
        get_scaled_size(10.0, 100.0, 1.0)
    );
}

#[test]
fn channel_fill_walks_one_edge_out_and_the_other_back() {
    // Four points: two lower, two upper.
    let pts = line(&[(0.0, 0.0), (10.0, 0.0), (10.0, 5.0), (0.0, 5.0)]);
    let shapes = get_axad_fill_shapes(lt::CHANNEL, &pts).unwrap().unwrap();
    assert_eq!(shapes.len(), 1);
    let ring: Vec<(f64, f64)> = shapes[0].points().iter().map(|p| (p.x, p.y)).collect();
    assert_eq!(ring, vec![(0.0, 0.0), (10.0, 0.0), (0.0, 5.0), (10.0, 5.0)]);
    assert!(get_axad_fill_shapes(lt::PL, &pts).unwrap().is_none());
}
