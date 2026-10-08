//! The declaration tables, one module per construction family, and the
//! constructors that keep each declaration to a line or two.

use super::{ModifierSpec, StandardRef, SymbolSpec};
use crate::family::Family;
use crate::modifier::ModifierField;
use crate::standard::StandardVersion;

mod first_milestone;

/// Every declaration table.
pub(super) static TABLES: &[&[SymbolSpec]] = &[first_milestone::SPECS];

/// Upper bound for symbols that take any number of points.
pub(super) const MANY: usize = 10_000;

pub(super) const D: &str = "mil-std-2525d-ch1";
pub(super) const E: &str = "mil-std-2525e-ch1";

/// A reference to `table` on PDF page `pdf_page` of `document`.
pub(super) const fn r(document: &'static str, table: &'static str, pdf_page: u16) -> StandardRef {
    StandardRef {
        document,
        table,
        pdf_page,
    }
}

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

/// A control measure (symbol set 25) defined by `reference` with draw rule
/// `draw_rule`; points and amplifiers are added with the methods below.
pub(super) const fn cm(
    standard: StandardVersion,
    entity: u32,
    name: &'static str,
    family: Family,
    draw_rule: &'static str,
    reference: StandardRef,
) -> SymbolSpec {
    SymbolSpec {
        standard,
        symbol_set: 25,
        entity,
        name,
        min_points: 1,
        max_points: 1,
        modifiers: &[],
        draw_rule,
        catalog_divergence: None,
        reference,
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

    /// The same declaration, noting why upstream's catalog draw rule differs.
    pub(super) const fn diverging(self, reason: &'static str) -> Self {
        Self {
            catalog_divergence: Some(reason),
            ..self
        }
    }
}
