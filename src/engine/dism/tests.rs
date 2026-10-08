use super::*;
use crate::engine::base::{EngineError, PathOp, Pt};
use crate::engine::settings::Settings;
use crate::engine::tactical_lines as lt;
use crate::engine::tg::Tg;

/// One recorded upstream call: builder name, line type, `vblCounter`,
/// control points, and the points upstream returned.
struct Case {
    dpi: i32,
    name: String,
    line_type: i32,
    vbl: i32,
    control: Vec<f64>,
    count: i32,
    expected: Vec<(f64, f64, i32)>,
}

fn parse_case(line: &str) -> Case {
    let (head, tail) = line.split_once(" => ").unwrap();
    let mut it = head.split(' ');
    let dpi = it.next().unwrap().parse().unwrap();
    let name = it.next().unwrap().to_owned();
    let line_type = it.next().unwrap().parse().unwrap();
    let vbl = it.next().unwrap().parse().unwrap();
    let n: usize = it.next().unwrap().parse().unwrap();
    let control: Vec<f64> = it.map(|s| s.parse().unwrap()).collect();
    assert_eq!(control.len(), n * 2);
    let mut t = tail.trim_end().split(' ');
    let count = t.next().unwrap().parse().unwrap();
    let rest: Vec<&str> = t.collect();
    let expected = rest
        .chunks(3)
        .map(|c| {
            (
                c[0].parse().unwrap(),
                c[1].parse().unwrap(),
                c[2].parse().unwrap(),
            )
        })
        .collect();
    Case {
        dpi,
        name,
        line_type,
        vbl,
        control,
        count,
        expected,
    }
}

fn run_case(c: &Case, pts: &mut Vec<Pt>, s: &Settings) -> Result<(i32, Vec<Pt>), EngineError> {
    let lt = c.line_type;
    let done = |count: i32, pts: &Vec<Pt>| Ok((count, pts.clone()));
    match c.name.as_str() {
        "bypass" => bypass::get_dism_bypass_double(pts, s).and_then(|n| done(n, pts)),
        "breach" => bypass::get_dism_breach_double(pts, s).and_then(|n| done(n, pts)),
        "canalize" => bypass::get_dism_canalize_double(pts, s).and_then(|n| done(n, pts)),
        "easy" => bypass::get_dism_easy_double(pts, s).and_then(|n| done(n, pts)),
        "byimp" => bypass::get_dism_by_imp_double(pts, s).and_then(|n| done(n, pts)),
        "bydif" => rip::get_dism_by_dif_double(pts, s).and_then(|n| done(n, pts)),
        "fix" => fix::get_dism_fix_double(pts, lt, s).and_then(|n| done(n, pts)),
        "clear" => fix::get_dism_clear_double(pts, s).and_then(|n| done(n, pts)),
        "seize" => seize::get_dism_seize_double(pts, 0.0, s).and_then(|n| done(n, pts)),
        "seizer" => seize::get_dism_seize_double(pts, 17.5, s).and_then(|n| done(n, pts)),
        "rip" => rip::get_dism_rip_double(pts, lt, s).and_then(|n| done(n, pts)),
        "delay" => delay::get_delay_graphic_etc_double(pts, lt, s).and_then(|n| done(n, pts)),
        "envelopment" => delay::get_envelopment_graphic_double(pts, s).and_then(|n| done(n, pts)),
        "infiltration" => delay::get_infiltration_double(pts, s).map(|v| (v.len() as i32 - 3, v)),
        "disrupt" => disrupt::get_dism_disrupt_double(pts, s).and_then(|n| done(n, pts)),
        "contain" => disrupt::get_dism_contain_double(pts, s).and_then(|n| done(n, pts)),
        "deceive" => disrupt::get_dism_deceive_double(pts).and_then(|()| done(4, pts)),
        "penetrate" => rip::get_dism_penetrate_double(pts, s).and_then(|()| done(7, pts)),
        "block" => target::get_dism_block_double2(pts, lt).and_then(|()| done(4, pts)),
        "gap" => target::get_dism_gap_double(pts, s).and_then(|n| done(n, pts)),
        "mnflddis" => target::get_dism_minefield_disrupt_double(pts, s).and_then(|n| done(n, pts)),
        "lintgt" => {
            target::get_dism_linear_target_double(pts, lt, c.vbl, s).and_then(|n| done(n, pts))
        }
        "sptbyfire" => fire::get_dism_support_by_fire_double(pts, s).and_then(|n| done(n, pts)),
        "atkbyfire" => fire::get_dism_atk_by_fire_double(pts, s).and_then(|n| done(n, pts)),
        "ambush" => escort::ambush_points_double(pts, s).and_then(|n| done(n, pts)),
        "cover" => cover::get_dism_cover_double(pts, lt, s).and_then(|n| done(n, pts)),
        "coverc" => {
            cover::get_dism_cover_double_rev_c(pts, lt, c.vbl, s).and_then(|n| done(n, pts))
        }
        "escort" => escort::get_dism_escort_double(&Tg::new(s), pts).and_then(|n| done(n, pts)),
        other => panic!("unknown builder {other}"),
    }
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-9 * (1.0 + b.abs())
}

/// Every builder reproduces the recorded upstream output (position within
/// 1e-9 relative, exact style) for random inputs at 96 and 144 DPI.
#[test]
fn builders_match_recorded_upstream_output() {
    let text = include_str!("tests/java_cases.txt");
    let mut checked = 0;
    for line in text.lines().filter(|l| !l.starts_with('#')) {
        let c = parse_case(line);
        let settings = Settings {
            dpi: c.dpi,
            ..Settings::default()
        };
        let mut pts = vec![Pt::default(); 60];
        for (slot, xy) in pts.iter_mut().zip(c.control.chunks(2)) {
            slot.x = xy[0];
            slot.y = xy[1];
        }
        let (count, got) = run_case(&c, &mut pts, &settings).unwrap();
        assert_eq!(count, c.count, "{} count for {:?}", c.name, c.control);
        for (i, (x, y, style)) in c.expected.iter().enumerate() {
            let p = got[i];
            assert!(
                close(p.x, *x) && close(p.y, *y) && p.style == *style,
                "{} point {i} for {:?}: ({}, {}, {}) != ({x}, {y}, {style})",
                c.name,
                c.control,
                p.x,
                p.y,
                p.style
            );
        }
        checked += 1;
    }
    assert!(checked > 100);
}

fn pts_from(coords: &[(f64, f64)]) -> Vec<Pt> {
    coords.iter().map(|&(x, y)| Pt::new(x, y)).collect()
}

#[test]
fn glyph_size_tiers_follow_the_pixel_thresholds() {
    assert_eq!(support::get_tg_font_size(19.9), 0.0);
    assert_eq!(support::get_tg_font_size(20.0), 1.0);
    assert_eq!(support::get_tg_font_size(49.9), 1.0);
    assert_eq!(support::get_tg_font_size(50.0), 2.0);
    assert_eq!(support::get_tg_font_size(250.0), 2.0);
    assert_eq!(support::get_tg_font_size(250.1), 3.0);
}

#[test]
fn clamp_size_scales_both_limits_with_dpi() {
    assert_eq!(support::clamp_size(1000.0, 1.0), 100.0);
    assert_eq!(support::clamp_size(1000.0, 2.0), 200.0);
    assert_eq!(support::clamp_size(0.1, 1.0), 2.5);
    assert_eq!(support::clamp_size(0.1, 2.0), 5.0);
    assert_eq!(support::clamp_size(40.0, 2.0), 40.0);
}

#[test]
fn normalized_angles_stay_in_a_turn() {
    assert_eq!(support::normalize_angle(-90.0), 270.0);
    assert_eq!(support::normalize_angle(450.0), 90.0);
    assert_eq!(support::normalize_angle(0.0), 0.0);
}

#[test]
fn side_is_left_right_or_collinear() {
    assert_eq!(support::side(0.0, 0.0, 10.0, 0.0, 5.0, 5.0), 0);
    assert_eq!(support::side(0.0, 0.0, 10.0, 0.0, 5.0, -5.0), 1);
    assert_eq!(support::side(0.0, 0.0, 10.0, 0.0, 20.0, 0.0), 2);
}

#[test]
fn block_replaces_the_third_point_with_the_midpoint() {
    let mut p = pts_from(&[(0.0, 0.0), (10.0, 0.0), (5.0, 5.0)]);
    target::get_dism_block_double2(&mut p, lt::BLOCK).unwrap();
    assert_eq!((p[0].style, p[1].style, p[2].style), (0, 5, 14));
    assert_eq!((p[2].x, p[2].y), (5.0, 0.0));
    assert_eq!((p[3].x, p[3].y), (5.0, 5.0));
}

#[test]
fn deceive_closes_the_triangle() {
    let mut p = pts_from(&[(0.0, 0.0), (10.0, 0.0), (5.0, 5.0)]);
    disrupt::get_dism_deceive_double(&mut p).unwrap();
    let styles: Vec<i32> = p.iter().take(4).map(|q| q.style).collect();
    assert_eq!(styles, [1, 5, 1, 5]);
    assert_eq!((p[3].x, p[3].y), (0.0, 0.0));
}

#[test]
fn open_rectangle_runs_from_the_second_point_back_to_the_first() {
    let p = pts_from(&[(0.0, 0.0), (10.0, 0.0), (5.0, 8.0)]);
    let r = support::draw_open_rectangle_double(&p).unwrap();
    let xy: Vec<(f64, f64)> = r.iter().map(|q| (q.x, q.y)).collect();
    assert_eq!(xy, [(10.0, 0.0), (10.0, 8.0), (0.0, 8.0), (0.0, 0.0)]);
    let styles: Vec<i32> = r.iter().map(|q| q.style).collect();
    assert_eq!(styles, [0, 0, 0, 5]);
}

#[test]
fn bypass_draws_twelve_points() {
    let mut p = pts_from(&[(0.0, 0.0), (0.0, 100.0), (50.0, 50.0)]);
    let n = bypass::get_dism_bypass_double(&mut p, &Settings::default()).unwrap();
    assert_eq!(n, 12);
}

#[test]
fn escort_keeps_the_line_whole_when_the_icon_does_not_fit() {
    let mut p = pts_from(&[(0.0, 20.0), (0.0, 0.0), (30.0, 0.0)]);
    let n = escort::get_dism_escort_double(&Tg::new(&Settings::default()), &mut p).unwrap();
    assert_eq!(n, 6);
    assert_eq!((p[2].x, p[2].y), (p[3].x, p[3].y));
    assert_eq!((p[2].style, p[3].style), (0, 0));
}

#[test]
fn escort_leaves_a_gap_for_the_icon_on_a_long_line() {
    let mut p = pts_from(&[(0.0, 20.0), (0.0, 0.0), (300.0, 0.0)]);
    let n = escort::get_dism_escort_double(&Tg::new(&Settings::default()), &mut p).unwrap();
    assert_eq!(n, 6);
    assert_eq!((p[2].style, p[3].style), (5, 0));
    // The gap is icon/2 + font size on each side of the centre (150, 20).
    assert!((p[3].x - p[2].x - 2.0 * (25.0 + 12.0)).abs() < 1e-9);
}

#[test]
fn feint_indicator_peaks_over_the_chord() {
    let tg = Tg::new(&Settings::default());
    let shape = escort::get_fdi_shape(&tg, Pt::new(0.0, 0.0), Pt::new(10.0, 0.0));
    assert_eq!(shape.style, 1);
    match shape.path.as_slice() {
        [
            PathOp::MoveTo(ax, ay),
            PathOp::LineTo(bx, by),
            PathOp::LineTo(cx, cy),
        ] => {
            assert_eq!((*ax, *ay, *cx, *cy), (0.0, 0.0, 10.0, 0.0));
            assert!(close(*bx, 5.0));
            assert!(close(by.abs(), 5.0));
        }
        other => panic!("unexpected path {other:?}"),
    }
}

#[test]
fn arrow_feint_indicator_keeps_a_gap_of_one_and_a_half_widths() {
    let mut tg = Tg::new(&Settings::default());
    tg.line_thickness = 3;
    let a = Pt::new(0.0, 0.0);
    let c = Pt::new(2.0, 0.0);
    let b = Pt::new(1.0, -1.0);
    let shape = escort::get_fdi_shape_arrow(&tg, a, b, c);
    let min_gap = f64::from(tg.line_thickness) * 1.5;
    match shape.path.as_slice() {
        [
            PathOp::MoveTo(ax, _),
            PathOp::LineTo(_, _),
            PathOp::LineTo(cx, _),
        ] => {
            assert!(close(*cx - 2.0, min_gap));
            assert!(close(-*ax, min_gap));
        }
        other => panic!("unexpected path {other:?}"),
    }
}

#[test]
fn infiltration_reserves_three_slots_for_the_arrowhead() {
    let p = pts_from(&[(0.0, 0.0), (100.0, 0.0), (200.0, 80.0)]);
    let out = delay::get_infiltration_double(&p, &Settings::default()).unwrap();
    assert_eq!(out[0].style, 0);
    assert!(out[1..out.len() - 3].iter().all(|q| q.style == 1));
    assert!(out[out.len() - 3..].iter().all(|q| *q == Pt::default()));
}

#[test]
fn short_input_fails_instead_of_drawing_garbage() {
    let mut p = pts_from(&[(0.0, 0.0), (10.0, 0.0)]);
    assert!(bypass::get_dism_bypass_double(&mut p, &Settings::default()).is_err());
    let mut p = pts_from(&[(0.0, 0.0), (10.0, 0.0), (5.0, 5.0)]);
    assert!(rip::get_dism_rip_double(&mut p, lt::RIP, &Settings::default()).is_err());
}
