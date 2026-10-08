//! Typed text and numeric amplifiers of a graphic.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::geo::Altitude;

/// The amplifiers (modifier fields) of a graphic, named by their standard
/// field letters.
///
/// Text fields hold only what the operator entered: `T` is the bare
/// designation ("ALPHA", not "PL ALPHA"). Prefixes that the standard draws
/// around a field are construction output. Fields this version does not
/// model are kept in `unknown` and written back unchanged.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Modifiers {
    /// `T`: unique designation.
    #[serde(rename = "T", default, skip_serializing_if = "Option::is_none")]
    pub designation: Option<String>,
    /// `T1`: second unique designation.
    #[serde(rename = "T1", default, skip_serializing_if = "Option::is_none")]
    pub designation2: Option<String>,
    /// `H`: additional information.
    #[serde(rename = "H", default, skip_serializing_if = "Option::is_none")]
    pub additional_info: Option<String>,
    /// `W`: date-time group, start of validity.
    #[serde(rename = "W", default, skip_serializing_if = "Option::is_none")]
    pub dtg_start: Option<String>,
    /// `W1`: date-time group, end of validity.
    #[serde(rename = "W1", default, skip_serializing_if = "Option::is_none")]
    pub dtg_end: Option<String>,
    /// `AM`: distances in metres (widths, radii, ranges), in field order.
    #[serde(rename = "AM", default, skip_serializing_if = "Vec::is_empty")]
    pub distances_m: Vec<f64>,
    /// `AN`: azimuths in degrees clockwise from true north, in field order.
    #[serde(rename = "AN", default, skip_serializing_if = "Vec::is_empty")]
    pub azimuths_deg: Vec<f64>,
    /// `X`: altitudes or depths, in field order.
    #[serde(rename = "X", default, skip_serializing_if = "Vec::is_empty")]
    pub altitudes: Vec<Altitude>,
    /// Fields this version does not model, preserved as JSON content.
    #[serde(flatten)]
    pub unknown: BTreeMap<String, Value>,
}

impl Modifiers {
    /// True when no field is set.
    pub fn is_empty(&self) -> bool {
        self == &Self::default()
    }

    /// The designation, treating an empty string as absent.
    pub fn designation(&self) -> Option<&str> {
        self.designation.as_deref().filter(|t| !t.is_empty())
    }
}

/// An amplifier field that [`Modifiers`] models, named by its letter.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ModifierField {
    /// `T`
    T,
    /// `T1`
    T1,
    /// `H`
    H,
    /// `W`
    W,
    /// `W1`
    W1,
    /// `AM`
    AM,
    /// `AN`
    AN,
    /// `X`
    X,
}

impl ModifierField {
    /// Every modelled field.
    pub const ALL: [Self; 8] = [
        Self::T,
        Self::T1,
        Self::H,
        Self::W,
        Self::W1,
        Self::AM,
        Self::AN,
        Self::X,
    ];

    /// The field letter as the standard writes it.
    pub const fn name(self) -> &'static str {
        match self {
            Self::T => "T",
            Self::T1 => "T1",
            Self::H => "H",
            Self::W => "W",
            Self::W1 => "W1",
            Self::AM => "AM",
            Self::AN => "AN",
            Self::X => "X",
        }
    }

    /// Whether `modifiers` sets this field to something non-empty.
    pub fn is_set(self, modifiers: &Modifiers) -> bool {
        let text = |t: &Option<String>| t.as_deref().is_some_and(|t| !t.is_empty());
        match self {
            Self::T => text(&modifiers.designation),
            Self::T1 => text(&modifiers.designation2),
            Self::H => text(&modifiers.additional_info),
            Self::W => text(&modifiers.dtg_start),
            Self::W1 => text(&modifiers.dtg_end),
            Self::AM => !modifiers.distances_m.is_empty(),
            Self::AN => !modifiers.azimuths_deg.is_empty(),
            Self::X => !modifiers.altitudes.is_empty(),
        }
    }
}
