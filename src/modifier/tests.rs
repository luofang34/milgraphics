use super::*;
use crate::geo::{Altitude, VerticalDatum};

#[test]
fn every_field_round_trips_through_get_and_set() {
    for &field in ModifierField::ALL {
        let mut m = Modifiers::default();
        let value = match field.kind() {
            ModifierKind::Distances | ModifierKind::Azimuths => {
                ModifierValue::Numbers(vec![1.0, 2.0])
            }
            ModifierKind::Direction => ModifierValue::Numbers(vec![45.0]),
            ModifierKind::Altitudes => {
                ModifierValue::Altitudes(vec![Altitude::new(10.0, VerticalDatum::MeanSeaLevel)])
            }
            _ => ModifierValue::Text("X1".into()),
        };
        m.set(field, value.clone()).unwrap();
        assert_eq!(m.get(field), Some(value), "{field}");
        assert!(m.is_set(field));
        let others = ModifierField::ALL.iter().copied().filter(|f| *f != field);
        assert!(
            others.clone().all(|f| !m.is_set(f)),
            "{field} set only itself"
        );
        m.clear(field);
        assert!(m.is_empty(), "{field} cleared");
    }
}

#[test]
fn values_of_the_wrong_shape_are_refused() {
    let mut m = Modifiers::default();
    assert!(matches!(
        m.set(ModifierField::T, ModifierValue::Numbers(vec![1.0])),
        Err(ModifierValueError::Kind {
            field: ModifierField::T,
            ..
        })
    ));
    assert!(matches!(
        m.set(ModifierField::Q, ModifierValue::Numbers(vec![1.0, 2.0])),
        Err(ModifierValueError::Count { count: 2, .. })
    ));
    assert!(
        m.set(ModifierField::AM, ModifierValue::Text("5".into()))
            .is_err()
    );
    assert!(m.is_empty(), "a refused value changes nothing");
}

#[test]
fn every_field_is_stored_under_its_letters() {
    for &field in ModifierField::ALL {
        let mut m = Modifiers::default();
        let value = match field.kind() {
            ModifierKind::Distances | ModifierKind::Azimuths | ModifierKind::Direction => {
                ModifierValue::Numbers(vec![3.0])
            }
            ModifierKind::Altitudes => {
                ModifierValue::Altitudes(vec![Altitude::new(1.0, VerticalDatum::AboveGround)])
            }
            _ => ModifierValue::Text("v".into()),
        };
        m.set(field, value).unwrap();
        let json = serde_json::to_value(&m).unwrap();
        let keys: Vec<&String> = json.as_object().unwrap().keys().collect();
        assert_eq!(keys, [field.name()], "{field}");
        let back: Modifiers = serde_json::from_value(json).unwrap();
        assert_eq!(back, m);
        assert_eq!(ModifierField::from_name(field.name()), Some(field));
    }
}

#[test]
fn empty_text_counts_as_unset() {
    let m = Modifiers {
        designation: Some(String::new()),
        ..Modifiers::default()
    };
    assert!(!m.is_set(ModifierField::T));
    assert_eq!(m.get(ModifierField::T), None);
}

#[test]
fn a_stored_value_of_the_wrong_shape_is_kept_not_fatal() {
    let json = r#"{"T":"ALPHA","Q":"north","AM":[1,"two"],"T1":""}"#;
    let m: Modifiers = serde_json::from_str(json).unwrap();
    assert_eq!(m.designation(), Some("ALPHA"));
    assert_eq!(m.unknown["Q"], "north");
    assert_eq!(m.unknown["AM"], serde_json::json!([1, "two"]));
    assert_eq!(m.direction_deg, None);
    let back = serde_json::to_value(&m).unwrap();
    let original: serde_json::Value = serde_json::from_str(json).unwrap();
    assert_eq!(
        back, original,
        "written back unchanged, empty text included"
    );
}

#[test]
fn every_field_has_a_distinct_label() {
    let mut labels: Vec<&str> = ModifierField::ALL.iter().map(|f| f.label()).collect();
    assert!(labels.iter().all(|l| !l.is_empty()));
    labels.sort_unstable();
    labels.dedup();
    assert_eq!(labels.len(), ModifierField::ALL.len());
    assert_eq!(ModifierField::T.label(), "Unique designation");
}
