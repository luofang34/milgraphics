use super::*;
use crate::engine::api::draw;
use crate::engine::tactical_lines as tl;
use crate::modifier::Modifiers;
use crate::sidc::SymbolId;

fn out(code: &str, line_type: i32, points: &[(f64, f64)], modifiers: &Modifiers) -> Output {
    let symbol = SymbolId::parse(code).unwrap();
    let width = |t: &str| 7.0 * t.chars().count() as f64;
    draw(&Input {
        line_type,
        symbol: &symbol,
        pixels: points.iter().map(|&(x, y)| Pt::new(x, y)).collect(),
        modifiers,
        meters_per_pixel: 10.0,
        text_width: &width,
        ms_info: crate::family::ms_info(&symbol),
        style: crate::engine::api::Style::default(),
        visible: None,
    })
    .unwrap()
}

fn lines(o: &Output) -> Vec<Vec<(f64, f64)>> {
    o.shapes.iter().flat_map(Shape::polylines).collect()
}

fn dist(a: (f64, f64), b: (f64, f64)) -> f64 {
    (a.0 - b.0).hypot(a.1 - b.1)
}

const SQUARE: &[(f64, f64)] = &[
    (100.0, 100.0),
    (300.0, 100.0),
    (300.0, 300.0),
    (100.0, 300.0),
];
const LINE: &[(f64, f64)] = &[(100.0, 300.0), (300.0, 300.0)];

#[test]
fn the_fix_arrowhead_is_open_from_version_15_and_filled_before() {
    let none = Modifiers::default();
    let fix = [(500.0, 300.0), (200.0, 360.0)];
    let e = out("15032500002705030000", tl::MNFLDFIX, &fix, &none);
    assert!(e.shapes.iter().all(|s| s.fill_color.is_none()));
    assert!(
        lines(&e)
            .iter()
            .any(|l| l.len() == 3 && l.get(1) == Some(&(500.0, 300.0)))
    );
    let d = out("11032500002705030000", tl::MNFLDFIX, &fix, &none);
    assert!(d.shapes.iter().any(|s| s.shape_type == shape_type::FILL));
}

#[test]
fn the_version_16_control_fills_both_arrowheads() {
    let none = Modifiers::default();
    let points = [(300.0, 300.0), (450.0, 300.0)];
    let app6e = out("16032500003432000000", tl::CONTROL, &points, &none);
    let filled = app6e.shapes.iter().filter(|s| s.fill_color.is_some());
    assert_eq!(filled.count(), 2);
    let d = out("11032500003432000000", tl::CONTROL, &points, &none);
    assert!(d.shapes.iter().all(|s| s.fill_color.is_none()));
}

#[test]
fn the_version_16_control_sets_its_c_outside_the_circle() {
    let none = Modifiers::default();
    let points = [(300.0, 300.0), (450.0, 300.0)];
    let from_centre = |o: &Output| {
        let c = o.labels.iter().find(|l| l.text == "C").unwrap();
        dist((c.x, c.y), points[0])
    };
    let app6e = out("16032500003432000000", tl::CONTROL, &points, &none);
    assert!(from_centre(&app6e) > 150.0 + 6.0);
    let d = out("11032500003432000000", tl::CONTROL, &points, &none);
    assert!((from_centre(&d) - 150.0).abs() < 1.0);
}

#[test]
fn the_frontal_attack_bar_is_twice_the_arrowhead_base() {
    let none = Modifiers::default();
    let points = [
        (500.0, 300.0),
        (300.0, 320.0),
        (100.0, 330.0),
        (450.0, 280.0),
    ];
    // Lines 2 and 3 run from the tip and to it through the wings; line 4
    // is the bar.
    let measure = |o: &Output| {
        let l = lines(o);
        let base = dist(l[2][1], l[3][1]);
        let mid = ((l[4][0].0 + l[4][1].0) / 2.0, (l[4][0].1 + l[4][1].1) / 2.0);
        (dist(l[4][0], l[4][1]), base, dist(mid, l[2][0]))
    };
    let (bar, base, off_tip) = measure(&out(
        "15032500001527000000",
        tl::FRONTAL_ATTACK,
        &points,
        &none,
    ));
    assert!((bar - 2.0 * base).abs() < 1e-9);
    assert!(off_tip < base / 4.0);
    let (bar, base, _) = measure(&out(
        "11032500001527000000",
        tl::FRONTAL_ATTACK,
        &points,
        &none,
    ));
    assert!(bar < 1.5 * base);
}

#[test]
fn the_version_16_trip_wire_draws_the_glyph_instead_of_t() {
    let none = Modifiers::default();
    let app6e = out("16032500002905000000", tl::TRIP, LINE, &none);
    assert!(app6e.labels.iter().all(|l| l.text != "t"));
    let all = lines(&app6e);
    // The stem rises above point 1 (up is negative y) and the wire runs
    // back past it.
    assert!(all.iter().flatten().any(|p| p.0 == 100.0 && p.1 < 270.0));
    assert!(all.iter().flatten().any(|p| p.0 < 80.0 && p.1 == 300.0));
    let e = out("15032500002905000000", tl::TRIP, LINE, &none);
    assert!(e.labels.iter().any(|l| l.text == "t"));
    assert_eq!(lines(&e).len(), 1);
}

#[test]
fn the_decision_line_ends_on_a_star_holding_its_text() {
    let modifiers = Modifiers {
        designation: Some("1X".into()),
        country: Some("007".into()),
        ..Modifiers::default()
    };
    let o = out("16032500001105000000", tl::DECISION_LINE, LINE, &modifiers);
    let stars: Vec<_> = lines(&o).into_iter().filter(|l| l.len() == 11).collect();
    assert_eq!(stars.len(), 2);
    let texts: Vec<&str> = o.labels.iter().map(|l| l.text.as_str()).collect();
    assert_eq!(texts, ["1X/007", "1X/007"]);
    for ((star, &end), label) in stars.iter().zip(LINE).zip(&o.labels) {
        let on = star.windows(2).any(|w| {
            let (a, b) = (w[0], w[1]);
            (dist(a, end) + dist(end, b) - dist(a, b)).abs() < 1e-6
        });
        assert!(on, "{end:?} is not on its star");
        // The text is centred in the star, beyond the line's end.
        let centre_x = star.iter().map(|p| p.0).sum::<f64>() / star.len() as f64;
        assert!((label.x - centre_x).abs() < 1.0);
        assert!((label.x - end.0).abs() > 1.0);
    }
    let only_t = Modifiers {
        designation: Some("1X".into()),
        ..Modifiers::default()
    };
    let o = out("16032500001105000000", tl::DECISION_LINE, LINE, &only_t);
    let texts: Vec<&str> = o.labels.iter().map(|l| l.text.as_str()).collect();
    assert_eq!(texts, ["1X", "1X"]);
    let e = out("15032500001105000000", tl::DECISION_LINE, LINE, &modifiers);
    assert_eq!(lines(&e).len(), 1);
}

#[test]
fn unspecified_wire_spaces_its_marks() {
    let none = Modifiers::default();
    let o = out("15032500002903010000", tl::UNSP, LINE, &none);
    let all = lines(&o);
    assert!(all.iter().all(|arm| arm.len() == 2));
    let centres: Vec<f64> = all
        .chunks(2)
        .map(|arms| (arms[0][0].0 + arms[0][1].0) / 2.0)
        .collect();
    assert_eq!(centres.len(), (200.0 / wire::PITCH) as usize);
    assert!(
        centres
            .windows(2)
            .all(|w| (w[1] - w[0] - wire::PITCH).abs() < 1e-9)
    );
}

#[test]
fn a_wire_too_long_for_its_marks_spaces_them_along_its_whole_length() {
    let none = Modifiers::default();
    let far = 100.0 + 1_000.0 * wire::PITCH * wire::MAX_MARKS as f64;
    let o = out(
        "15032500002903010000",
        tl::UNSP,
        &[(100.0, 300.0), (far, 300.0)],
        &none,
    );
    let all = lines(&o);
    assert_eq!(all.len(), 2 * wire::MAX_MARKS);
    let last = all.last().map_or(0.0, |arm| (arm[0].0 + arm[1].0) / 2.0);
    assert!(far - last < 1_000.0 * wire::PITCH);
}

#[test]
fn labels_set_into_the_outline_are_marked_from_version_15() {
    let none = Modifiers::default();
    let e = out("15032500002405030000", tl::PAA, SQUARE, &none);
    let paa: Vec<_> = e.labels.iter().filter(|l| l.text == "PAA").collect();
    assert!(!paa.is_empty() && paa.iter().all(|l| l.knockout));
    let d = out("11032500002405030000", tl::PAA, SQUARE, &none);
    assert!(d.labels.iter().all(|l| !l.knockout));
}

#[test]
fn the_version_16_dynamic_minefield_puts_h_above_and_w_below() {
    let modifiers = Modifiers {
        additional_info: Some("HH".into()),
        dtg_start: Some("WW".into()),
        ..Modifiers::default()
    };
    let o = out("16032500002707070000", tl::DEPICT, SQUARE, &modifiers);
    let at = |t: &str| o.labels.iter().find(|l| l.text == t).map(|l| (l.x, l.y));
    let (h, w) = (at("HH").unwrap(), at("WW").unwrap());
    assert_eq!((h.0, w.0), (200.0, 200.0));
    assert!(h.1 < 100.0 && w.1 > 300.0);
    let e = out("15032500002707070000", tl::DEPICT, SQUARE, &modifiers);
    assert!(e.labels.iter().all(|l| l.text != "HH"));
}

/// Version 16 code `entity` drawn with the line type it is mapped to.
fn app6e(entity: u32, points: &[(f64, f64)], modifiers: &Modifiers) -> Output {
    let line_type = crate::engine::line_type::line_type(16, 25, entity).unwrap();
    out(
        &format!("1603250000{entity}0000"),
        line_type,
        points,
        modifiers,
    )
}

fn texts(o: &Output) -> Vec<&str> {
    o.labels.iter().map(|l| l.text.as_str()).collect()
}

fn full() -> Modifiers {
    Modifiers {
        designation: Some("T1X".into()),
        additional_info: Some("HH".into()),
        dtg_start: Some("W0".into()),
        dtg_end: Some("W1".into()),
        echelon: Some("XX".into()),
        ..Modifiers::default()
    }
}

#[test]
fn the_new_version_16_areas_carry_their_template_labels() {
    let t = Modifiers {
        designation: Some("T1X".into()),
        ..Modifiers::default()
    };
    assert_eq!(texts(&app6e(120_800, SQUARE, &t)), ["BA T1X"]);
    assert_eq!(texts(&app6e(242_600, SQUARE, &t)), ["ZF T1X"]);
    let h = Modifiers {
        additional_info: Some("HH".into()),
        ..Modifiers::default()
    };
    let ht = app6e(370_100, SQUARE, &h);
    assert_eq!(texts(&ht), ["HT", "HH"]);
    assert!((ht.labels[1].y - ht.labels[0].y - 12.0).abs() < 1e-9);
    // Version 15 has no such graphics.
    assert_eq!(crate::engine::line_type::line_type(15, 25, 120_800), None);
}

#[test]
fn the_artillery_areas_break_their_outline_under_the_label_at_four_sides() {
    for (entity, label) in [(242_400, "AMA"), (242_500, "ARA")] {
        let o = app6e(entity, SQUARE, &full());
        let on_line: Vec<_> = o.labels.iter().filter(|l| l.text == label).collect();
        assert_eq!(on_line.len(), 4);
        assert!(on_line.iter().all(|l| l.knockout));
        assert!(texts(&o).contains(&"T1X"));
    }
}

#[test]
fn restricted_terrain_is_hatched_and_severely_restricted_cross_hatched() {
    use crate::engine::render_utility::hatch::HATCH_FORWARD_DIAGONAL as FORWARD;
    let hatches = |o: &Output| -> Vec<i32> {
        o.shapes
            .iter()
            .filter_map(|s| s.pattern_fill.map(|h| h.style))
            .collect()
    };
    let restricted = app6e(152_400, SQUARE, &full());
    assert_eq!(hatches(&restricted).len(), 1);
    assert_eq!(texts(&restricted), ["HH"]);
    assert_eq!(
        (restricted.labels[0].x, restricted.labels[0].y),
        (200.0, 203.6)
    );
    let severe = app6e(152_500, SQUARE, &full());
    let styles = hatches(&severe);
    assert_eq!(styles.len(), 2);
    assert!(styles.contains(&FORWARD) && styles.iter().any(|&s| s != FORWARD));
    // The second hatch adds no outline.
    let outlined = severe
        .shapes
        .iter()
        .filter(|s| s.shape_type != shape_type::FILL);
    assert_eq!(outlined.count(), 1);
}

#[test]
fn psyops_zones_hold_a_loudspeaker_beside_h_over_t() {
    let circle = [(200.0, 200.0)];
    let mut m = full();
    m.distances_m = vec![1000.0];
    for (entity, points) in [
        (242_701, SQUARE),
        (242_702, &[(100.0, 200.0), (300.0, 200.0)][..]),
        (242_703, &circle[..]),
    ] {
        let o = app6e(entity, points, &m);
        let t = texts(&o);
        assert!(!t.contains(&"PKB"), "{entity}: {t:?}");
        let h = o.labels.iter().find(|l| l.text == "HH").unwrap();
        let name = o.labels.iter().find(|l| l.text == "T1X").unwrap();
        assert_eq!((h.justify, name.justify), (Justify::Left, Justify::Left));
        assert!(h.x == name.x && h.y < name.y);
        let speaker = o.shapes.last().unwrap();
        assert_eq!(speaker.shape_type, shape_type::FILL);
        // The loudspeaker ends left of the text and is centred between
        // its two lines.
        let xs = speaker
            .points()
            .iter()
            .map(|p| p.x)
            .fold(f64::MIN, f64::max);
        assert!(xs < h.x, "{entity}");
        assert!(t.iter().any(|l| l.starts_with("W0")), "{entity}: {t:?}");
    }
}

#[test]
fn the_avenue_of_approach_is_labelled_aa_with_h_and_n_beside_it() {
    let axis = [
        (400.0, 100.0),
        (300.0, 120.0),
        (100.0, 150.0),
        (380.0, 140.0),
    ];
    let o = app6e(152_300, &axis, &full());
    assert_eq!(texts(&o), ["AA T1X", "HH"]);
    let hostile = out(
        "16062500001523000000",
        crate::engine::line_type::line_type(16, 25, 152_300).unwrap(),
        &axis,
        &full(),
    );
    let eny: Vec<_> = hostile.labels.iter().filter(|l| l.text == "ENY").collect();
    assert_eq!(eny.len(), 2);
    // Near the rear (point N-1), one each side of the axis.
    assert!(eny.iter().all(|l| l.x < 150.0));
    assert!((eny[0].y - eny[1].y).abs() > 20.0);
    let h = hostile.labels.iter().find(|l| l.text == "HH").unwrap();
    assert!(eny.iter().all(|l| h.y < l.y));
}

#[test]
fn the_mobility_corridor_forks_both_ends_and_repeats_b_and_h_per_segment() {
    let line = [(100.0, 300.0), (300.0, 300.0), (400.0, 200.0)];
    let o = app6e(142_100, &line, &full());
    let b: Vec<_> = o.labels.iter().filter(|l| l.text == "XX").collect();
    let h: Vec<_> = o.labels.iter().filter(|l| l.text == "HH").collect();
    assert_eq!((b.len(), h.len()), (2, 2));
    assert!(b.iter().all(|l| l.knockout) && h.iter().all(|l| !l.knockout));
    // B in the middle of the first segment, H above it.
    assert!((b[0].x - 200.0).abs() < 1e-9 && h[0].y < b[0].y);
    // Two prongs at each end, opening away from the line.
    let prongs: Vec<_> = lines(&o).into_iter().filter(|l| l.len() == 2).collect();
    assert_eq!(prongs.len(), 4);
    assert!(
        prongs[..2]
            .iter()
            .all(|l| l[0] == (100.0, 300.0) && l[1].0 < 100.0)
    );
}

#[test]
fn the_rhumb_line_carries_an_along_it_and_t_boxed_across_it() {
    let m = Modifiers {
        designation: Some("15".into()),
        azimuths_deg: vec![60.0],
        ..Modifiers::default()
    };
    // Drawn left to right: its left is up on screen.
    let o = app6e(220_109, &[(100.0, 300.0), (300.0, 300.0)], &m);
    let an = o.labels.iter().find(|l| l.text == "060").unwrap();
    let t = o.labels.iter().find(|l| l.text == "15").unwrap();
    assert!(an.y < 300.0 && t.y > 300.0);
    assert_eq!(t.angle_deg, 0.0);
    let boxed = lines(&o).into_iter().find(|l| l.len() == 5).unwrap();
    let top = boxed.iter().map(|p| p.1).fold(f64::MAX, f64::min);
    assert!(top > 300.0 && top < 310.0, "{top}");
    assert!(boxed.iter().any(|p| p.0 < t.x) && boxed.iter().any(|p| p.0 > t.x));
    assert_eq!(rhumb::course_text(5.0), "005");
    assert_eq!(rhumb::course_text(45.5), "45.5");
}

#[test]
fn recover_is_drawn_as_evacuate_with_r() {
    let points = [
        (100.0, 200.0),
        (130.0, 200.0),
        (250.0, 150.0),
        (350.0, 200.0),
    ];
    let none = Modifiers::default();
    let recover = app6e(344_600, &points, &none);
    assert_eq!(texts(&recover), ["R"]);
    let evacuate = out("16032500003445000000", tl::EVACUATE, &points, &none);
    assert_eq!(lines(&recover), lines(&evacuate));
}
