use crate::construction::SymbolSize;
use crate::definition::{ControlPoint, GraphicDefinition, GraphicId};
use crate::family::{Config, ConstructError, construct};
use crate::geo::GeoPoint;
use crate::sidc::SymbolId;

fn def(sidc: &str, points: &[(f64, f64)], icon: Option<&str>) -> GraphicDefinition {
    let mut d = GraphicDefinition::new(
        GraphicId::new("g").unwrap(),
        SymbolId::parse(sidc).unwrap(),
        points
            .iter()
            .map(|&(lon, lat)| ControlPoint::ground(GeoPoint::new(lon, lat).unwrap()))
            .collect(),
    );
    d.modifiers.symbol_icon = icon.map(str::to_owned);
    d
}

const UNIT: &str = "15031000001211000000";

#[test]
fn a_seized_objective_puts_the_unit_in_its_circle() {
    let points = [(20.0, 50.0), (20.01, 50.0), (20.05, 50.02), (20.08, 49.99)];
    let c = construct(
        &def("15032500003423000000", &points, Some(UNIT)),
        &Config::default(),
    )
    .unwrap();
    let [s] = c.symbols.as_slice() else {
        panic!("one symbol: {:?}", c.symbols)
    };
    assert_eq!(s.symbol.as_str(), UNIT);
    assert_eq!(s.anchor, GeoPoint::new(20.0, 50.0).unwrap());
    assert!(matches!(s.size, SymbolSize::WithinCircle { .. }));
}

#[test]
fn cover_centres_the_unit_between_points_2_and_3() {
    let points = [(20.0, 50.0), (20.02, 50.0), (20.04, 50.0), (20.06, 50.0)];
    let c = construct(
        &def("15032500003422010000", &points, Some(UNIT)),
        &Config::default(),
    )
    .unwrap();
    let s = c.symbols.first().unwrap();
    assert!((s.anchor.lon() - 20.03).abs() < 1e-6);
    // Without amplifier A there is nothing to place.
    let none = construct(
        &def("15032500003422010000", &points, None),
        &Config::default(),
    )
    .unwrap();
    assert!(none.symbols.is_empty());
}

#[test]
fn a_contaminated_area_shows_its_event_symbol() {
    let points = [(20.0, 50.0), (20.04, 50.0), (20.04, 50.03), (20.0, 50.03)];
    let c = construct(
        &def("15062500002717000000", &points, None),
        &Config::default(),
    )
    .unwrap();
    let s = c.symbols.first().unwrap();
    // Same edition, context, identity and status; the Biological Event.
    assert_eq!(s.symbol.as_str(), "15062500002814000000");
}

#[test]
fn an_unreadable_unit_code_is_refused() {
    let points = [(20.0, 50.0), (20.01, 50.0), (20.05, 50.02), (20.08, 49.99)];
    let err = construct(
        &def("15032500003423000000", &points, Some("10XX")),
        &Config::default(),
    );
    assert!(matches!(err, Err(ConstructError::InvalidSymbolIcon { .. })));
}
