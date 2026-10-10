//! The stored form of [`Modifiers`]: a JSON object keyed by field letters.
//!
//! A value whose shape does not fit its field (for example one written by a
//! version that stored it differently) is kept under its key in `unknown`,
//! so it is reported when the graphic is constructed and written back
//! unchanged, instead of making the whole definition unreadable.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use super::{ModifierField, ModifierKind, Modifiers};
use crate::geo::Altitude;

/// The stored JSON object, kept out of the public API so the conversion is
/// an implementation detail of serialization.
#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub(super) struct StoredModifiers(Map<String, Value>);

impl From<StoredModifiers> for Modifiers {
    fn from(StoredModifiers(mut map): StoredModifiers) -> Self {
        let mut m = Modifiers::default();
        for &field in ModifierField::ALL {
            let Some(value) = map.remove(field.name()) else {
                continue;
            };
            if let Err(value) = m.store(field, value) {
                m.unknown.insert(field.name().to_owned(), value);
            }
        }
        m.unknown.extend(map);
        m
    }
}

impl From<Modifiers> for StoredModifiers {
    fn from(m: Modifiers) -> Self {
        let mut map = Map::new();
        for &field in ModifierField::ALL {
            if let Some(value) = m.stored(field) {
                map.insert(field.name().to_owned(), value);
            }
        }
        map.extend(m.unknown);
        Self(map)
    }
}

impl Modifiers {
    /// Takes the stored `value` of `field`, or gives it back if its shape
    /// does not fit.
    fn store(&mut self, field: ModifierField, value: Value) -> Result<(), Value> {
        let numbers =
            |v: &Value| -> Option<Vec<f64>> { v.as_array()?.iter().map(Value::as_f64).collect() };
        if let Some(slot) = self.text_mut(field) {
            let Value::String(text) = value else {
                return Err(value);
            };
            *slot = Some(text);
            return Ok(());
        }
        match field.kind() {
            ModifierKind::Distances | ModifierKind::Azimuths => {
                let list = numbers(&value).ok_or_else(|| value.clone())?;
                if field == ModifierField::AM {
                    self.distances_m = list;
                } else {
                    self.azimuths_deg = list;
                }
            }
            ModifierKind::Direction => {
                self.direction_deg = Some(value.as_f64().ok_or_else(|| value.clone())?);
            }
            ModifierKind::Altitudes => {
                let list: Vec<Altitude> =
                    serde_json::from_value(value.clone()).map_err(|_| value)?;
                self.altitudes = list;
            }
            _ => return Err(value),
        }
        Ok(())
    }

    /// The stored value of `field`, if it is set; text is kept as entered,
    /// even when empty.
    fn stored(&self, field: ModifierField) -> Option<Value> {
        let list = |v: &[f64]| (!v.is_empty()).then(|| Value::from(v.to_vec()));
        match field {
            ModifierField::AM => list(&self.distances_m),
            ModifierField::AN => list(&self.azimuths_deg),
            ModifierField::Q => self.direction_deg.map(Value::from),
            ModifierField::X if self.altitudes.is_empty() => None,
            ModifierField::X => serde_json::to_value(&self.altitudes).ok(),
            _ => self.text(field)?.clone().map(Value::String),
        }
    }
}
