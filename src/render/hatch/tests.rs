use super::*;
use crate::style::Rgba;

fn square(size: f64) -> Vec<ScreenPoint> {
    [(0.0, 0.0), (size, 0.0), (size, size), (0.0, size)]
        .into_iter()
        .map(|(x, y)| ScreenPoint { x, y })
        .collect()
}

#[test]
fn horizontal_lines_cross_a_square_at_the_spacing() {
    let lines = segments(&square(100.0), &Hatch::new(Rgba::BLACK, 0.0, 10.0, 1.0));
    // From the first vertex's line, every 10 px; a line along the far edge
    // is not inside.
    assert_eq!(lines.len(), 10);
    for [a, b] in &lines {
        assert!((a.y - b.y).abs() < 1e-9);
        assert!((a.x - b.x).abs() > 99.0);
    }
}

#[test]
fn rising_lines_stay_inside_and_keep_their_direction() {
    let ring = square(100.0);
    let lines = segments(&ring, &Hatch::new(Rgba::BLACK, 45.0, 10.0, 1.0));
    assert!(!lines.is_empty());
    for [a, b] in &lines {
        for p in [a, b] {
            assert!((-1e-9..=100.0 + 1e-9).contains(&p.x));
            assert!((-1e-9..=100.0 + 1e-9).contains(&p.y));
        }
        // Rising to the right on a y-down screen.
        assert!((b.x - a.x) * (b.y - a.y) < 0.0);
    }
}

#[test]
fn a_concave_area_is_cut_into_pieces() {
    // A U shape: a horizontal line through both arms has two segments.
    let u: Vec<ScreenPoint> = [
        (0.0, 0.0),
        (30.0, 0.0),
        (30.0, 70.0),
        (70.0, 70.0),
        (70.0, 0.0),
        (100.0, 0.0),
        (100.0, 100.0),
        (0.0, 100.0),
    ]
    .into_iter()
    .map(|(x, y)| ScreenPoint { x, y })
    .collect();
    let lines = segments(&u, &Hatch::new(Rgba::BLACK, 0.0, 25.0, 1.0));
    let at_25 = lines
        .iter()
        .filter(|[a, _]| (a.y - 25.0).abs() < 1e-9)
        .count();
    assert_eq!(at_25, 2);
}

#[test]
fn degenerate_input_draws_nothing() {
    let none = segments(
        &square(100.0)[..2],
        &Hatch::new(Rgba::BLACK, 0.0, 10.0, 1.0),
    );
    assert!(none.is_empty());
    let zero = segments(&square(100.0), &Hatch::new(Rgba::BLACK, 0.0, 0.0, 1.0));
    assert!(zero.is_empty());
}

#[test]
fn label_boxes_are_left_clear() {
    let line = [
        ScreenPoint { x: 0.0, y: 50.0 },
        ScreenPoint { x: 100.0, y: 50.0 },
    ];
    let label = [
        ScreenPoint { x: 40.0, y: 40.0 },
        ScreenPoint { x: 60.0, y: 40.0 },
        ScreenPoint { x: 60.0, y: 60.0 },
        ScreenPoint { x: 40.0, y: 60.0 },
    ];
    let pieces = outside(line, &[label]);
    assert_eq!(pieces.len(), 2);
    assert!((pieces[0][1].x - 40.0).abs() < 1e-9);
    assert!((pieces[1][0].x - 60.0).abs() < 1e-9);
    // A box the line misses leaves it whole.
    let away = label.map(|p| ScreenPoint {
        x: p.x,
        y: p.y + 100.0,
    });
    assert_eq!(outside(line, &[away]), vec![line]);
}
