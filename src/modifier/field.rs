//! Amplifier fields: their letters and what kind of value each holds.

/// An amplifier (modifier) field, named by the letters the standard gives it.
///
/// These are the fields upstream's symbol tables list for multipoint
/// graphics. Which of them a symbol draws is declared per symbol in
/// [`crate::SymbolSpec::modifiers`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum ModifierField {
    /// `A`: symbol icon, the code of a symbol drawn inside the graphic.
    A,
    /// `AM`: distances (widths, radii, ranges) in metres.
    AM,
    /// `AN`: azimuths in degrees clockwise from true north.
    AN,
    /// `AP`: target number.
    AP,
    /// `AP1`: target number extension.
    AP1,
    /// `AS`: country.
    AS,
    /// `B`: echelon.
    B,
    /// `C`: quantity.
    C,
    /// `H`: additional information.
    H,
    /// `H1`: second additional information.
    H1,
    /// `N`: hostile marking ("ENY").
    N,
    /// `Q`: direction of movement in degrees clockwise from true north.
    Q,
    /// `T`: unique designation.
    T,
    /// `T1`: second unique designation.
    T1,
    /// `T2`: third unique designation.
    T2,
    /// `V`: equipment type.
    V,
    /// `W`: date-time group, start of validity.
    W,
    /// `W1`: date-time group, end of validity.
    W1,
    /// `X`: altitudes or depths.
    X,
    /// `Y`: location.
    Y,
}

/// What kind of value a field holds, for editors that build a form from a
/// symbol's declaration.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ModifierKind {
    /// Free text.
    Text,
    /// A date-time group, as entered.
    DateTime,
    /// A symbol identification code.
    SymbolCode,
    /// Distances in metres, finite and non-negative.
    Distances,
    /// Azimuths in degrees, finite.
    Azimuths,
    /// One direction in degrees, finite.
    Direction,
    /// Altitudes or depths, each with its vertical datum.
    Altitudes,
}

impl ModifierField {
    /// Every field.
    pub const ALL: &'static [Self] = &[
        Self::A,
        Self::AM,
        Self::AN,
        Self::AP,
        Self::AP1,
        Self::AS,
        Self::B,
        Self::C,
        Self::H,
        Self::H1,
        Self::N,
        Self::Q,
        Self::T,
        Self::T1,
        Self::T2,
        Self::V,
        Self::W,
        Self::W1,
        Self::X,
        Self::Y,
    ];

    /// The field letters as the standard writes them; also the key under
    /// which the field is stored.
    pub const fn name(self) -> &'static str {
        match self {
            Self::A => "A",
            Self::AM => "AM",
            Self::AN => "AN",
            Self::AP => "AP",
            Self::AP1 => "AP1",
            Self::AS => "AS",
            Self::B => "B",
            Self::C => "C",
            Self::H => "H",
            Self::H1 => "H1",
            Self::N => "N",
            Self::Q => "Q",
            Self::T => "T",
            Self::T1 => "T1",
            Self::T2 => "T2",
            Self::V => "V",
            Self::W => "W",
            Self::W1 => "W1",
            Self::X => "X",
            Self::Y => "Y",
        }
    }

    /// A short English label for the field, for editor forms. Labels may be
    /// reworded; [`ModifierField::name`] is the stable key.
    pub const fn label(self) -> &'static str {
        match self {
            Self::A => "Symbol icon",
            Self::AM => "Distance",
            Self::AN => "Azimuth",
            Self::AP => "Target number",
            Self::AP1 => "Target number extension",
            Self::AS => "Country",
            Self::B => "Echelon",
            Self::C => "Quantity",
            Self::H => "Additional information",
            Self::H1 => "Additional information 1",
            Self::N => "Hostile",
            Self::Q => "Direction of movement",
            Self::T => "Unique designation",
            Self::T1 => "Unique designation 1",
            Self::T2 => "Unique designation 2",
            Self::V => "Equipment type",
            Self::W => "Date-time group",
            Self::W1 => "Date-time group 1",
            Self::X => "Altitude/depth",
            Self::Y => "Location",
        }
    }

    /// The field whose letters are `name`, if any.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|f| f.name() == name)
    }

    /// The kind of value the field holds.
    pub const fn kind(self) -> ModifierKind {
        match self {
            Self::A => ModifierKind::SymbolCode,
            Self::AM => ModifierKind::Distances,
            Self::AN => ModifierKind::Azimuths,
            Self::Q => ModifierKind::Direction,
            Self::W | Self::W1 => ModifierKind::DateTime,
            Self::X => ModifierKind::Altitudes,
            Self::AP
            | Self::AP1
            | Self::AS
            | Self::B
            | Self::C
            | Self::H
            | Self::H1
            | Self::N
            | Self::T
            | Self::T1
            | Self::T2
            | Self::V
            | Self::Y => ModifierKind::Text,
        }
    }
}

impl core::fmt::Display for ModifierField {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.name())
    }
}
