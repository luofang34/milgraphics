use super::*;

fn pt(x: f64, y: f64) -> ScreenPoint {
    ScreenPoint { x, y }
}

#[test]
fn a_box_on_a_line_splits_it_in_two() {
    let line = [pt(0.0, 0.0), pt(10.0, 0.0), pt(20.0, 0.0)];
    let gap = [pt(8.0, -1.0), pt(12.0, -1.0), pt(12.0, 1.0), pt(8.0, 1.0)];
    let got = pieces(&line, &[gap]);
    assert_eq!(
        got,
        vec![
            vec![pt(0.0, 0.0), pt(8.0, 0.0)],
            vec![pt(12.0, 0.0), pt(20.0, 0.0)]
        ]
    );
}

#[test]
fn a_line_clear_of_every_box_stays_whole() {
    let line = [pt(0.0, 0.0), pt(10.0, 0.0), pt(20.0, 0.0)];
    let away = [pt(8.0, 5.0), pt(12.0, 5.0), pt(12.0, 9.0), pt(8.0, 9.0)];
    assert_eq!(pieces(&line, &[away]), vec![line.to_vec()]);
}
