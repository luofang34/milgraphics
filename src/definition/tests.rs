use super::*;
use crate::geo::VerticalDatum;
use crate::persist::PersistedGraphic;

/// The definition as stored.
fn stored(def: &GraphicDefinition) -> String {
    PersistedGraphic::from_definition(def)
        .unwrap()
        .as_json()
        .to_owned()
}

/// A stored definition decoded.
fn decoded(json: &str) -> GraphicDefinition {
    PersistedGraphic::from_json(json).unwrap().decode().unwrap()
}

fn sample() -> GraphicDefinition {
    let mut def = GraphicDefinition::new(
        GraphicId::new("pl-1").unwrap(),
        SymbolId::parse("11032500001403000000").unwrap(),
        vec![
            ControlPoint::ground(GeoPoint::new(20.0, 50.0).unwrap()),
            ControlPoint {
                position: GeoPoint::new(20.1, 50.02).unwrap(),
                altitude: Some(Altitude::new(10.0, VerticalDatum::MeanSeaLevel)),
                unknown: BTreeMap::new(),
            },
        ],
    );
    def.modifiers.designation = Some("ALPHA".to_owned());
    def.modifiers.distances_m = vec![1000.0, 5000.5];
    def
}

#[test]
fn round_trips_through_json() {
    let def = sample();
    let json = stored(&def);
    assert_eq!(
        json,
        concat!(
            r#"{"id":"pl-1","modifiers":{"AM":[1000.0,5000.5],"T":"ALPHA"},"points":["#,
            r#"{"lat":50.0,"lon":20.0},"#,
            r#"{"altitude":{"datum":"msl","metres":10.0},"lat":50.02,"lon":20.1}],"#,
            r#""revision":0,"schema":1,"symbol":"11032500001403000000"}"#
        )
    );
    assert_eq!(decoded(&json), def);
}

#[test]
fn unknown_fields_survive_at_every_level() {
    let json = r#"{"schema":1,"id":"a","symbol":"11032500001403000000",
        "points":[{"lon":1,"lat":2,"z_future":true}],
        "modifiers":{"T":"X","Q":"9"},"style":{"glow":3},"revision":7,"layer":"ops"}"#;
    let def = decoded(json);
    assert_eq!(def.unknown["layer"], "ops");
    assert_eq!(def.points[0].unknown["z_future"], true);
    assert_eq!(def.modifiers.unknown["Q"], "9");
    assert_eq!(def.style.unknown["glow"], 3);
    let back: Value = serde_json::from_str(&stored(&def)).unwrap();
    assert_eq!(back["layer"], "ops");
    assert_eq!(back["modifiers"]["Q"], "9");
    assert_eq!(back["style"]["glow"], 3);
    assert_eq!(back["points"][0]["z_future"], true);
    assert_eq!(back["points"][0]["lon"], 1.0);
}

#[test]
fn ids_are_validated() {
    assert_eq!(GraphicId::new(""), Err(GraphicIdError::Empty));
    assert!(GraphicId::new("x".repeat(MAX_ID_BYTES)).is_ok());
    assert!(GraphicId::new("x".repeat(MAX_ID_BYTES + 1)).is_err());
    assert!(serde_json::from_str::<GraphicId>("\"\"").is_err());
}

#[test]
fn empty_designation_counts_as_absent() {
    let mut def = sample();
    def.modifiers.designation = Some(String::new());
    assert_eq!(def.modifiers.designation(), None);
}

#[test]
fn malformed_control_points_are_rejected() {
    for bad in [
        r#"{"lat":1}"#,
        r#"{"lon":"1","lat":1}"#,
        r#"{"lon":1,"lat":95}"#,
        r#"{"lon":1,"lat":2,"altitude":3}"#,
    ] {
        assert!(serde_json::from_str::<ControlPoint>(bad).is_err(), "{bad}");
    }
}
