use super::anchorage::{get_anchorage_count_double, get_anchorage_flot_segment};
use super::angle::{calc_angle_points, calc_new_point};
use super::flot_line::{get_flot_count_double, get_flot_double};
use super::flot_wf::{get_flot_count2_double, get_flot2_double};
use super::occluded::{get_occluded_count_double, get_occluded_points_double};
use super::ofy::{get_ofy_count_double, get_ofy_points_double};
use super::segment::FlipState;
use super::sf::{get_sf_count_double, get_sf_points_double};
use super::{FlotStyle, get_scaled_size};
use crate::engine::base::Pt;
use crate::engine::tactical_lines as tl;

fn line(coords: &[(f64, f64)]) -> Vec<Pt> {
    coords.iter().map(|&(x, y)| Pt::new(x, y)).collect()
}

fn style(line_type: i32) -> FlotStyle {
    FlotStyle::new(line_type, 0.0, 1.0)
}

#[test]
fn scaled_size_grows_only_above_the_default_width() {
    assert_eq!(get_scaled_size(20.0, 3.0, 1.0), 20.0);
    assert_eq!(get_scaled_size(20.0, 5.0, 1.0), 40.0);
    assert_eq!(
        get_scaled_size(20.0, 200.0, 1.0),
        get_scaled_size(20.0, 100.0, 1.0)
    );
}

#[test]
fn new_point_follows_the_compass_angle() {
    assert_eq!(calc_new_point(10, 0, 90.0, 10.0), [20, 0]);
    assert_eq!(calc_new_point(10, 0, 270.0, 10.0), [0, 0]);
    assert_eq!(calc_new_point(10, 0, 45.0, 10.0), [17, -7]);
}

#[test]
fn angle_points_trace_a_half_circle() {
    let p = calc_angle_points(10, 0, 0.0, 10.0);
    let xy = |j: usize| (p[3 * j], p[3 * j + 1]);
    assert_eq!(xy(0), (0, 0));
    assert_eq!(xy(1), (0, -3));
    assert_eq!(xy(2), (2, -6));
    assert_eq!(xy(4), (8, -9));
    assert_eq!(xy(5), (11, -9));
    assert_eq!(xy(9), (20, 0));
    assert!(p.chunks_exact(3).all(|t| t[2] == 0));
}

#[test]
fn flot_count_adds_ten_per_flot_and_one_per_short_segment() {
    let pts = line(&[(0.0, 0.0), (100.0, 0.0)]);
    assert_eq!(get_flot_count_double(&pts, 20.0, 2), Ok(51));
    let pts = line(&[(0.0, 0.0), (10.0, 0.0), (10.0, 100.0)]);
    assert_eq!(get_flot_count_double(&pts, 20.0, 3), Ok(52));
}

#[test]
fn flot_line_writes_half_circles_along_the_segment() {
    let mut pts = line(&[(0.0, 0.0), (100.0, 0.0)]);
    assert_eq!(get_flot_double(&mut pts, 20.0, 2), Ok(50));
    assert_eq!((pts[0].x, pts[0].y), (0.0, 0.0));
    assert_eq!((pts[9].x, pts[9].y), (20.0, 0.0));
    assert_eq!((pts[10].x, pts[10].y), (20.0, 0.0));
    assert_eq!(pts[48].style, 0);
    assert_eq!(pts[49].style, 5);
}

#[test]
fn flot_line_flips_the_side_when_the_direction_reverses() {
    let mut pts = line(&[(0.0, 0.0), (100.0, 0.0), (0.0, 0.0)]);
    assert!(get_flot_double(&mut pts, 20.0, 3).is_ok());
    // Flots bulge up (negative y) on the way out and down on the way back.
    assert!(pts[4].y < 0.0);
    assert!(pts[54].y > 0.0);
}

#[test]
fn flot_line_rejects_a_single_point() {
    let mut pts = line(&[(0.0, 0.0)]);
    assert!(get_flot_double(&mut pts, 20.0, 1).is_err());
}

#[test]
fn flot_line_rejects_output_beyond_its_budget() {
    let mut pts = line(&[(0.0, 0.0), (1.0e12, 0.0)]);
    assert!(get_flot_double(&mut pts, 20.0, 2).is_err());
}

#[test]
fn anchorage_skips_every_other_flot() {
    let pts = line(&[(0.0, 0.0), (100.0, 0.0)]);
    assert_eq!(get_anchorage_count_double(&pts, 20.0, 2), Ok(61));
    let vb = [0, 0, 100, 0];
    let mut points = vec![0_i32; 300];
    let mut state = FlipState::unset();
    let written = get_anchorage_flot_segment(&vb, (0, 0, 100, 0), 0, 20.0, &mut points, &mut state);
    assert_eq!(written, Ok(90));
    assert_eq!((points[0], points[1]), (0, 0));
    assert_eq!((points[30], points[31]), (40, 0));
    assert_eq!((points[57], points[58]), (60, 0));
}

#[test]
fn warm_front_counts_follow_the_variant() {
    let pts = line(&[(0.0, 0.0), (100.0, 0.0)]);
    assert_eq!(get_flot_count2_double(&style(tl::WF), &pts, 2), Ok(20));
    assert_eq!(get_flot_count2_double(&style(tl::WFG), &pts, 2), Ok(17));
    assert_eq!(get_flot_count2_double(&style(tl::WFY), &pts, 2), Ok(20));
    let short = line(&[(0.0, 0.0), (10.0, 0.0)]);
    assert_eq!(get_flot_count2_double(&style(tl::WFG), &short, 2), Ok(2));
    assert_eq!(get_flot_count2_double(&style(tl::WF), &short, 2), Ok(0));
}

#[test]
fn warm_front_marks_the_end_of_each_flot() {
    let mut pts = line(&[(0.0, 0.0), (100.0, 0.0)]);
    assert_eq!(get_flot2_double(&style(tl::WF), &mut pts, 2), Ok(20));
    assert_eq!(pts[0].style, 9);
    assert_eq!(pts[9].style, 10);
    assert_eq!(pts[10].style, 9);
    assert_eq!(pts[19].style, 10);
}

#[test]
fn warm_front_gold_appends_the_dotted_pieces() {
    let mut pts = line(&[(0.0, 0.0), (200.0, 0.0)]);
    let n = get_flot2_double(&style(tl::WFG), &mut pts, 2).unwrap_or(0);
    // Three 60-pixel flots (51 points), then the collected pieces.
    assert!(n > 30);
    assert!(pts.iter().take(n as usize).any(|p| p.style == 20));
}

#[test]
fn warm_front_without_flots_draws_nothing() {
    let mut pts = line(&[(0.0, 0.0), (10.0, 0.0)]);
    assert_eq!(get_flot2_double(&style(tl::WF), &mut pts, 2), Ok(0));
}

#[test]
fn occluded_count_has_a_floor_of_thirteen_per_point() {
    let pts = line(&[(0.0, 0.0), (100.0, 0.0)]);
    assert_eq!(get_occluded_count_double(&pts, 2), Ok(26));
    let pts = line(&[(0.0, 0.0), (200.0, 0.0)]);
    assert_eq!(get_occluded_count_double(&pts, 2), Ok(52));
}

#[test]
fn occluded_front_alternates_flots_and_spikes() {
    let mut pts = line(&[(0.0, 0.0), (200.0, 0.0)]);
    let n = get_occluded_points_double(&style(tl::OCCLUDED), &mut pts, 2);
    assert_eq!(n, Ok(52));
    for g in 0..4 {
        assert_eq!(pts[g * 13].style, 9);
        assert_eq!(pts[g * 13 + 9].style, 10);
        assert_eq!(pts[g * 13].y, 0.0);
    }
}

#[test]
fn occluded_front_rejects_an_unbounded_segment() {
    let mut pts = line(&[(0.0, 0.0), (1.0e12, 0.0)]);
    assert!(get_occluded_points_double(&style(tl::OCCLUDED), &mut pts, 2).is_err());
}

#[test]
fn ofy_count_has_a_floor_of_twenty_five_per_point() {
    let pts = line(&[(0.0, 0.0), (160.0, 0.0)]);
    assert_eq!(get_ofy_count_double(&pts, 80.0, 2), Ok(50));
    let pts = line(&[(0.0, 0.0), (10.0, 0.0)]);
    assert_eq!(get_ofy_count_double(&pts, 80.0, 2), Ok(50));
}

#[test]
fn ofy_front_starts_with_all_flot_points() {
    let mut pts = line(&[(0.0, 0.0), (400.0, 0.0)]);
    let n = get_ofy_points_double(&style(tl::OFY), &mut pts, 2).unwrap_or(0);
    assert!(n > 50);
    // Five 80-pixel flots, each closed by a style 10 point.
    assert_eq!(pts[9].style, 10);
    assert_eq!(pts[49].style, 10);
}

#[test]
fn sf_count_includes_the_line_feature_points() {
    let pts = line(&[(0.0, 0.0), (160.0, 0.0)]);
    assert_eq!(get_sf_count_double(&pts, 2), Ok(66));
    let pts = line(&[(0.0, 0.0), (10.0, 0.0)]);
    assert_eq!(get_sf_count_double(&pts, 2), Ok(50));
}

#[test]
fn stationary_fronts_build_for_every_variant() {
    for lt in [tl::SF, tl::USF, tl::SFG, tl::SFY] {
        let mut pts = line(&[(0.0, 0.0), (400.0, 0.0), (400.0, 300.0)]);
        let n = get_sf_points_double(&style(lt), &mut pts, 3);
        assert!(matches!(n, Ok(c) if c > 60), "{lt}: {n:?}");
        assert!(pts.iter().all(|p| p.x.is_finite() && p.y.is_finite()));
    }
}

#[test]
fn stationary_front_red_and_blue_flots_use_their_styles() {
    let mut pts = line(&[(0.0, 0.0), (400.0, 0.0)]);
    assert!(get_sf_points_double(&style(tl::SF), &mut pts, 2).is_ok());
    assert_eq!(pts[0].style, 19);
    assert_eq!(pts[9].style, 5);
    assert!(pts.iter().any(|p| p.style == 25));
    let mut pts = line(&[(0.0, 0.0), (400.0, 0.0)]);
    assert!(get_sf_points_double(&style(tl::SFG), &mut pts, 2).is_ok());
    assert_eq!(pts[0].style, 9);
    assert_eq!(pts[9].style, 23);
}

#[test]
fn thick_lines_grow_the_pattern() {
    let thick = FlotStyle::new(tl::WF, 7.0, 1.0);
    let pts = line(&[(0.0, 0.0), (100.0, 0.0)]);
    // The pitch becomes 40 * 3 = 120 pixels, longer than the segment.
    assert_eq!(get_flot_count2_double(&thick, &pts, 2), Ok(0));
}
