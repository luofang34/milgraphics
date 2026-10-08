//! The persisted, editable description of one tactical graphic.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::geo::{Altitude, GeoPoint};
use crate::modifier::Modifiers;
use crate::sidc::SymbolId;

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
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ControlPoint {
    /// Horizontal position.
    #[serde(flatten)]
    pub position: GeoPoint,
    /// Altitude, when the graphic is not clamped to the ground.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub altitude: Option<Altitude>,
}

impl ControlPoint {
    /// A ground-clamped control point.
    pub fn ground(position: GeoPoint) -> Self {
        Self {
            position,
            altitude: None,
        }
    }
}

/// Display overrides chosen by the operator; absent fields follow the standard.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct StyleOverrides {
    /// Line colour as `#rrggbb` or `#rrggbbaa`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_color: Option<String>,
    /// Fill colour as `#rrggbb` or `#rrggbbaa`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_color: Option<String>,
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
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Validity {
    /// Start of the span, if bounded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,
    /// End of the span, if bounded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,
}

/// A tactical graphic as persisted and edited: the only authority from which
/// constructions and render plans are derived.
///
/// Fields this version does not understand are carried in `unknown`, so an
/// edit made by an older version does not drop data written by a newer one.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GraphicDefinition {
    /// Stable identifier.
    pub id: GraphicId,
    /// Symbol identification code, including the standard version.
    pub symbol: SymbolId,
    /// Ordered control points.
    pub points: Vec<ControlPoint>,
    /// Amplifiers.
    #[serde(default, skip_serializing_if = "Modifiers::is_empty")]
    pub modifiers: Modifiers,
    /// Display overrides.
    #[serde(default, skip_serializing_if = "StyleOverrides::is_empty")]
    pub style: StyleOverrides,
    /// When the graphic applies.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validity: Option<Validity>,
    /// Change counter, advanced by every edit with `wrapping_add(1)`. It does
    /// not identify commands; hosts use their own event IDs for that.
    #[serde(default)]
    pub revision: u64,
    /// Fields this version does not model, preserved as JSON content.
    #[serde(flatten)]
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
