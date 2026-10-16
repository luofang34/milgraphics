use super::{paths, strip};
use crate::family::ported_baseline::Edge;
use crate::render::ScreenPoint;

const EDGE: [ScreenPoint; 2] = [
    ScreenPoint { x: 0.0, y: 0.0 },
    ScreenPoint { x: 100.0, y: 0.0 },
];

#[test]
fn segments_on_an_edge_are_left_out_and_the_rest_kept() {
    let p = paths(&EDGE, &[(0, Edge::Straight)], std::iter::empty());
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
        strip(
            vec![line.clone()],
            paths(&EDGE, &[], std::iter::empty()).as_ref()
        ),
        [line]
    );
}

#[test]
fn a_segment_beside_the_edge_is_kept() {
    let line = vec![(0.0, 2.0), (100.0, 2.0)];
    let p = paths(&EDGE, &[(0, Edge::Straight)], std::iter::empty());
    assert_eq!(strip(vec![line.clone()], p.as_ref()), [line]);
}

#[test]
fn segments_on_a_chain_are_left_out() {
    let chain = vec![
        ScreenPoint { x: 0.0, y: 0.0 },
        ScreenPoint { x: 30.0, y: 0.0 },
        ScreenPoint { x: 60.0, y: 10.0 },
    ];
    let p = paths(&EDGE, &[], std::iter::once(chain));
    // The chain, then an arrowhead barb held at a pixel size.
    let line = vec![(0.0, 0.0), (30.0, 0.0), (60.0, 10.0), (55.0, 14.0)];
    assert_eq!(
        strip(vec![line], p.as_ref()),
        [vec![(60.0, 10.0), (55.0, 14.0)]]
    );
}

#[test]
fn a_closed_chain_strips_its_closing_segment() {
    let ring: Vec<ScreenPoint> = [
        (0.0, 0.0),
        (10.0, 0.0),
        (10.0, 10.0),
        (0.0, 10.0),
        (0.0, 0.0),
    ]
    .iter()
    .map(|&(x, y)| ScreenPoint { x, y })
    .collect();
    let p = paths(&EDGE, &[], std::iter::once(ring));
    let closing = vec![(0.0, 10.0), (0.0, 0.0)];
    assert!(strip(vec![closing], p.as_ref()).is_empty());
}
