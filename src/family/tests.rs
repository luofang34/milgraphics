use super::*;
use crate::construction::HandleKind;
use crate::definition::{ControlPoint, GraphicId};
use crate::geo::GeoPoint;
use crate::modifier::ModifierField;
use crate::sidc::SymbolId;

fn def(sidc: &str, points: &[(f64, f64)]) -> GraphicDefinition {
    GraphicDefinition::new(
        GraphicId::new("g").unwrap(),
        SymbolId::parse(sidc).unwrap(),
        points
            .iter()
            .map(|&(lon, lat)| ControlPoint::ground(GeoPoint::new(lon, lat).unwrap()))
            .collect(),
    )
}

fn handles(d: &GraphicDefinition) -> Vec<(HandleId, HandleKind)> {
    construct(d, &Config::default())
        .unwrap()
        .handles
        .iter()
        .map(|h| (h.id, h.kind))
        .collect()
}

#[test]
fn each_family_exposes_its_handles() {
    let mut fan = def("11032500002422000000", &[(20.0, 50.0)]);
    fan.modifiers.distances_m = vec![1000.0, 5000.0];
    fan.modifiers.azimuths_deg = vec![30.0, 90.0];
    assert_eq!(
        handles(&fan),
        [
            (HandleId::Vertex(0), HandleKind::Vertex),
            (HandleId::Range(0), HandleKind::Range),
            (HandleId::Range(1), HandleKind::Range),
            (HandleId::Azimuth(0), HandleKind::Azimuth),
            (HandleId::Azimuth(1), HandleKind::Azimuth),
        ]
    );
    let axis = def(
        "11032500001514030000",
        &[(20.0, 50.0), (20.1, 50.03), (20.0, 50.01)],
    );
    assert_eq!(
        handles(&axis).last(),
        Some(&(HandleId::Vertex(2), HandleKind::Width))
    );
    let mut corridor = def("11032500001701000000", &[(20.0, 50.0), (20.1, 50.0)]);
    corridor.modifiers.distances_m = vec![2000.0];
    assert!(handles(&corridor).contains(&(HandleId::Width, HandleKind::Width)));
}

#[test]
fn definitions_are_validated_before_construction() {
    let mut pl = def("11032500001403000000", &[(20.0, 50.0), (20.1, 50.0)]);
    pl.modifiers.distances_m = vec![5.0];
    assert!(matches!(
        construct(&pl, &Config::default()),
        Err(ConstructError::UnsupportedModifier {
            field: ModifierField::AM,
            ..
        })
    ));
    pl.modifiers.distances_m.clear();
    pl.modifiers
        .unknown
        .insert("Q9".to_owned(), serde_json::json!("x"));
    assert!(matches!(
        construct(&pl, &Config::default()),
        Err(ConstructError::UnknownModifier { key, .. }) if key == "Q9"
    ));
    let corridor = def("11032500001701000000", &[(20.0, 50.0), (20.1, 50.0)]);
    assert!(matches!(
        construct(&corridor, &Config::default()),
        Err(ConstructError::MissingModifier {
            field: ModifierField::AM,
            ..
        })
    ));
    let mut fan = def("11032500002422000000", &[(20.0, 50.0)]);
    fan.modifiers.distances_m = vec![f64::NAN];
    fan.modifiers.azimuths_deg = vec![0.0, 90.0];
    assert!(matches!(
        construct(&fan, &Config::default()),
        Err(ConstructError::InvalidModifier {
            field: ModifierField::AM,
            index: 0,
            ..
        })
    ));
    let short = def("11032500001202000000", &[(20.0, 50.0), (20.1, 50.0)]);
    assert!(matches!(
        construct(&short, &Config::default()),
        Err(ConstructError::PointCount {
            count: 2,
            min: 3,
            ..
        })
    ));
}

#[test]
fn budgets_bound_input_and_output() {
    let pl = def("11032500001403000000", &[(0.0, 0.0), (179.0, 0.0)]);
    let mut config = Config::default();
    config.budget.max_vertices = 100;
    assert!(matches!(
        construct(&pl, &config),
        Err(ConstructError::Budget(BudgetError::Vertices { .. }))
    ));
    let mut long = def("11032500001403000000", &[(20.0, 50.0), (20.1, 50.0)]);
    long.modifiers.designation = Some("x".repeat(257));
    assert!(matches!(
        construct(&long, &Config::default()),
        Err(ConstructError::Budget(BudgetError::Text {
            field: ModifierField::T,
            ..
        }))
    ));
}

#[test]
fn planned_status_dashes_lines_but_not_bypass_arrowheads() {
    use crate::construction::Decoration;
    use crate::style::DashPattern;
    let bypass = def(
        "11032510002706010000",
        &[(20.0, 50.0), (20.04, 50.03), (20.08, 50.0)],
    );
    let c = construct(&bypass, &Config::default()).unwrap();
    assert!(
        c.parts
            .iter()
            .all(|p| p.stroke.is_some_and(|s| s.dash == DashPattern::Dashed))
    );
    for d in &c.decorations {
        let Decoration::Arrowhead { stroke, filled, .. } = &d.0 else {
            panic!()
        };
        assert_eq!((stroke.dash, *filled), (DashPattern::Solid, true));
    }
}

#[test]
fn edges_are_geodesics_not_mercator_chords() {
    // A 100 km east-west phase line at 50°N: the geodesic bows poleward of
    // the straight Mercator line by about 233 m at its middle.
    let earth = Earth::wgs84();
    let a = GeoPoint::new(0.0, 50.0).unwrap();
    let b = earth.direct(a, 90.0, 100_000.0);
    let pl = def(
        "11032500001403000000",
        &[(a.lon(), a.lat()), (b.lon(), b.lat())],
    );
    let c = construct(&pl, &Config::default()).unwrap();
    let line = c.parts[0].geometry.points();
    assert!(
        line.len() >= 11,
        "100 km is densified to at most 10 km steps"
    );
    let total = earth.inverse(a, b).distance_m;
    for &v in line {
        let detour = earth.inverse(a, v).distance_m + earth.inverse(v, b).distance_m - total;
        assert!(
            detour < 1e-6,
            "vertex {v:?} is off the geodesic ({detour} m detour)"
        );
    }
    let middle = earth.interpolate(a, b, 0.5);
    let mercator = |lat: f64| {
        (std::f64::consts::FRAC_PI_4 + lat.to_radians() / 2.0)
            .tan()
            .ln()
    };
    let chord_lat = (2.0 * ((mercator(a.lat()) + mercator(b.lat())) / 2.0).exp().atan()
        - std::f64::consts::FRAC_PI_2)
        .to_degrees();
    let chord = GeoPoint::new((a.lon() + b.lon()) / 2.0, chord_lat).unwrap();
    let bow = earth.inverse(middle, chord).distance_m;
    assert!((bow - 233.1).abs() < 1.0, "bow {bow}");
    assert!(middle.lat() > chord.lat());
}

#[test]
fn control_point_altitudes_are_refused_not_ignored() {
    use crate::geo::{Altitude, VerticalDatum};
    let mut pl = def("11032500001403000000", &[(20.0, 50.0), (20.1, 50.0)]);
    let ground = construct(&pl, &Config::default());
    assert!(ground.is_ok());
    pl.points[1].altitude = Some(Altitude::new(1000.0, VerticalDatum::Ellipsoid));
    assert!(matches!(
        construct(&pl, &Config::default()),
        Err(ConstructError::UnsupportedAltitude { index: 1, .. })
    ));
}

#[test]
fn ported_graphics_take_the_operators_colour() {
    // Light Line, drawn by the ported renderer.
    let mut d = def("11032500001102000000", &[(20.0, 50.0), (20.1, 50.0)]);
    d.style.line_color = Some("#0000ff".to_owned());
    let c = construct(&d, &Config::default()).unwrap();
    let blue = crate::style::Rgba::opaque(0, 0, 255);
    assert!(
        c.parts
            .iter()
            .all(|p| p.stroke.is_some_and(|s| s.color == blue))
    );
    assert!(!c.parts.is_empty());
}

#[test]
fn ported_graphics_allow_vertex_edits_unless_their_point_count_is_fixed() {
    use crate::edit::{Edit, apply_edit};
    let line = def("11032500001102000000", &[(20.0, 50.0), (20.1, 50.0)]);
    let at = crate::geo::GeoPoint::new(20.05, 50.01).unwrap();
    assert!(apply_edit(&line, &Edit::InsertVertex { index: 1, at }).is_ok());
    // A bypass task takes exactly three points.
    let task = def(
        "11032500002705010000",
        &[(20.0, 50.0), (20.1, 50.0), (20.05, 50.05)],
    );
    assert!(apply_edit(&task, &Edit::InsertVertex { index: 1, at }).is_err());
}
