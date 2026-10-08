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
