//! Typed text and numeric amplifiers of a graphic.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::geo::Altitude;

mod field;
mod stored;

#[cfg(test)]
mod tests;

pub use field::{ModifierField, ModifierKind};

/// The amplifiers (modifier fields) of a graphic, stored under their
/// standard field letters.
///
/// Text fields hold only what the operator entered: `T` is the bare
/// designation ("ALPHA", not "PL ALPHA"). Prefixes that the standard draws
/// around a field are construction output. Fields this version does not
/// model are kept in `unknown` and written back unchanged.
///
/// Editors that work from a symbol's declaration rather than per-field code
/// use [`Modifiers::get`] and [`Modifiers::set`].
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(from = "Map<String, Value>", into = "Map<String, Value>")]
#[non_exhaustive]
pub struct Modifiers {
    /// `A`: code of a symbol drawn inside the graphic.
    pub symbol_icon: Option<String>,
    /// `AM`: distances in metres (widths, radii, ranges), in field order.
    pub distances_m: Vec<f64>,
    /// `AN`: azimuths in degrees clockwise from true north, in field order.
    pub azimuths_deg: Vec<f64>,
    /// `AP`: target number.
    pub target_number: Option<String>,
    /// `AP1`: target number extension.
    pub target_number_extension: Option<String>,
    /// `AS`: country.
    pub country: Option<String>,
    /// `B`: echelon.
    pub echelon: Option<String>,
    /// `C`: quantity.
    pub quantity: Option<String>,
    /// `H`: additional information.
    pub additional_info: Option<String>,
    /// `H1`: second additional information.
    pub additional_info2: Option<String>,
    /// `N`: hostile marking.
    pub hostile: Option<String>,
    /// `Q`: direction of movement in degrees clockwise from true north.
    pub direction_deg: Option<f64>,
    /// `T`: unique designation.
    pub designation: Option<String>,
    /// `T1`: second unique designation.
    pub designation2: Option<String>,
    /// `T2`: third unique designation.
    pub designation3: Option<String>,
    /// `V`: equipment type.
    pub equipment_type: Option<String>,
    /// `W`: date-time group, start of validity.
    pub dtg_start: Option<String>,
    /// `W1`: date-time group, end of validity.
    pub dtg_end: Option<String>,
    /// `X`: altitudes or depths, in field order.
    pub altitudes: Vec<Altitude>,
    /// `Y`: location.
    pub location: Option<String>,
    /// Fields this version does not model, preserved as JSON content.
    pub unknown: BTreeMap<String, Value>,
}

/// The value of one amplifier field, in the shape its [`ModifierKind`] gives.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum ModifierValue {
    /// Text, date-time and symbol-code fields.
    Text(String),
    /// Distances, azimuths and directions, in field order.
    Numbers(Vec<f64>),
    /// Altitudes or depths, in field order.
    Altitudes(Vec<Altitude>),
}

/// Why a value cannot be stored in a field.
#[derive(Clone, Debug, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum ModifierValueError {
    /// The value's shape does not fit the field's kind.
    #[error("amplifier {field} holds {kind:?} values")]
    Kind {
        /// The field.
        field: ModifierField,
        /// What the field holds.
        kind: ModifierKind,
    },
    /// A direction field holds exactly one number.
    #[error("amplifier {field} holds one value; {count} given")]
    Count {
        /// The field.
        field: ModifierField,
        /// Values given.
        count: usize,
    },
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

    /// The value of `field`, or `None` when it is unset or empty.
    pub fn get(&self, field: ModifierField) -> Option<ModifierValue> {
        if let Some(text) = self.text(field) {
            return text
                .as_deref()
                .filter(|t| !t.is_empty())
                .map(|t| ModifierValue::Text(t.to_owned()));
        }
        let numbers = match field {
            ModifierField::AM => self.distances_m.clone(),
            ModifierField::AN => self.azimuths_deg.clone(),
            ModifierField::Q => self.direction_deg.into_iter().collect(),
            ModifierField::X if !self.altitudes.is_empty() => {
                return Some(ModifierValue::Altitudes(self.altitudes.clone()));
            }
            _ => Vec::new(),
        };
        (!numbers.is_empty()).then_some(ModifierValue::Numbers(numbers))
    }

    /// Whether `field` is set to something non-empty.
    pub fn is_set(&self, field: ModifierField) -> bool {
        self.count(field) > 0
    }

    /// How many values `field` holds: 0 or 1 for single-valued fields.
    pub fn count(&self, field: ModifierField) -> usize {
        match self.get(field) {
            None => 0,
            Some(ModifierValue::Text(_)) => 1,
            Some(ModifierValue::Numbers(v)) => v.len(),
            Some(ModifierValue::Altitudes(v)) => v.len(),
        }
    }

    /// Stores `value` in `field`; an empty text or list clears it. The
    /// value's shape must fit the field's kind; contents are checked when
    /// the graphic is constructed.
    pub fn set(
        &mut self,
        field: ModifierField,
        value: ModifierValue,
    ) -> Result<(), ModifierValueError> {
        let kind = field.kind();
        let mismatch = ModifierValueError::Kind { field, kind };
        match value {
            ModifierValue::Text(text) => {
                let slot = self.text_mut(field).ok_or(mismatch)?;
                *slot = (!text.is_empty()).then_some(text);
            }
            ModifierValue::Numbers(values) => match field {
                ModifierField::AM => self.distances_m = values,
                ModifierField::AN => self.azimuths_deg = values,
                ModifierField::Q => match values[..] {
                    [] => self.direction_deg = None,
                    [one] => self.direction_deg = Some(one),
                    _ => {
                        let count = values.len();
                        return Err(ModifierValueError::Count { field, count });
                    }
                },
                _ => return Err(mismatch),
            },
            ModifierValue::Altitudes(values) if field == ModifierField::X => {
                self.altitudes = values;
            }
            ModifierValue::Altitudes(_) => return Err(mismatch),
        }
        Ok(())
    }

    /// Clears `field`.
    pub fn clear(&mut self, field: ModifierField) {
        if let Some(slot) = self.text_mut(field) {
            *slot = None;
            return;
        }
        match field {
            ModifierField::AM => self.distances_m.clear(),
            ModifierField::AN => self.azimuths_deg.clear(),
            ModifierField::Q => self.direction_deg = None,
            ModifierField::X => self.altitudes.clear(),
            _ => {}
        }
    }

    fn text(&self, field: ModifierField) -> Option<&Option<String>> {
        Some(match field {
            ModifierField::A => &self.symbol_icon,
            ModifierField::AP => &self.target_number,
            ModifierField::AP1 => &self.target_number_extension,
            ModifierField::AS => &self.country,
            ModifierField::B => &self.echelon,
            ModifierField::C => &self.quantity,
            ModifierField::H => &self.additional_info,
            ModifierField::H1 => &self.additional_info2,
            ModifierField::N => &self.hostile,
            ModifierField::T => &self.designation,
            ModifierField::T1 => &self.designation2,
            ModifierField::T2 => &self.designation3,
            ModifierField::V => &self.equipment_type,
            ModifierField::W => &self.dtg_start,
            ModifierField::W1 => &self.dtg_end,
            ModifierField::Y => &self.location,
            ModifierField::AM | ModifierField::AN | ModifierField::Q | ModifierField::X => {
                return None;
            }
        })
    }

    fn text_mut(&mut self, field: ModifierField) -> Option<&mut Option<String>> {
        Some(match field {
            ModifierField::A => &mut self.symbol_icon,
            ModifierField::AP => &mut self.target_number,
            ModifierField::AP1 => &mut self.target_number_extension,
            ModifierField::AS => &mut self.country,
            ModifierField::B => &mut self.echelon,
            ModifierField::C => &mut self.quantity,
            ModifierField::H => &mut self.additional_info,
            ModifierField::H1 => &mut self.additional_info2,
            ModifierField::N => &mut self.hostile,
            ModifierField::T => &mut self.designation,
            ModifierField::T1 => &mut self.designation2,
            ModifierField::T2 => &mut self.designation3,
            ModifierField::V => &mut self.equipment_type,
            ModifierField::W => &mut self.dtg_start,
            ModifierField::W1 => &mut self.dtg_end,
            ModifierField::Y => &mut self.location,
            ModifierField::AM | ModifierField::AN | ModifierField::Q | ModifierField::X => {
                return None;
            }
        })
    }
}
