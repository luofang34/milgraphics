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

#[test]
fn an_anchorage_line_carries_the_anchor_halfway_along() {
    // Two equal legs: halfway is the middle point.
    let points = [(20.0, 50.0), (20.02, 50.0), (20.04, 50.0)];
    let c = construct(
        &def("15004600001203050000", &points, None),
        &Config::default(),
    );
    let c = c.unwrap();
    let [s] = c.symbols.as_slice() else {
        panic!("one symbol: {:?}", c.symbols)
    };
    assert_eq!(s.symbol.as_str(), "15004600001203040000");
    assert!((s.anchor.lon() - 20.02).abs() < 1e-6 && (s.anchor.lat() - 50.0).abs() < 1e-6);
}

#[test]
fn an_anchorage_area_carries_the_anchor_at_its_centre_only_in_2525e() {
    let points = [(20.0, 50.0), (20.02, 50.0), (20.02, 50.02), (20.0, 50.02)];
    let e = construct(
        &def("15004600001203060000", &points, None),
        &Config::default(),
    )
    .unwrap();
    let [s] = e.symbols.as_slice() else {
        panic!("one symbol: {:?}", e.symbols)
    };
    assert_eq!(s.symbol.as_str(), "15004600001203040000");
    let d = construct(
        &def("11004600001203060000", &points, None),
        &Config::default(),
    )
    .unwrap();
    assert!(d.symbols.is_empty());
}

#[test]
fn a_mined_area_shows_its_mine_types_in_a_row_from_version_15() {
    let area = [(20.0, 50.0), (20.04, 50.0), (20.04, 50.02), (20.0, 50.02)];
    // Sector 1 code 21: antipersonnel and antitank mines.
    let c = construct(
        &def("15032500002708002100", &area, None),
        &Config::default(),
    )
    .unwrap();
    let codes: Vec<&str> = c.symbols.iter().map(|s| s.symbol.as_str()).collect();
    assert_eq!(codes, ["15032500002802000000", "15032500002803000000"]);
    let [a, b] = c.symbols.as_slice() else {
        panic!("two mines")
    };
    assert_eq!(a.anchor, b.anchor);
    assert_eq!(a.offset_px[0], -b.offset_px[0]);
    assert!(a.offset_px[0] < 0.0);
    let d = construct(
        &def("11032500002708002100", &area, None),
        &Config::default(),
    )
    .unwrap();
    assert!(d.symbols.is_empty());
}

#[test]
fn a_mineline_puts_its_mines_halfway_along_it() {
    let line = [(20.0, 50.0), (20.02, 50.0), (20.06, 50.0)];
    let c = construct(
        &def("16032500002901010000", &line, None),
        &Config::default(),
    )
    .unwrap();
    assert_eq!(c.symbols.len(), 3);
    let at = c.symbols.first().unwrap().anchor;
    assert!((at.lon() - 20.03).abs() < 1e-6 && (at.lat() - 50.0).abs() < 1e-4);
}

#[test]
fn a_mine_cluster_takes_its_place_in_the_row_as_a_dashed_figure() {
    let area = [(20.0, 50.0), (20.04, 50.0), (20.04, 50.02), (20.0, 50.02)];
    // Sector 1 code 24: antipersonnel mine and mine cluster.
    let c = construct(
        &def("15032500002708002400", &area, None),
        &Config::default(),
    )
    .unwrap();
    let [mine] = c.symbols.as_slice() else {
        panic!("one mine: {:?}", c.symbols)
    };
    assert_eq!(mine.symbol.as_str(), "15032500002802000000");
    let glyphs: Vec<_> = c
        .decorations
        .iter()
        .filter_map(|d| match &d.0 {
            crate::construction::Decoration::Glyph {
                anchor,
                offset_px,
                stroke,
                ..
            } => Some((*anchor, *offset_px, *stroke)),
            _ => None,
        })
        .collect();
    let [(anchor, offset, stroke)] = glyphs.as_slice() else {
        panic!("one cluster: {glyphs:?}")
    };
    assert_eq!(*anchor, mine.anchor);
    assert_eq!(offset[0], -mine.offset_px[0]);
    assert_eq!(stroke.dash, crate::style::DashPattern::Dashed);
}
