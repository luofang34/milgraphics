use super::*;
use crate::definition::{ControlPoint, GraphicId};
use crate::geo::GeoPoint;
use crate::sidc::SymbolId;

const STORED: &str = r#"{ "schema": 1, "id": "nai-7", "symbol": "15032500001202000000",
  "points": [{"lon": 20.0, "lat": 50.0}, {"lon": 20.08, "lat": 50.0}, {"lon": 20.08, "lat": 50.05}],
  "modifiers": {"T": "1", "Z9": [1.10, "x"]}, "revision": 4, "owner": {"cell": "G3"} }"#;

#[test]
fn stored_bytes_are_kept_exactly() {
    let stored = PersistedGraphic::from_json(STORED).unwrap();
    assert_eq!(stored.as_json(), STORED);
}

#[test]
fn unknown_schema_is_kept_and_reported() {
    let future = r#"{"schema":2,"shape":{"kind":"spline"}}"#;
    let stored = PersistedGraphic::from_json(future).unwrap();
    assert!(matches!(
        stored.decode(),
        Err(PersistError::UnknownSchema { found: Some(_) })
    ));
    assert_eq!(stored.as_json(), future);
    let missing = PersistedGraphic::from_json(r#"{"id":"a"}"#).unwrap();
    assert!(matches!(
        missing.decode(),
        Err(PersistError::UnknownSchema { found: None })
    ));
}

#[test]
fn non_objects_are_rejected() {
    assert!(matches!(
        PersistedGraphic::from_json("[1]"),
        Err(PersistError::WrongType { found: "array" })
    ));
    assert!(PersistedGraphic::from_json("{").is_err());
}

#[test]
fn unknown_content_survives_decode_edit_encode() {
    let stored = PersistedGraphic::from_json(STORED).unwrap();
    let mut def = stored.decode().unwrap();
    def.points
        .push(ControlPoint::ground(GeoPoint::new(20.0, 50.05).unwrap()));
    def.revision = def.revision.wrapping_add(1);
    let written = PersistedGraphic::from_definition(&def).unwrap();
    let before: Value = serde_json::from_str(STORED).unwrap();
    let after: Value = serde_json::from_str(written.as_json()).unwrap();
    assert_eq!(after["owner"], before["owner"]);
    assert_eq!(after["modifiers"]["Z9"], before["modifiers"]["Z9"]);
    assert_eq!(after["schema"], 1);
    assert_eq!(after["revision"], 5);
    assert_eq!(written.decode().unwrap(), def);
}

#[test]
fn invalid_known_fields_are_reported_not_dropped() {
    let bad = r#"{"schema":1,"id":"a","symbol":"123","points":[]}"#;
    let stored = PersistedGraphic::from_json(bad).unwrap();
    assert!(matches!(stored.decode(), Err(PersistError::Invalid(_))));
    assert_eq!(stored.as_json(), bad);
}

#[test]
fn fresh_definitions_encode_with_the_schema_version() {
    let def = GraphicDefinition::new(
        GraphicId::new("x").unwrap(),
        SymbolId::parse("11032500001403000000").unwrap(),
        vec![],
    );
    let written = PersistedGraphic::from_definition(&def).unwrap();
    assert!(written.as_json().contains(r#""schema":1"#));
    assert_eq!(written.decode().unwrap(), def);
}
