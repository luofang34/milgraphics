use super::*;

#[test]
fn access_outside_an_array_fails_like_java() {
    let pts = [Pt::new(1.0, 2.0)];
    assert_eq!(pts.at(0), Ok(Pt::new(1.0, 2.0)));
    assert!(matches!(
        pts.at(1),
        Err(EngineError::Index { index: 1, len: 1 })
    ));
    assert!(idx(-1, 3).is_err());
    assert_eq!(idx(2, 3), Ok(2));
}

#[test]
fn rounding_matches_java() {
    assert_eq!(java_round(2.5), 3.0);
    assert_eq!(java_round(-2.5), -2.0);
    assert_eq!(java_round(-2.6), -3.0);
}

#[test]
fn paths_split_into_polylines_at_moves() {
    let mut s = Shape::new(shape_type::POLYLINE);
    s.move_to(Pt::new(0.0, 0.0));
    s.line_to(Pt::new(1.0, 0.0));
    s.move_to(Pt::new(5.0, 5.0));
    s.line_to(Pt::new(6.0, 5.0));
    assert_eq!(
        s.polylines(),
        vec![vec![(0.0, 0.0), (1.0, 0.0)], vec![(5.0, 5.0), (6.0, 5.0)]]
    );
}
