//! The stored form of a [`GraphicDefinition`], private so that definitions
//! are persisted only through [`super::PersistedGraphic`].

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::PersistError;
use crate::definition::{ControlPoint, GraphicDefinition, GraphicId, StyleOverrides, Validity};
use crate::geo::Altitude;
use crate::modifier::{ModifierField, ModifierValue, Modifiers};
use crate::sidc::SymbolId;

/// The JSON object of a definition, without the schema version.
#[derive(Serialize, Deserialize)]
pub(super) struct StoredDefinition {
    id: GraphicId,
    symbol: SymbolId,
    points: Vec<ControlPoint>,
    #[serde(default, skip_serializing_if = "Modifiers::is_empty")]
    modifiers: Modifiers,
    #[serde(default, skip_serializing_if = "StyleOverrides::is_empty")]
    style: StyleOverrides,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    validity: Option<Validity>,
    #[serde(default)]
    revision: u64,
    #[serde(flatten)]
    unknown: BTreeMap<String, Value>,
}

impl From<GraphicDefinition> for StoredDefinition {
    fn from(d: GraphicDefinition) -> Self {
        Self {
            id: d.id,
            symbol: d.symbol,
            points: d.points,
            modifiers: d.modifiers,
            style: d.style,
            validity: d.validity,
            revision: d.revision,
            unknown: d.unknown,
        }
    }
}

impl From<StoredDefinition> for GraphicDefinition {
    fn from(s: StoredDefinition) -> Self {
        let mut d = GraphicDefinition::new(s.id, s.symbol, s.points);
        d.modifiers = s.modifiers;
        d.style = s.style;
        d.validity = s.validity;
        d.revision = s.revision;
        d.unknown = s.unknown;
        d
    }
}

/// Refuses NaN and infinities, which serde_json would write as `null`.
/// Unknown fields hold JSON values already, which cannot be non-finite.
pub(super) fn check_finite(d: &GraphicDefinition) -> Result<(), PersistError> {
    let non_finite = |field: String| PersistError::NonFinite {
        id: d.id.clone(),
        field,
    };
    let altitude_ok = |a: &Altitude| a.metres.is_finite();
    for (i, p) in d.points.iter().enumerate() {
        if p.altitude.as_ref().is_some_and(|a| !altitude_ok(a)) {
            return Err(non_finite(format!("points[{i}].altitude")));
        }
    }
    for &field in ModifierField::ALL {
        let finite = match d.modifiers.get(field) {
            Some(ModifierValue::Numbers(v)) => v.iter().all(|n| n.is_finite()),
            Some(ModifierValue::Altitudes(v)) => v.iter().all(altitude_ok),
            _ => true,
        };
        if !finite {
            return Err(non_finite(field.name().to_owned()));
        }
    }
    Ok(())
}
