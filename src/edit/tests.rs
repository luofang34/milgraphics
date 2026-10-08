use super::*;
use crate::definition::GraphicId;
use crate::sidc::SymbolId;

fn phase_line(points: &[(f64, f64)]) -> GraphicDefinition {
    GraphicDefinition::new(
        GraphicId::new("pl").unwrap(),
        SymbolId::parse("11032500001403000000").unwrap(),
        points
            .iter()
            .map(|&(lon, lat)| ControlPoint::ground(GeoPoint::new(lon, lat).unwrap()))
            .collect(),
    )
}

fn p(lon: f64, lat: f64) -> GeoPoint {
    GeoPoint::new(lon, lat).unwrap()
}

#[test]
fn moving_a_vertex_changes_only_that_point_and_the_revision() {
    let def = phase_line(&[(20.0, 50.0), (20.1, 50.0)]);
    let edited = apply_edit(
        &def,
        &Edit::Move {
            handle: HandleId::Vertex(1),
            to: p(20.2, 50.1),
        },
    )
    .unwrap();
    assert_eq!(edited.points[1].position, p(20.2, 50.1));
    assert_eq!(edited.points[0], def.points[0]);
    assert_eq!(edited.revision, def.revision + 1);
    assert_eq!(def.points[1].position, p(20.1, 50.0), "input untouched");
}

#[test]
fn insert_and_delete_respect_point_limits() {
    let def = phase_line(&[(20.0, 50.0), (20.1, 50.0)]);
    let three = apply_edit(
        &def,
        &Edit::InsertVertex {
            index: 1,
            at: p(20.05, 50.02),
        },
    )
    .unwrap();
    assert_eq!(three.points.len(), 3);
    assert_eq!(three.points[1].position, p(20.05, 50.02));
    let two = apply_edit(&three, &Edit::DeleteVertex { index: 1 }).unwrap();
    assert_eq!(two.points, def.points);
    assert_eq!(two.revision, 2);
    assert!(matches!(
        apply_edit(&two, &Edit::DeleteVertex { index: 0 }),
        Err(EditError::PointCount {
            count: 1,
            min: 2,
            ..
        })
    ));
}

#[test]
fn missing_handles_and_unsupported_symbols_are_refused() {
    let def = phase_line(&[(20.0, 50.0), (20.1, 50.0)]);
    for edit in [
        Edit::Move {
            handle: HandleId::Vertex(2),
            to: p(0.0, 0.0),
        },
        Edit::Move {
            handle: HandleId::Width,
            to: p(0.0, 0.0),
        },
        Edit::DeleteVertex { index: 5 },
        Edit::InsertVertex {
            index: 3,
            at: p(0.0, 0.0),
        },
    ] {
        assert!(
            matches!(apply_edit(&def, &edit), Err(EditError::NoSuchHandle { .. })),
            "{edit:?}"
        );
    }
    let mut other = def.clone();
    other.symbol = SymbolId::parse("11032500009999990000").unwrap();
    assert!(matches!(
        apply_edit(&other, &Edit::DeleteVertex { index: 0 }),
        Err(EditError::Unsupported(_))
    ));
}

#[test]
fn revisions_wrap_instead_of_overflowing() {
    let mut def = phase_line(&[(20.0, 50.0), (20.1, 50.0)]);
    def.revision = u64::MAX;
    let edited = apply_edit(
        &def,
        &Edit::Move {
            handle: HandleId::Vertex(0),
            to: p(20.0, 50.1),
        },
    )
    .unwrap();
    assert_eq!(edited.revision, 0);
}

fn graphic(sidc: &str, points: &[(f64, f64)]) -> GraphicDefinition {
    let mut d = phase_line(points);
    d.symbol = SymbolId::parse(sidc).unwrap();
    d
}

fn range_fan() -> GraphicDefinition {
    let mut d = graphic("11032500002422000000", &[(20.0, 50.0)]);
    d.modifiers.distances_m = vec![1000.0, 5000.0];
    d.modifiers.azimuths_deg = vec![30.0, 90.0];
    d
}

/// Applies `edit` and checks the input is unchanged whatever the outcome.
fn apply(def: &GraphicDefinition, edit: Edit) -> Result<GraphicDefinition, EditError> {
    let before = def.clone();
    let result = apply_edit(def, &edit);
    assert_eq!(def, &before, "apply_edit must not modify its input");
    result
}

#[test]
fn range_and_azimuth_handles_set_their_values_from_the_origin() {
    let earth = crate::geodesy::Earth::wgs84();
    let def = range_fan();
    let centre = def.points[0].position;
    let to = earth.direct(centre, 200.0, 7_500.0);
    let edited = apply(
        &def,
        Edit::Move {
            handle: HandleId::Range(1),
            to,
        },
    )
    .unwrap();
    assert!((edited.modifiers.distances_m[1] - 7_500.0).abs() < 1e-6);
    assert_eq!(edited.modifiers.distances_m[0], 1000.0);
    let edited = apply(
        &def,
        Edit::Move {
            handle: HandleId::Azimuth(0),
            to,
        },
    )
    .unwrap();
    assert!((edited.modifiers.azimuths_deg[0] - 200.0).abs() < 1e-9);
    let west = earth.direct(centre, -45.0, 3_000.0);
    let edited = apply(
        &def,
        Edit::Move {
            handle: HandleId::Azimuth(1),
            to: west,
        },
    )
    .unwrap();
    assert!(
        (edited.modifiers.azimuths_deg[1] - 315.0).abs() < 1e-9,
        "wrapped into [0, 360)"
    );
    assert!(matches!(
        apply(
            &def,
            Edit::Move {
                handle: HandleId::Range(2),
                to
            }
        ),
        Err(EditError::NoSuchHandle { .. })
    ));
}

#[test]
fn corridor_width_handle_sets_one_width() {
    let earth = crate::geodesy::Earth::wgs84();
    let mut def = graphic(
        "11032500001701000000",
        &[(20.0, 50.0), (20.1, 50.0), (20.2, 50.1)],
    );
    def.modifiers.distances_m = vec![1000.0, 2000.0, 3000.0];
    let to = earth.direct(def.points[0].position, 0.0, 750.0);
    let edited = apply(
        &def,
        Edit::Move {
            handle: HandleId::Width,
            to,
        },
    )
    .unwrap();
    assert_eq!(edited.modifiers.distances_m.len(), 1);
    assert!((edited.modifiers.distances_m[0] - 1_500.0).abs() < 1e-6);
}

#[test]
fn fixed_shape_symbols_refuse_vertex_insertion_and_deletion() {
    let mut axis = graphic(
        "11032500001514030000",
        &[(20.0, 50.0), (20.1, 50.03), (20.0, 50.01)],
    );
    let bypass = graphic(
        "11032500002706010000",
        &[(20.0, 50.0), (20.04, 50.03), (20.08, 50.0)],
    );
    for def in [&axis, &bypass, &range_fan()] {
        assert!(
            apply(
                def,
                Edit::InsertVertex {
                    index: 1,
                    at: p(20.0, 50.0)
                }
            )
            .is_err()
        );
        assert!(apply(def, Edit::DeleteVertex { index: 0 }).is_err());
    }
    // Moving the width point of an axis is a vertex move.
    axis.revision = 9;
    let moved = apply(
        &axis,
        Edit::Move {
            handle: HandleId::Vertex(2),
            to: p(20.0, 50.02),
        },
    )
    .unwrap();
    assert_eq!(
        (moved.points[2].position, moved.revision),
        (p(20.0, 50.02), 10)
    );
}

#[test]
fn handles_exist_only_where_the_family_draws_them() {
    let pl = phase_line(&[(20.0, 50.0), (20.1, 50.0)]);
    for handle in [HandleId::Width, HandleId::Range(0), HandleId::Azimuth(0)] {
        assert!(
            apply(
                &pl,
                Edit::Move {
                    handle,
                    to: p(20.0, 50.0)
                }
            )
            .is_err(),
            "{handle:?}"
        );
    }
    assert!(
        apply(
            &range_fan(),
            Edit::Move {
                handle: HandleId::Width,
                to: p(20.0, 50.0)
            }
        )
        .is_err()
    );
}

#[test]
fn edits_that_would_break_the_graphic_are_refused() {
    // Dragging the corridor width handle onto the first point means zero width.
    let mut corridor = graphic("11032500001701000000", &[(20.0, 50.0), (20.1, 50.0)]);
    corridor.modifiers.distances_m = vec![2000.0];
    let first = corridor.points[0].position;
    assert!(matches!(
        apply(
            &corridor,
            Edit::Move {
                handle: HandleId::Width,
                to: first
            }
        ),
        Err(EditError::Invalid(_))
    ));
    // Moving an axis width point onto its centreline leaves no width.
    let axis = graphic(
        "11032500001514030000",
        &[(20.0, 50.0), (20.1, 50.0), (20.0, 50.01)],
    );
    let on_axis = crate::geodesy::Earth::wgs84().interpolate(p(20.0, 50.0), p(20.1, 50.0), 0.5);
    assert!(matches!(
        apply(
            &axis,
            Edit::Move {
                handle: HandleId::Vertex(2),
                to: on_axis
            }
        ),
        Err(EditError::Invalid(_))
    ));
    // A range moved to zero leaves a fan with no extent.
    let fan = range_fan();
    let centre = fan.points[0].position;
    assert!(matches!(
        apply(
            &fan,
            Edit::Move {
                handle: HandleId::Range(1),
                to: centre
            }
        ),
        Err(EditError::Invalid(_))
    ));
}
