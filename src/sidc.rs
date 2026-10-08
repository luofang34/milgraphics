//! Numeric symbol identification codes (MIL-STD-2525D/E, APP-6D/E).

use core::fmt;
use core::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::standard::StandardVersion;

#[cfg(test)]
mod tests;

/// A six-digit entity code (entity, entity type, entity subtype).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EntityCode(u32);

impl EntityCode {
    /// The entity code with these six digits, if it has at most six.
    pub const fn new(code: u32) -> Option<Self> {
        if code <= 999_999 {
            Some(Self(code))
        } else {
            None
        }
    }

    /// The code as a number, e.g. `140300`.
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl fmt::Display for EntityCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:06}", self.0)
    }
}

/// Why a symbol identification code was rejected.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum SidcError {
    /// The code is not 20 or 30 ASCII digits.
    #[error("symbol ID {code:?} must be 20 or 30 digits")]
    Format {
        /// The rejected input, truncated to 40 characters.
        code: String,
    },
}

/// A parsed numeric symbol identification code.
///
/// Only the first 20 digits carry meaning for tactical graphics; a 30-digit
/// code's trailing country and extension digits are preserved for output.
/// The version code is kept as written: whether the edition is supported is
/// decided later, so codes from unknown editions still round-trip.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SymbolId {
    digits: String,
}

impl SymbolId {
    /// Parses a 20- or 30-digit numeric code.
    pub fn parse(code: &str) -> Result<Self, SidcError> {
        let ascii_digits = code.bytes().all(|b| b.is_ascii_digit());
        if ascii_digits && (code.len() == 20 || code.len() == 30) {
            Ok(Self {
                digits: code.to_owned(),
            })
        } else {
            Err(SidcError::Format {
                code: code.chars().take(40).collect(),
            })
        }
    }

    /// Value of the digits at `range`, which `parse` guarantees exist.
    fn digits(&self, start: usize, len: usize) -> u32 {
        self.digits
            .bytes()
            .skip(start)
            .take(len)
            .fold(0, |acc, b| acc * 10 + u32::from(b - b'0'))
    }

    /// Digits 1–2: the standard version code.
    pub fn version_code(&self) -> u8 {
        self.digits(0, 2) as u8
    }

    /// The standard edition, if the version code is one the library knows.
    pub fn standard(&self) -> Option<StandardVersion> {
        StandardVersion::from_code(self.version_code())
    }

    /// Digit 3: context (reality, exercise, simulation).
    pub fn context(&self) -> u8 {
        self.digits(2, 1) as u8
    }

    /// Digit 4: standard identity (pending, unknown, friend, hostile, …).
    pub fn identity(&self) -> u8 {
        self.digits(3, 1) as u8
    }

    /// Digits 5–6: symbol set (25 = control measures).
    pub fn symbol_set(&self) -> u8 {
        self.digits(4, 2) as u8
    }

    /// Digit 7: status (0 present, 1 planned/anticipated/suspect).
    pub fn status(&self) -> u8 {
        self.digits(6, 1) as u8
    }

    /// Digit 8: headquarters / task force / dummy.
    pub fn hq_tf_dummy(&self) -> u8 {
        self.digits(7, 1) as u8
    }

    /// Digits 9–10: amplifier / descriptor.
    pub fn amplifier(&self) -> u8 {
        self.digits(8, 2) as u8
    }

    /// Digits 11–16: entity code.
    pub fn entity(&self) -> EntityCode {
        EntityCode(self.digits(10, 6))
    }

    /// Digits 17–18: sector 1 modifier.
    pub fn modifier1(&self) -> u8 {
        self.digits(16, 2) as u8
    }

    /// Digits 19–20: sector 2 modifier.
    pub fn modifier2(&self) -> u8 {
        self.digits(18, 2) as u8
    }

    /// The code as written.
    pub fn as_str(&self) -> &str {
        &self.digits
    }
}

impl fmt::Display for SymbolId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.digits)
    }
}

impl FromStr for SymbolId {
    type Err = SidcError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl Serialize for SymbolId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.digits)
    }
}

impl<'de> Deserialize<'de> for SymbolId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let code = String::deserialize(deserializer)?;
        Self::parse(&code).map_err(serde::de::Error::custom)
    }
}
