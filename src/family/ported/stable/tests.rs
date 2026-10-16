use super::super::ORIGIN_PX as O;
use super::{chains, same};

#[test]
fn segments_whose_ends_stay_put_are_geographic() {
    // The second drawing is at three times the scale; its last vertex is an
    // arrowhead barb held at a pixel size.
    let a = [(O, O), (O + 10.0, O), (O + 20.0, O), (O + 18.0, O + 2.0)];
    let b = [(O, O), (O + 30.0, O), (O + 60.0, O), (O + 58.0, O + 2.0)];
    let found = same(&a, 3.0, &b, 1.0, 1e-6).unwrap();
    assert_eq!(found, [true, true, false]);
    let runs = chains(&found);
    assert_eq!((runs.len(), runs.first()), (1, Some(&(0..3))));
}

#[test]
fn drawings_with_other_vertices_are_not_matched() {
    assert_eq!(
        same(&[(0.0, 0.0), (1.0, 0.0)], 1.0, &[(0.0, 0.0)], 1.0, 1.0),
        None
    );
}

#[test]
fn repeated_vertices_are_no_segment() {
    let a = [(O, O), (O, O), (O + 10.0, O)];
    let b = [(O, O), (O, O), (O + 30.0, O)];
    assert_eq!(same(&a, 3.0, &b, 1.0, 1e-6).unwrap(), [false, true]);
}
