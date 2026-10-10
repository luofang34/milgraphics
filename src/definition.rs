//! The persisted, editable description of one tactical graphic.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::geo::{Altitude, GeoPoint};
use crate::modifier::Modifiers;
use crate::sidc::SymbolId;
use crate::style::Rgba;

#[cfg(test)]
mod tests;

/// Longest accepted graphic ID, in bytes.
pub const MAX_ID_BYTES: usize = 256;

/// A stable, host-assigned identifier of a graphic.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct GraphicId(String);

/// Why a graphic ID was rejected.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum GraphicIdError {
    /// The ID is empty.
    #[error("graphic ID is empty")]
    Empty,
    /// The ID is longer than [`MAX_ID_BYTES`].
    #[error("graphic ID is {len} bytes; the limit is {MAX_ID_BYTES}")]
    TooLong {
        /// Length of the rejected ID in bytes.
        len: usize,
    },
}

impl GraphicId {
    /// A validated ID.
    pub fn new(id: impl Into<String>) -> Result<Self, GraphicIdError> {
        let id = id.into();
        if id.is_empty() {
            Err(GraphicIdError::Empty)
        } else if id.len() > MAX_ID_BYTES {
            Err(GraphicIdError::TooLong { len: id.len() })
        } else {
            Ok(Self(id))
        }
    }

    /// The ID as a string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for GraphicId {
    type Error = GraphicIdError;

    fn try_from(id: String) -> Result<Self, GraphicIdError> {
        Self::new(id)
    }
}

impl From<GraphicId> for String {
    fn from(id: GraphicId) -> Self {
        id.0
    }
}

impl fmt::Display for GraphicId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// One control (anchor) point of a graphic.
///
/// Stored as a JSON object with `lon`, `lat`, an optional `altitude`, and
/// any fields a newer version added, which are kept as JSON content.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "StoredPoint", into = "StoredPoint")]
#[non_exhaustive]
pub struct ControlPoint {
    /// Horizontal position.
    pub position: GeoPoint,
    /// Altitude, when the graphic is not clamped to the ground.
    pub altitude: Option<Altitude>,
    /// Fields this version does not model, preserved as JSON content.
    pub unknown: BTreeMap<String, Value>,
}

impl ControlPoint {
    /// A ground-clamped control point.
    pub fn ground(position: GeoPoint) -> Self {
        Self {
            position,
            altitude: None,
            unknown: BTreeMap::new(),
        }
    }
}

/// The stored JSON object of a control point, kept out of the public API so
/// the conversion is an implementation detail of serialization.
#[derive(Serialize, Deserialize)]
#[serde(transparent)]
struct StoredPoint(Map<String, Value>);

/// Why a stored control point was rejected.
#[derive(Debug, thiserror::Error)]
pub(crate) enum ControlPointError {
    /// `lon` or `lat` is missing or not a number.
    #[error("control point needs numeric lon and lat")]
    Coordinates,
    /// The coordinates are out of range.
    #[error(transparent)]
    Geo(#[from] crate::geo::GeoError),
    /// `altitude` is malformed.
    #[error("control point altitude is malformed: {0}")]
    Altitude(#[source] serde_json::Error),
}

impl TryFrom<StoredPoint> for ControlPoint {
    type Error = ControlPointError;

    fn try_from(StoredPoint(mut map): StoredPoint) -> Result<Self, ControlPointError> {
        let mut number = |key: &str| {
            map.remove(key)
                .and_then(|v| v.as_f64())
                .ok_or(ControlPointError::Coordinates)
        };
        let (lon, lat) = (number("lon")?, number("lat")?);
        let altitude = match map.remove("altitude") {
            Some(v) => Some(serde_json::from_value(v).map_err(ControlPointError::Altitude)?),
            None => None,
        };
        Ok(Self {
            position: GeoPoint::new(lon, lat)?,
            altitude,
            unknown: map.into_iter().collect(),
        })
    }
}

impl From<ControlPoint> for StoredPoint {
    fn from(p: ControlPoint) -> Self {
        let mut map = Map::new();
        map.insert("lon".to_owned(), Value::from(p.position.lon()));
        map.insert("lat".to_owned(), Value::from(p.position.lat()));
        if let Some(a) = p.altitude {
            // Serializing these plain fields cannot fail; a failure would
            // drop only the altitude, never the point.
            if let Ok(altitude) = serde_json::to_value(a) {
                map.insert("altitude".to_owned(), altitude);
            }
        }
        map.extend(p.unknown);
        Self(map)
    }
}

/// Display overrides chosen by the operator; absent fields follow the standard.
///
/// Colours are stored as lower-case `#rrggbb` when opaque and `#rrggbbaa`
/// otherwise. A stored colour that is not `#rrggbb` or `#rrggbbaa` makes the
/// whole definition undecodable, so the host keeps it as stored rather than
/// drawing it in a colour the operator did not choose.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StyleOverrides {
    /// Line colour.
    #[serde(default, skip_serializing_if = "Option::is_none", with = "hex_color")]
    pub line_color: Option<Rgba>,
    /// Fill colour.
    #[serde(default, skip_serializing_if = "Option::is_none", with = "hex_color")]
    pub fill_color: Option<Rgba>,
    /// Fields this version does not model, preserved as JSON content.
    #[serde(flatten)]
    pub unknown: BTreeMap<String, Value>,
}

impl StyleOverrides {
    /// True when nothing is overridden.
    pub fn is_empty(&self) -> bool {
        self == &Self::default()
    }
}

/// The time span in which a graphic applies, as ISO 8601 strings.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Validity {
    /// Start of the span, if bounded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,
    /// End of the span, if bounded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,
    /// Fields this version does not model, preserved as JSON content.
    #[serde(flatten)]
    pub unknown: BTreeMap<String, Value>,
}

/// A tactical graphic as persisted and edited: the only authority from which
/// constructions and render plans are derived.
///
/// It is stored through [`crate::PersistedGraphic`] only, which adds the
/// schema version. Fields this version does not understand are carried in
/// `unknown`, so an edit made by an older version does not drop data written
/// by a newer one.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct GraphicDefinition {
    /// Stable identifier.
    pub id: GraphicId,
    /// Symbol identification code, including the standard version.
    pub symbol: SymbolId,
    /// Ordered control points.
    pub points: Vec<ControlPoint>,
    /// Amplifiers.
    pub modifiers: Modifiers,
    /// Display overrides.
    pub style: StyleOverrides,
    /// When the graphic applies.
    pub validity: Option<Validity>,
    /// Change counter, advanced by every edit with `wrapping_add(1)`. It does
    /// not identify commands; hosts use their own event IDs for that.
    pub revision: u64,
    /// Fields this version does not model, preserved as JSON content.
    pub unknown: BTreeMap<String, Value>,
}

impl GraphicDefinition {
    /// A definition at revision 0 with no amplifiers or overrides.
    pub fn new(id: GraphicId, symbol: SymbolId, points: Vec<ControlPoint>) -> Self {
        Self {
            id,
            symbol,
            points,
            modifiers: Modifiers::default(),
            style: StyleOverrides::default(),
            validity: None,
            revision: 0,
            unknown: BTreeMap::new(),
        }
    }

    /// Control-point positions, in order.
    pub fn positions(&self) -> impl ExactSizeIterator<Item = GeoPoint> + '_ {
        self.points.iter().map(|p| p.position)
    }
}

/// Serde form of an optional colour: a hex string.
mod hex_color {
    use serde::{Deserialize, Deserializer, Serializer};

    use crate::style::Rgba;

    pub(super) fn serialize<S: Serializer>(
        color: &Option<Rgba>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match color {
            Some(c) => serializer.serialize_str(&c.to_stored_hex()),
            None => serializer.serialize_none(),
        }
    }

    pub(super) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<Rgba>, D::Error> {
        let Some(text) = Option::<String>::deserialize(deserializer)? else {
            return Ok(None);
        };
        Rgba::parse_hex(&text).map(Some).ok_or_else(|| {
            serde::de::Error::custom(format!("colour {text:?} is not #rrggbb or #rrggbbaa"))
        })
    }
}
