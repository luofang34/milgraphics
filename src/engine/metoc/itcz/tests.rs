use super::*;

fn line(points: &[(f64, f64)]) -> Vec<Pt> {
    points.iter().map(|&(x, y)| Pt::new(x, y)).collect()
}

#[test]
fn rails_run_either_side_of_the_line() {
    let shapes = shapes(&line(&[(0.0, 0.0), (200.0, 0.0)]), 1.0);
    let [left, right, _rungs] = shapes.as_slice() else {
        panic!("two rails and the rungs: {shapes:?}")
    };
    let ys = |s: &Shape| s.points().iter().map(|p| p.y).collect::<Vec<_>>();
    assert_eq!(ys(left), [-HALF_WIDTH, -HALF_WIDTH]);
    assert_eq!(ys(right), [HALF_WIDTH, HALF_WIDTH]);
}

#[test]
fn rungs_come_in_groups_of_two_and_three() {
    let positions = rung_positions(&line(&[(0.0, 0.0), (200.0, 0.0)]), 1.0);
    let xs: Vec<f64> = positions.iter().map(|(p, _)| p.x).collect();
    // Half a gap in, two rungs; a gap on, three; a gap on, two.
    let mut expected = vec![
        15.0, 21.0, 51.0, 57.0, 63.0, 93.0, 99.0, 129.0, 135.0, 141.0,
    ];
    expected.extend([171.0, 177.0]);
    assert_eq!(xs.len(), expected.len());
    assert!(
        xs.iter().zip(&expected).all(|(a, b)| (a - b).abs() < 1e-9),
        "{xs:?}"
    );
    // Each rung crosses both rails.
    let all = shapes(&line(&[(0.0, 0.0), (200.0, 0.0)]), 1.0);
    let rungs = all.last().unwrap().polylines();
    assert_eq!(rungs.len(), xs.len());
    for r in &rungs {
        let ys: Vec<f64> = r.iter().map(|p| p.1).collect();
        assert_eq!(ys, [-HALF_WIDTH, HALF_WIDTH]);
    }
}

#[test]
fn a_corner_keeps_the_rails_parallel() {
    let rail = offset(&line(&[(0.0, 0.0), (100.0, 0.0), (100.0, 100.0)]), 8.0);
    let corner = &rail[1];
    // On the outside of the turn, 8 px from both legs.
    assert!(
        (corner.x - 108.0).abs() < 1e-9 && (corner.y + 8.0).abs() < 1e-9,
        "{corner:?}"
    );
}

#[test]
fn a_degenerate_line_draws_no_rungs() {
    assert!(rung_positions(&line(&[(5.0, 5.0), (5.0, 5.0)]), 1.0).is_empty());
    assert!(rung_positions(&line(&[(0.0, 0.0), (1.0e9, 0.0)]), 1.0).len() <= MAX_RUNGS);
}
