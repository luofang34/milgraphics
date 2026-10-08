//! Hand-derived checks of the lineutility primitives against the arithmetic
//! of the upstream Java.

use super::arc::{arc_array_double, calc_clockwise_center_double, get_arc_points_double};
use super::arrow::get_arrow_head4_double;
use super::basics::*;
use super::bounds::mbr_distance;
use super::channel_pixels::move_channel_pixels;
use super::circle::calc_circle_double;
use super::ditch::get_ditch_spike_double;
use super::extend::*;
use super::intersect::calc_distance2;
use super::relative::closest_point_on_line;
use super::saafr::get_saafr_fill_segment;
use super::slope::*;
use super::squall::get_squall_segment;
use crate::engine::base::Pt;
use crate::engine::tactical_lines as lt;

const EPS: f64 = 1e-9;

fn near(p: Pt, x: f64, y: f64) -> bool {
    (p.x - x).abs() < EPS && (p.y - y).abs() < EPS
}

#[test]
fn distance_and_integer_distance() {
    assert_eq!(
        calc_distance_double(Pt::new(0.0, 0.0), Pt::new(3.0, 4.0)),
        5.0
    );
    assert_eq!(
        calc_distance_double(Pt::new(2.0, 2.0), Pt::new(2.0, 2.0)),
        0.0
    );
    assert_eq!(calc_distance2(0, 0, 3, 4), 5.0);
}

#[test]
fn extend_along_and_beyond() {
    let (a, b) = (Pt::new(0.0, 0.0), Pt::new(10.0, 0.0));
    assert!(near(extend_along_line_double(a, b, 4.0), 4.0, 0.0));
    assert_eq!(extend_along_line_double(a, b, 0.0), b);
    assert_eq!(extend_along_line_double2(a, b, 0.0), a);
    assert!(near(extend_line_double(a, b, 5.0), 15.0, 0.0));
    let p = extend_line2_double(a, b, -2.0, 5);
    assert!(near(p, 8.0, 0.0) && p.style == 5);
    let q = extend_along_line_double_style(a, b, 3.0, 9);
    assert!(near(q, 3.0, 0.0) && q.style == 9);
}

#[test]
fn directed_line_on_horizontal_and_diagonal() {
    let (a, b, mid) = (Pt::new(0.0, 0.0), Pt::new(10.0, 0.0), Pt::new(5.0, 0.0));
    assert!(near(extend_directed_line(a, b, mid, 2, 3.0), 5.0, -3.0));
    assert!(near(extend_directed_line(a, b, mid, 3, 3.0), 5.0, 3.0));
    // Left of a horizontal line is undefined upstream and yields the origin.
    assert!(near(extend_directed_line(a, b, mid, 0, 3.0), 0.0, 0.0));
    // The style variant maps left to above on horizontal lines.
    let p = extend_directed_line_style(a, b, mid, 0, 3.0, 7);
    assert!(near(p, 5.0, -3.0) && p.style == 7);
    // A direction code outside 0..=3 returns the point unchanged.
    assert_eq!(extend_directed_line(a, b, mid, 9, 3.0), mid);
    let d = Pt::new(10.0, 10.0);
    let r = 10.0 / 2.0_f64.sqrt();
    assert!(near(extend_directed_line(a, d, a, 2, 10.0), r, -r));
}

#[test]
fn midpoint_keeps_first_point_attributes() {
    let m = mid_point_double(Pt::styled(0.0, 0.0, 3), Pt::new(4.0, 6.0), 5);
    assert!(near(m, 2.0, 3.0) && m.style == 5);
}

#[test]
fn slope_widens_near_vertical_runs() {
    let (ok, m) = calc_true_slope_double(Pt::new(0.0, 0.0), Pt::new(10.0, 5.0));
    assert_eq!((ok, m), (1, 0.5));
    let (_, m) = calc_true_slope_double(Pt::new(0.0, 0.0), Pt::new(0.5, 3.0));
    assert_eq!(m, 3.0);
    let (ok2, _) = calc_true_slope_double2(Pt::new(0.0, 0.0), Pt::new(0.5, 3.0));
    assert!(!ok2);
    assert!(!calc_true_slope_double_for_routes(Pt::new(0.0, 0.0), Pt::new(1.5, 3.0)).0);
}

#[test]
fn intersection_and_line_distance() {
    let p = calc_true_intersect_double2(1.0, 0.0, -1.0, 10.0, 1, 1, (0.0, 0.0));
    assert!(near(p, 5.0, 5.0));
    let v = calc_true_intersect_double2(0.0, 0.0, 2.0, 1.0, 0, 1, (4.0, 0.0));
    assert!(near(v, 4.0, 9.0));
    let d = calc_distance_to_line_double(Pt::new(0.0, 0.0), Pt::new(10.0, 0.0), Pt::new(3.0, 4.0));
    assert_eq!(d, 4.0);
}

#[test]
fn direction_from_line() {
    let (a, b) = (Pt::new(0.0, 0.0), Pt::new(0.0, 10.0));
    assert_eq!(calc_direction_from_line(a, b, Pt::new(-1.0, 5.0)), 0);
    assert_eq!(calc_direction_from_line(a, b, Pt::new(1.0, 5.0)), 1);
    let c = Pt::new(10.0, 0.0);
    assert_eq!(calc_direction_from_line(a, c, Pt::new(5.0, -2.0)), 2);
    assert_eq!(calc_direction_from_line(a, c, Pt::new(5.0, 2.0)), 3);
    assert_eq!(reverse_direction(2), 3);
}

#[test]
fn offset_point_beyond_end() {
    let p = get_offset_point_double(Pt::new(0.0, 0.0), Pt::new(10.0, 0.0), 5);
    assert!(near(p, 15.0, 0.0));
}

#[test]
fn arrowhead_truncates_to_whole_pixels() {
    let mut out = [Pt::default(); 3];
    get_arrow_head4_double(Pt::new(0.0, 0.0), Pt::new(10.0, 0.0), 5, 6, &mut out, 0).unwrap();
    // angle = pi, so sin(pi) = 1.2246e-16 pushes the exact 3 to 2.9999999999999996
    // and the exact -3 to -3.0000000000000004 before truncation.
    assert!(near(out[0], 5.0, 2.0));
    assert!(near(out[1], 10.0, 0.0));
    assert!(near(out[2], 5.0, -3.0));
    assert_eq!(out.map(|p| p.style), [0, 0, 5]);
    get_arrow_head4_double(Pt::new(0.0, 0.0), Pt::new(10.0, 0.0), 5, 6, &mut out, 9).unwrap();
    assert_eq!(out.map(|p| p.style), [9, 9, 10]);
}

#[test]
fn arc_array_full_circle_truncates() {
    let mut pts = vec![Pt::default(); 30];
    pts[0] = Pt::new(100.0, 100.0);
    pts[1] = Pt::new(110.0, 100.0);
    arc_array_double(&mut pts, 10.0, 0).unwrap();
    assert!(near(pts[0], 100.0, 100.0));
    // 6 * (2 pi / 25) rad: cos = 0.0628 truncates to 0, sin = 0.998 to 9.
    assert!(near(pts[6], 90.0, 109.0));
    assert!(near(pts[25], 100.0, 100.0));
    assert!(near(pts[26], 0.0, 0.0));
}

#[test]
fn circle_closes_with_style_mapping() {
    let mut pts = vec![Pt::default(); 5];
    calc_circle_double(Pt::new(0.0, 0.0), 2.0, 5, &mut pts, 9).unwrap();
    assert!(near(pts[0], 2.0, 0.0));
    assert!(near(pts[1], 0.0, 2.0));
    assert!(near(pts[2], -2.0, 0.0));
    assert!(near(pts[4], 2.0, 0.0));
    assert_eq!((pts[0].style, pts[4].style), (9, 10));
}

#[test]
fn arc_points_follow_the_shorter_sweep() {
    let pts = get_arc_points_double(Pt::new(1.0, 0.0), Pt::new(0.0, 1.0), Pt::new(0.0, 0.0), 2);
    assert_eq!(pts.len(), 3);
    let h = std::f64::consts::FRAC_1_SQRT_2;
    assert!(near(pts[1], h, h));
    assert!(near(pts[2], 0.0, 1.0));
}

#[test]
fn clockwise_center_of_horizontal_pair() {
    let mut pts = [Pt::new(0.0, 0.0), Pt::new(10.0, 0.0)];
    let r = calc_clockwise_center_double(&mut pts).unwrap();
    assert!((r - 10.0 / 2.0_f64.sqrt()).abs() < EPS);
    assert!(near(pts[0], 5.0, 5.0));
}

#[test]
fn closest_point_clamps_to_the_segment() {
    let (a, b) = (Pt::new(0.0, 0.0), Pt::new(10.0, 0.0));
    assert!(near(
        closest_point_on_line(a, b, Pt::new(4.0, 3.0)),
        4.0,
        0.0
    ));
    assert!(near(
        closest_point_on_line(a, b, Pt::new(20.0, 3.0)),
        10.0,
        0.0
    ));
}

#[test]
fn bounds_and_small_array_helpers() {
    let pts = [Pt::new(0.0, 0.0), Pt::new(3.0, 4.0), Pt::new(1.0, 1.0)];
    assert_eq!(mbr_distance(&pts, 3).unwrap(), 5.0);
    assert_eq!(get_pixels_min(&pts, 3).unwrap(), (0.0, 0.0));
    assert!(near(calc_center_point_double(&pts, 3).unwrap(), 1.5, 2.0));
    assert_eq!(resize_array(&pts, 2).unwrap().len(), 2);
    assert_eq!(resize_array(&pts, 5).unwrap().len(), 3);
    let mut r = pts;
    reverse_points_double2(&mut r, 3).unwrap();
    assert!(near(r[0], 1.0, 1.0) && near(r[2], 0.0, 0.0));
    assert!(get_pixels_min(&pts, 4).is_err());
    assert_eq!(
        get_quadrant_double(Pt::new(0.0, 0.0), Pt::new(5.0, -5.0)),
        1
    );
}

#[test]
fn coincident_channel_points_are_separated() {
    let mut pts = [Pt::new(1.0, 1.0); 3];
    move_channel_pixels(&mut pts).unwrap();
    assert!(near(pts[0], 1.0, 1.0) && near(pts[1], 2.0, 1.0) && near(pts[2], 1.0, 1.0));
}

#[test]
fn saafr_fill_quad_corners() {
    let mut pts = [
        Pt::new(0.0, 0.0),
        Pt::new(10.0, 0.0),
        Pt::default(),
        Pt::default(),
    ];
    get_saafr_fill_segment(&mut pts, 2.0).unwrap();
    assert!(near(pts[0], 0.0, -2.0) && near(pts[1], 10.0, -2.0));
    assert!(near(pts[2], 10.0, 2.0) && near(pts[3], 0.0, 2.0));
}

#[test]
fn squall_shorter_than_one_wave_is_a_straight_segment() {
    let mut out = [Pt::default(); 4];
    let mut sign = 1;
    let n = get_squall_segment(
        Pt::new(0.0, 0.0),
        Pt::new(10.0, 0.0),
        &mut out,
        &mut sign,
        3.0,
        4,
        20.0,
    )
    .unwrap();
    assert_eq!(n, 2);
    assert!(near(out[1], 10.0, 0.0));
}

#[test]
fn ditch_spikes_on_one_horizontal_segment() {
    let mut pts = vec![Pt::new(0.0, 0.0), Pt::new(100.0, 0.0)];
    let n = get_ditch_spike_double(lt::ATDITCH, 10.0, 4.0, &mut pts, 2, 0).unwrap();
    // 9 spikes of 3 points after the 2 originals, then 9 base-line points.
    assert_eq!(n, 38);
    assert_eq!(pts.len(), 38);
    assert!(near(pts[0], 100.0, 0.0) && pts[0].style == 5);
    assert!(near(pts[1], 0.0, 0.0) && pts[1].style == 5);
    assert!(near(pts[2], 5.0, 0.0));
    assert!(near(pts[3], 10.0, -11.0));
    assert!(near(pts[4], 15.0, 0.0));
    assert_eq!(pts[28].style, 5);
    assert!(near(pts[29], 0.0, 0.0));
    assert!(near(pts[37], 100.0, 0.0) && pts[37].style == 5);
}
