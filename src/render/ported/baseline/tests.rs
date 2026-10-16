use super::{edges, strip};
use crate::render::ScreenPoint;

#[test]
fn segments_on_an_edge_are_left_out_and_the_rest_kept() {
    let pixels = [
        ScreenPoint { x: 0.0, y: 0.0 },
        ScreenPoint { x: 100.0, y: 0.0 },
    ];
    let e = edges(&pixels, &[0]);
    // Along the edge, up a tooth and back, along the edge again.
    let line = vec![
        (0.0, 0.0),
        (40.0, 0.0),
        (45.0, -8.0),
        (50.0, 0.0),
        (100.0, 0.0),
    ];
    assert_eq!(
        strip(vec![line], &e),
        [vec![(40.0, 0.0), (45.0, -8.0), (50.0, 0.0)]]
    );
}

#[test]
fn without_edges_lines_are_unchanged() {
    let line = vec![(0.0, 0.0), (10.0, 0.0)];
    assert_eq!(strip(vec![line.clone()], &[]), [line]);
}

#[test]
fn a_segment_beside_the_edge_is_kept() {
    let pixels = [
        ScreenPoint { x: 0.0, y: 0.0 },
        ScreenPoint { x: 100.0, y: 0.0 },
    ];
    let line = vec![(0.0, 2.0), (100.0, 2.0)];
    assert_eq!(strip(vec![line.clone()], &edges(&pixels, &[0])), [line]);
}
