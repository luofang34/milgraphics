//! Hand-checkable placements: each expected value follows from reading
//! Modifier2.java with the 12 pixel font and a 7 pixel per character width.

use super::display::display_modifiers2;
use super::geo::add_modifiers_geo;
use super::layout::{remove_decimal, split_commas_for_tests};
use super::post::{SectorFrame, add_modifiers2};
use super::scale::scale_modifiers;
use super::{
    ABOVE_MIDDLE, ABOVE_START_INSIDE, AREA, JUSTIFY_CENTER, JUSTIFY_LEFT, JUSTIFY_RIGHT, TO_END,
    center_label::get_center_label,
};
use crate::engine::base::Pt;
use crate::engine::settings::Settings;
use crate::engine::tactical_lines as tl;
use crate::engine::tg::{ModifierLabel, Tg};

mod versions;

fn width(text: &str) -> f64 {
    7.0 * text.chars().count() as f64
}

fn graphic(line_type: i32, points: &[(f64, f64)]) -> Tg {
    let mut tg = Tg::new(&Settings::default());
    tg.line_type = line_type;
    tg.pixels = points.iter().map(|&(x, y)| Pt::new(x, y)).collect();
    tg
}

fn label(text: &str, kind: i32, line_factor: f64, path: [(f64, f64); 2]) -> ModifierLabel {
    ModifierLabel {
        text: text.to_owned(),
        kind,
        line_factor,
        text_path: [Pt::new(path[0].0, path[0].1), Pt::new(path[1].0, path[1].1)],
        ..ModifierLabel::default()
    }
}

#[test]
fn center_labels_follow_the_standard_version() {
    let mut tg = graphic(tl::PL, &[(0.0, 0.0)]);
    assert_eq!(get_center_label(&tg), "PL");
    tg.line_type = tl::BRDGHD;
    assert_eq!(get_center_label(&tg), "B");
    tg.symbol_id = "13".to_owned() + &"0".repeat(18);
    assert_eq!(get_center_label(&tg), "BL");
    tg.line_type = tl::ATI;
    assert_eq!(get_center_label(&tg), "ATI ZONE");
    tg.symbol_id = "15".to_owned() + &"0".repeat(18);
    assert_eq!(get_center_label(&tg), "ATIZ");
    tg.line_type = tl::UXO;
    assert_eq!(get_center_label(&tg), "UXO");
}

#[test]
fn phase_line_labels_both_ends_outside_the_line() {
    let mut tg = graphic(tl::PL, &[(0.0, 0.0), (100.0, 0.0), (200.0, 0.0)]);
    tg.t = "ALPHA".to_owned();
    add_modifiers_geo(&mut tg, &Settings::default(), &width).unwrap();
    assert_eq!(tg.modifiers.len(), 2);
    let placed = display_modifiers2(&tg, &width, false);
    let [first, last] = placed.as_slice() else {
        panic!("two labels expected");
    };
    assert_eq!(first.text, "PL ALPHA");
    // The first label hangs left of the first point, the last right of the
    // last point, each 6 pixels (half the font size) off the line.
    assert_eq!((first.x, first.y), (0.0, 0.0));
    assert_eq!(first.justify, JUSTIFY_RIGHT);
    assert_eq!(first.anchor_offset, (-6.0, 0.0));
    assert_eq!(first.angle_deg, 0.0);
    assert_eq!((last.x, last.y), (200.0, 0.0));
    assert_eq!(last.justify, JUSTIFY_LEFT);
    assert_eq!(last.anchor_offset, (6.0, 0.0));
    assert_eq!(last.angle_deg, 0.0);
    // Geometry points are restored.
    assert_eq!(tg.pixels.len(), 3);
}

#[test]
fn named_area_of_interest_label_is_centred_below_the_centre() {
    let mut tg = graphic(
        tl::NAI,
        &[(0.0, 0.0), (100.0, 0.0), (100.0, 100.0), (0.0, 100.0)],
    );
    tg.t = "X".to_owned();
    add_modifiers_geo(&mut tg, &Settings::default(), &width).unwrap();
    let placed = display_modifiers2(&tg, &width, false);
    let [only] = placed.as_slice() else {
        panic!("one label expected");
    };
    assert_eq!(only.text, "NAI X");
    // Upstream drops the baseline by half the font height.
    assert_eq!((only.x, only.y), (50.0, 56.0));
    assert_eq!(only.justify, JUSTIFY_CENTER);
    assert_eq!(only.anchor, (50.0, 50.0));
    assert_eq!(only.anchor_offset, (0.0, 6.0));
}

#[test]
fn groups_modifiers_when_asked() {
    let mut tg = graphic(
        tl::FFA,
        &[(0.0, 0.0), (100.0, 0.0), (100.0, 100.0), (0.0, 100.0)],
    );
    tg.t = "T".to_owned();
    tg.w = "D1".to_owned();
    let settings = Settings {
        group_modifiers: true,
        ..Settings::default()
    };
    add_modifiers_geo(&mut tg, &settings, &width).unwrap();
    let [only] = tg.modifiers.as_slice() else {
        panic!("one grouped label expected");
    };
    assert_eq!(only.text, "FFA\nT\nD1 -");
}

#[test]
fn above_middle_label_sits_above_the_path() {
    let mut tg = graphic(tl::PL, &[(0.0, 0.0), (1.0, 0.0)]);
    tg.modifiers
        .push(label("AB", ABOVE_MIDDLE, -1.0, [(0.0, 0.0), (100.0, 0.0)]));
    let [p] = display_modifiers2(&tg, &width, false).try_into().unwrap();
    assert_eq!(p.justify, JUSTIFY_CENTER);
    assert_eq!(p.anchor, (50.0, 0.0));
    assert_eq!((p.x, p.y), (50.0, -12.0));
    // Flipped text puts it on the other side.
    let [q] = display_modifiers2(&tg, &width, true).try_into().unwrap();
    assert_eq!((q.x, q.y), (50.0, 12.0));
}

#[test]
fn path_pointing_left_turns_the_text_upright() {
    let mut tg = graphic(tl::PL, &[(0.0, 0.0), (1.0, 0.0)]);
    tg.modifiers
        .push(label("AB", ABOVE_MIDDLE, 0.0, [(100.0, 100.0), (0.0, 0.0)]));
    let [p] = display_modifiers2(&tg, &width, false).try_into().unwrap();
    // atan2(-100, -100) is -135 degrees; upstream then subtracts a half
    // turn, which draws the same as 45 degrees.
    assert!((p.angle_deg + 315.0).abs() < 1e-9, "{}", p.angle_deg);
}

#[test]
fn inside_labels_move_by_the_width_plus_one() {
    let mut tg = graphic(tl::PL, &[(0.0, 0.0), (1.0, 0.0)]);
    tg.modifiers.push(label(
        "ABC",
        ABOVE_START_INSIDE,
        0.0,
        [(0.0, 0.0), (100.0, 0.0)],
    ));
    let [p] = display_modifiers2(&tg, &width, false).try_into().unwrap();
    // 3 characters at 7 pixels, plus the 1 upstream adds.
    assert_eq!((p.x, p.y), (22.0, 0.0));
    assert_eq!(p.justify, JUSTIFY_LEFT);
}

#[test]
fn area_label_uses_whole_pixel_arithmetic() {
    let mut tg = graphic(tl::PL, &[(0.0, 0.0), (1.0, 0.0)]);
    tg.modifiers
        .push(label("AB", AREA, 1.5, [(10.4, 20.6), (10.4, 20.6)]));
    let [p] = display_modifiers2(&tg, &width, false).try_into().unwrap();
    // Rounded to (10, 21); y = 21 + 6 + (int)(1.5 * 12).
    assert_eq!((p.x, p.y), (10.0, 45.0));
    assert_eq!(p.angle_deg, 0.0);
}

#[test]
fn labels_without_text_are_skipped() {
    let mut tg = graphic(tl::PL, &[(0.0, 0.0), (1.0, 0.0)]);
    tg.modifiers
        .push(label("", TO_END, 0.0, [(0.0, 0.0), (1.0, 0.0)]));
    assert!(display_modifiers2(&tg, &width, false).is_empty());
}

#[test]
fn small_areas_lose_the_labels_that_do_not_fit() {
    let mut tg = graphic(
        tl::NAI,
        &[(0.0, 0.0), (100.0, 0.0), (100.0, 24.0), (0.0, 24.0)],
    );
    for lf in [-1.5, -0.5, 0.5, 1.5] {
        tg.modifiers
            .push(label("L", AREA, lf, [(50.0, 12.0), (50.0, 12.0)]));
    }
    scale_modifiers(&mut tg, &Settings::default());
    // Half the area is 12 pixels; the labels were 18 tall. Each moves in by
    // 0.5 lines; those at 1.0 and 2.0 lines are past the edge.
    let factors: Vec<f64> = tg.modifiers.iter().map(|m| m.line_factor).collect();
    assert_eq!(factors, vec![-1.0, 0.0, 1.0]);
    let ellipsis = tg.modifiers.last().unwrap();
    assert_eq!(ellipsis.text, "\u{25cf}\u{25cf}\u{25cf}");
    assert_eq!(ellipsis.kind, AREA);
}

#[test]
fn tall_areas_spread_their_labels() {
    let mut tg = graphic(
        tl::NAI,
        &[(0.0, 0.0), (100.0, 0.0), (100.0, 480.0), (0.0, 480.0)],
    );
    tg.modifiers
        .push(label("L", AREA, -1.0, [(50.0, 240.0), (50.0, 240.0)]));
    scale_modifiers(&mut tg, &Settings::default());
    // Half the area is 240 pixels, the label 12: the factor is capped at 2.
    assert_eq!(tg.modifiers.first().unwrap().line_factor, -2.0);
}

#[test]
fn fix_task_label_goes_on_the_first_segment() {
    let mut tg = graphic(tl::FIX, &[(0.0, 0.0), (60.0, 0.0), (60.0, 10.0)]);
    let frame = SectorFrame {
        meters_per_pixel: 1.0,
        origin: Pt::new(0.0, 0.0),
    };
    add_modifiers2(&mut tg, &Settings::default(), &width, frame).unwrap();
    let [only] = tg.modifiers.as_slice() else {
        panic!("one label expected");
    };
    assert_eq!(only.text, "F");
    assert_eq!(only.kind, ABOVE_MIDDLE);
    assert_eq!(only.line_factor, -0.125);
    assert!(only.is_integral);
}

#[test]
fn range_fan_sector_labels_follow_the_scale() {
    // The orientation indicator runs north from (0, 0).
    let mut tg = graphic(
        tl::RANGE_FAN_SECTOR,
        &[
            (5.0, 5.0),
            (0.0, 0.0),
            (0.0, -10.0),
            (7.0, 7.0),
            (8.0, 8.0),
            (9.0, 9.0),
        ],
    );
    tg.am = "1000,2000".to_owned();
    tg.an = "0,90".to_owned();
    tg.x = "100".to_owned();
    let frame = SectorFrame {
        meters_per_pixel: 10.0,
        origin: Pt::new(0.0, 0.0),
    };
    add_modifiers2(&mut tg, &Settings::default(), &width, frame).unwrap();
    let texts: Vec<&str> = tg.modifiers.iter().map(|m| m.text.as_str()).collect();
    assert_eq!(texts, vec!["ALT 100", "RG 2000", "0", "90"]);
    // Mid radius 1500 m is 150 pixels north; the 90 degree edge is east.
    let at = |i: usize| tg.modifiers[i].text_path[0];
    assert!((at(0).x).abs() < 1e-9 && (at(0).y + 150.0).abs() < 1e-9);
    assert!((at(3).x - 150.0).abs() < 1e-9 && at(3).y.abs() < 1e-9);
}

#[test]
fn numbers_round_like_java() {
    assert_eq!(remove_decimal("1500.5 km").unwrap(), "1501 km");
    assert_eq!(remove_decimal("-0.5").unwrap(), "0");
    assert!(remove_decimal("abc").is_err());
    assert_eq!(split_commas_for_tests("a,b,,"), vec!["a", "b"]);
    assert_eq!(split_commas_for_tests(""), vec![""]);
    assert!(split_commas_for_tests(",").is_empty());
}
