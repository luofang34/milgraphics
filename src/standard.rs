//! Symbology standard editions, identified by the SIDC version code.

use core::fmt;

/// A symbology standard edition, as carried in the first two digits of a
/// numeric symbol identification code.
///
/// Only editions the library can reason about are listed. Persisted
/// definitions with any other version code are kept verbatim and reported as
/// unsupported rather than mapped to a nearby edition.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum StandardVersion {
    /// APP-6(D), version code 10. Upstream also files base MIL-STD-2525D here.
    App6D,
    /// MIL-STD-2525D change 1, version code 11.
    Mil2525Dch1,
    /// MIL-STD-2525E change 1, version code 15.
    Mil2525Ech1,
    /// APP-6(E) change 2, version code 16.
    App6Ech2,
}

impl StandardVersion {
    /// Every edition, in version-code order.
    pub const ALL: [Self; 4] = [
        Self::App6D,
        Self::Mil2525Dch1,
        Self::Mil2525Ech1,
        Self::App6Ech2,
    ];

    /// The edition with this SIDC version code, if it is one the library knows.
    pub const fn from_code(code: u8) -> Option<Self> {
        match code {
            10 => Some(Self::App6D),
            11 => Some(Self::Mil2525Dch1),
            15 => Some(Self::Mil2525Ech1),
            16 => Some(Self::App6Ech2),
            _ => None,
        }
    }

    /// The SIDC version code of this edition.
    pub const fn code(self) -> u8 {
        match self {
            Self::App6D => 10,
            Self::Mil2525Dch1 => 11,
            Self::Mil2525Ech1 => 15,
            Self::App6Ech2 => 16,
        }
    }
}

impl fmt::Display for StandardVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::App6D => "APP-6(D)",
            Self::Mil2525Dch1 => "MIL-STD-2525D change 1",
            Self::Mil2525Ech1 => "MIL-STD-2525E change 1",
            Self::App6Ech2 => "APP-6(E) change 2",
        })
    }
}
