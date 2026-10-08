//! The declaration tables, one module per construction family, and the
//! constructors that keep each declaration to a line or two.

use super::{ModifierSpec, SymbolSpec};
use crate::family::Family;
use crate::modifier::ModifierField;
use crate::standard::StandardVersion;

mod first_milestone;

/// Every declaration table.
pub(super) static TABLES: &[&[SymbolSpec]] = &[first_milestone::SPECS];

/// Upper bound for symbols that take any number of points.
pub(super) const MANY: usize = 10_000;

/// An optional single-valued field.
pub(super) const fn opt(field: ModifierField) -> ModifierSpec {
    ModifierSpec::optional(field)
}

/// A required single-valued field.
pub(super) const fn req(field: ModifierField) -> ModifierSpec {
    ModifierSpec::required(field)
}

/// A field of `min..=max` values.
pub(super) const fn list(
    field: ModifierField,
    required: bool,
    min: usize,
    max: usize,
) -> ModifierSpec {
    ModifierSpec::counted(field, required, min, max)
}

/// Control measure (symbol set 25) `entity` of `standard`, built by
/// `family`; points and amplifiers are added with the methods below.
pub(super) const fn cm(standard: StandardVersion, entity: u32, family: Family) -> SymbolSpec {
    SymbolSpec {
        standard,
        symbol_set: 25,
        entity,
        min_points: 1,
        max_points: 1,
        modifiers: &[],
        family,
    }
}

impl SymbolSpec {
    /// The same declaration taking `min..=max` control points.
    pub(super) const fn points(self, min: usize, max: usize) -> Self {
        Self {
            min_points: min,
            max_points: max,
            ..self
        }
    }

    /// The same declaration drawing `modifiers`.
    pub(super) const fn amplifiers(self, modifiers: &'static [ModifierSpec]) -> Self {
        Self { modifiers, ..self }
    }
}
