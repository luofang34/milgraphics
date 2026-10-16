use super::{paths, strip};
use crate::family::ported_baseline::Edge;
use crate::render::ScreenPoint;

const EDGE: [ScreenPoint; 2] = [
    ScreenPoint { x: 0.0, y: 0.0 },
    ScreenPoint { x: 100.0, y: 0.0 },
];

#[test]
fn segments_on_an_edge_are_left_out_and_the_rest_kept() {
    let p = paths(&EDGE, &[(0, Edge::Straight)]);
    // Along the edge, up a tooth and back, along the edge again.
    let line = vec![
        (0.0, 0.0),
        (40.0, 0.0),
        (45.0, -8.0),
        (50.0, 0.0),
        (100.0, 0.0),
    ];
    assert_eq!(
        strip(vec![line], p.as_ref()),
        [vec![(40.0, 0.0), (45.0, -8.0), (50.0, 0.0)]]
    );
}

#[test]
fn without_edges_lines_are_unchanged() {
    let line = vec![(0.0, 0.0), (10.0, 0.0)];
    assert_eq!(
        strip(vec![line.clone()], paths(&EDGE, &[]).as_ref()),
        [line]
    );
}

#[test]
fn a_segment_beside_the_edge_is_kept() {
    let line = vec![(0.0, 2.0), (100.0, 2.0)];
    let p = paths(&EDGE, &[(0, Edge::Straight)]);
    assert_eq!(strip(vec![line.clone()], p.as_ref()), [line]);
}
