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
