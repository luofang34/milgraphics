//! The declared support table: one entry per (standard, symbol).
//!
//! The generated catalog lists what upstream knows; only symbols listed here,
//! each with its acceptance tests, are drawn.

use crate::catalog::CatalogEntry;
use crate::family::Family;
use crate::generated::references::REFERENCES;
use crate::modifier::ModifierField;
use crate::sidc::SymbolId;
use crate::standard::StandardVersion;

mod table;

#[cfg(test)]
mod tests;

/// Where the standard defines a symbol.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct StandardRef {
    /// Document identifier, as in `tools/oracle/pin.json`.
    pub document: &'static str,
    /// Table holding the symbol's template.
    pub table: &'static str,
    /// 1-based page of the PDF file.
    pub pdf_page: u16,
    /// Draw rule printed in the symbol's row, when the row prints one.
    pub draw_rule: Option<&'static str>,
}

/// How a symbol uses one amplifier field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ModifierSpec {
    /// The field.
    pub field: ModifierField,
    /// Whether the symbol cannot be drawn without it.
    pub required: bool,
    /// Fewest values when set; 1 for single-valued fields.
    pub min_count: usize,
    /// Most values; 1 for single-valued fields.
    pub max_count: usize,
}

impl ModifierSpec {
    /// An optional single-valued field.
    pub(crate) const fn optional(field: ModifierField) -> Self {
        Self::counted(field, false, 1, 1)
    }

    /// A required single-valued field.
    pub(crate) const fn required(field: ModifierField) -> Self {
        Self::counted(field, true, 1, 1)
    }

    /// A field holding `min_count..=max_count` values.
    pub(crate) const fn counted(
        field: ModifierField,
        required: bool,
        min_count: usize,
        max_count: usize,
    ) -> Self {
        Self {
            field,
            required,
            min_count,
            max_count,
        }
    }
}

/// Everything the library declares about one supported symbol: enough for an
/// editor to offer it in a palette and build its amplifier form.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct SymbolSpec {
    /// Standard edition.
    pub standard: StandardVersion,
    /// Two-digit symbol set: 25 for control measures, 45 and 46 for METOC.
    pub symbol_set: u8,
    /// Six-digit entity code.
    pub entity: u32,
    /// Fewest control points.
    pub min_points: usize,
    /// Most control points.
    pub max_points: usize,
    /// Amplifiers that are drawn, with their constraints. A field not listed
    /// here is refused.
    pub modifiers: &'static [ModifierSpec],
    /// Construction family.
    pub(crate) family: Family,
}

impl SymbolSpec {
    /// The symbol's name, as upstream's catalog gives it.
    pub fn name(&self) -> &'static str {
        self.catalog_entry().map_or("unnamed symbol", |e| e.name)
    }

    /// Where the standard defines the symbol, including the draw rule it
    /// prints. Every declared symbol has one.
    pub fn reference(&self) -> Option<&'static StandardRef> {
        REFERENCES
            .iter()
            .find(|(standard, set, entity, _)| {
                (*standard, *set, *entity) == (self.standard, self.symbol_set, self.entity)
            })
            .map(|(_, _, _, r)| r)
    }

    /// How the symbol uses `field`, or `None` when it does not draw it.
    pub fn modifier(&self, field: ModifierField) -> Option<&'static ModifierSpec> {
        self.modifiers.iter().find(|m| m.field == field)
    }

    /// The fields without which the symbol cannot be drawn.
    pub fn required(&self) -> impl Iterator<Item = ModifierField> + 'static {
        self.modifiers
            .iter()
            .filter(|m| m.required)
            .map(|m| m.field)
    }

    /// Whether control points may be inserted and deleted.
    pub fn allows_vertex_edits(&self) -> bool {
        self.family.allows_vertex_edits()
    }

    /// The upstream catalog row for this symbol: its hierarchy path and
    /// geometry, for grouping a palette.
    pub fn catalog_entry(&self) -> Option<&'static CatalogEntry> {
        crate::catalog::lookup(self.standard.code(), self.symbol_set, self.entity)
    }
}

/// Why a symbol cannot be drawn.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum Unsupported {
    /// The version code is not a known edition.
    #[error("standard version code {code} is not supported")]
    Standard {
        /// The version code.
        code: u8,
    },
    /// The symbol is not declared for this edition.
    #[error("{standard} symbol set {symbol_set} entity {entity:06} is not supported")]
    Symbol {
        /// The edition.
        standard: StandardVersion,
        /// Symbol set.
        symbol_set: u8,
        /// Entity code.
        entity: u32,
    },
}

/// The declaration for `symbol`, or why it is not supported.
pub fn spec(symbol: &SymbolId) -> Result<&'static SymbolSpec, Unsupported> {
    let code = symbol.version_code();
    let standard = symbol.standard().ok_or(Unsupported::Standard { code })?;
    let (symbol_set, entity) = (symbol.symbol_set(), symbol.entity().get());
    all()
        .find(|s| s.symbol_set == symbol_set && s.standard == standard && s.entity == entity)
        .ok_or(Unsupported::Symbol {
            standard,
            symbol_set,
            entity,
        })
}

/// Every declared symbol.
pub fn all() -> impl Iterator<Item = &'static SymbolSpec> + Clone {
    table::TABLES.iter().flat_map(|t| t.iter())
}
