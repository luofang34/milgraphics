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

#[test]
fn malformed_json_is_reported_as_invalid_json() {
    assert!(matches!(
        PersistedGraphic::from_json("{"),
        Err(PersistError::InvalidJson(_))
    ));
}

#[test]
fn an_unreadable_colour_makes_the_graphic_undecodable_not_recoloured() {
    let bad = r##"{"schema":1,"id":"a","symbol":"11032500001403000000",
        "points":[{"lon":20.0,"lat":50.0},{"lon":20.1,"lat":50.0}],"style":{"line_color":"green"}}"##;
    let stored = PersistedGraphic::from_json(bad).unwrap();
    assert!(matches!(stored.decode(), Err(PersistError::Invalid(_))));
    assert_eq!(stored.as_json(), bad);
}

#[test]
fn colours_keep_their_stored_form() {
    for color in ["#112233", "#11223340"] {
        let json = format!(
            r#"{{"id":"a","points":[],"revision":0,"schema":1,"style":{{"fill_color":"{color}","line_color":"{color}"}},"symbol":"11032500001403000000"}}"#
        );
        let def = PersistedGraphic::from_json(&json)
            .unwrap()
            .decode()
            .unwrap();
        let written = PersistedGraphic::from_definition(&def).unwrap();
        assert_eq!(written.as_json(), json);
    }
}

#[test]
fn non_finite_numbers_are_refused_not_written_as_null() {
    use crate::geo::{Altitude, VerticalDatum};
    use crate::modifier::{ModifierField, ModifierValue};
    let base = GraphicDefinition::new(
        GraphicId::new("x").unwrap(),
        SymbolId::parse("11032500002422000000").unwrap(),
        vec![ControlPoint::ground(GeoPoint::new(20.0, 50.0).unwrap())],
    );
    let mut numbers = base.clone();
    numbers
        .modifiers
        .set(
            ModifierField::AM,
            ModifierValue::Numbers(vec![1.0, f64::NAN]),
        )
        .unwrap();
    let mut altitudes = base.clone();
    let infinite = Altitude::new(f64::INFINITY, VerticalDatum::MeanSeaLevel);
    altitudes
        .modifiers
        .set(ModifierField::X, ModifierValue::Altitudes(vec![infinite]))
        .unwrap();
    let mut point = base;
    point.points[0].altitude = Some(Altitude::new(f64::NEG_INFINITY, VerticalDatum::AboveGround));
    for (def, at) in [
        (numbers, "AM"),
        (altitudes, "X"),
        (point, "points[0].altitude"),
    ] {
        match PersistedGraphic::from_definition(&def) {
            Err(PersistError::NonFinite { id, field }) => {
                assert_eq!((id.as_str(), field.as_str()), ("x", at));
            }
            other => panic!("{at}: {other:?}"),
        }
    }
}
