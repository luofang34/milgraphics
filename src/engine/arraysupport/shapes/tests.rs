use super::*;
use crate::engine::base::PathOp;

#[test]
fn add_polyline_splits_runs_and_skips_doubled_ends() {
    let pts = [
        Pt::styled(0.0, 0.0, 0),
        Pt::styled(1.0, 0.0, 5),
        Pt::styled(2.0, 0.0, 5),
        Pt::styled(3.0, 0.0, 0),
    ];
    let mut shapes = Vec::new();
    add_polyline(&pts, 4, &mut shapes).unwrap();
    assert_eq!(shapes.len(), 1);
    assert_eq!(
        shapes[0].path,
        vec![
            PathOp::MoveTo(0.0, 0.0),
            PathOp::LineTo(1.0, 0.0),
            PathOp::MoveTo(3.0, 0.0)
        ]
    );
}
