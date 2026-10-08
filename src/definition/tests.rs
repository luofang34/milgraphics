use super::*;
use crate::geo::VerticalDatum;

fn sample() -> GraphicDefinition {
    let mut def = GraphicDefinition::new(
        GraphicId::new("pl-1").unwrap(),
        SymbolId::parse("11032500001403000000").unwrap(),
        vec![
            ControlPoint::ground(GeoPoint::new(20.0, 50.0).unwrap()),
            ControlPoint {
                position: GeoPoint::new(20.1, 50.02).unwrap(),
                altitude: Some(Altitude {
                    metres: 10.0,
                    datum: VerticalDatum::MeanSeaLevel,
                }),
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
    let json = serde_json::to_string(&def).unwrap();
    assert_eq!(
        json,
        concat!(
            r#"{"id":"pl-1","symbol":"11032500001403000000","points":["#,
            r#"{"lat":50.0,"lon":20.0},"#,
            r#"{"altitude":{"datum":"msl","metres":10.0},"lat":50.02,"lon":20.1}],"#,
            r#""modifiers":{"T":"ALPHA","AM":[1000.0,5000.5]},"revision":0}"#
        )
    );
    assert_eq!(
        serde_json::from_str::<GraphicDefinition>(&json).unwrap(),
        def
    );
}

#[test]
fn unknown_fields_survive_at_every_level() {
    let json = r#"{"id":"a","symbol":"11032500001403000000",
        "points":[{"lon":1,"lat":2,"z_future":true}],
        "modifiers":{"T":"X","Q":"9"},"style":{"glow":3},"revision":7,"layer":"ops"}"#;
    let def: GraphicDefinition = serde_json::from_str(json).unwrap();
    assert_eq!(def.unknown["layer"], "ops");
    assert_eq!(def.points[0].unknown["z_future"], true);
    assert_eq!(def.modifiers.unknown["Q"], "9");
    assert_eq!(def.style.unknown["glow"], 3);
    let back: Value = serde_json::to_value(&def).unwrap();
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
