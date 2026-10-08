use super::clip::{MAX_LINE_PX, clip, limits_itself, near, runs};
use crate::engine::tactical_lines as tl;
use crate::render::{ScreenPoint, ScreenRect};

fn pt(x: f64, y: f64) -> ScreenPoint {
    ScreenPoint { x, y }
}

const BOX: ScreenRect = ScreenRect {
    min: ScreenPoint { x: 0.0, y: 0.0 },
    max: ScreenPoint { x: 100.0, y: 100.0 },
};

#[test]
fn a_line_leaving_and_reentering_is_cut_into_runs() {
    let line = [
        pt(-50.0, 50.0),
        pt(50.0, 50.0),
        pt(50.0, 200.0),
        pt(80.0, 200.0),
        pt(80.0, 50.0),
    ];
    let got = clip(&line, BOX);
    assert_eq!(
        got,
        vec![
            vec![pt(0.0, 50.0), pt(50.0, 50.0), pt(50.0, 100.0)],
            vec![pt(80.0, 100.0), pt(80.0, 50.0)],
        ]
    );
}

#[test]
fn a_line_missing_the_box_has_no_runs() {
    assert!(clip(&[pt(-10.0, -10.0), pt(-10.0, 500.0)], BOX).is_empty());
}

#[test]
fn short_lines_are_drawn_whole_without_a_viewport() {
    let line = vec![pt(-5_000.0, 0.0), pt(5_000.0, 0.0)];
    assert_eq!(runs((tl::SFENCE, false), line.clone(), None), vec![line]);
}

#[test]
fn long_repeating_lines_keep_only_the_part_near_the_viewport() {
    let line = vec![pt(-2.0 * MAX_LINE_PX, 50.0), pt(2.0 * MAX_LINE_PX, 50.0)];
    let got = runs((tl::SFENCE, false), line.clone(), near(Some(BOX)));
    let [run] = got.as_slice() else {
        panic!("{got:?}");
    };
    let at = |p: &ScreenPoint, x: f64| (p.x - x).abs() < 1e-6 && p.y == 50.0;
    assert!(
        run.len() == 2 && at(&run[0], -100.0) && at(&run[1], 200.0),
        "{run:?}"
    );
    assert!(runs((tl::SFENCE, false), line.clone(), None).is_empty());
    // An axis of advance cannot be cut: its last point sets its width.
    assert!(runs((tl::MAIN, false), line, near(Some(BOX))).is_empty());
}

#[test]
fn graphics_keeping_to_the_view_are_drawn_whole_when_it_is_known() {
    let line = vec![pt(-2.0 * MAX_LINE_PX, 50.0), pt(2.0 * MAX_LINE_PX, 50.0)];
    let sidc = |s: &str| crate::sidc::SymbolId::parse(s).unwrap();
    let zone = sidc("15032500002704000000");
    let wire_e = sidc("15032500002903010000");
    let wire_d = sidc("11032500002903010000");
    assert!(limits_itself(tl::OBSAREA, &zone) && limits_itself(tl::UNSP, &wire_e));
    // Version 11 wire is upstream's own marks, drawn along the whole line.
    assert!(!limits_itself(tl::UNSP, &wire_d));
    let whole = runs((tl::OBSAREA, true), line.clone(), near(Some(BOX)));
    assert_eq!(whole, vec![line.clone()]);
    assert!(runs((tl::OBSAREA, true), line, None).is_empty());
}
