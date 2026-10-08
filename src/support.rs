//! The declared support table: one entry per (standard, symbol).
//!
//! The generated catalog lists what upstream knows; only symbols listed here,
//! each with its acceptance tests, are drawn.

use crate::family::Family;
use crate::modifier::ModifierField;
use crate::sidc::SymbolId;
use crate::standard::StandardVersion;

#[cfg(test)]
mod tests;

/// Where the standard defines a symbol.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StandardRef {
    /// Document identifier, as in `tools/oracle/pin.json`.
    pub document: &'static str,
    /// Table holding the symbol's template.
    pub table: &'static str,
    /// 1-based page of the PDF file.
    pub pdf_page: u16,
}

/// Everything the library declares about one supported symbol.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SymbolSpec {
    /// Standard edition.
    pub standard: StandardVersion,
    /// Six-digit entity code in symbol set 25.
    pub entity: u32,
    /// Name as the standard gives it.
    pub name: &'static str,
    /// Construction family.
    pub family: Family,
    /// Fewest control points.
    pub min_points: usize,
    /// Most control points.
    pub max_points: usize,
    /// Amplifiers that are drawn.
    pub modifiers: &'static [ModifierField],
    /// Amplifiers without which the symbol cannot be drawn.
    pub required: &'static [ModifierField],
    /// Draw rule printed in the standard.
    pub draw_rule: &'static str,
    /// Why the upstream catalog's draw rule differs, if it does.
    pub catalog_divergence: Option<&'static str>,
    /// Where the standard defines it.
    pub reference: StandardRef,
}

/// Why a symbol cannot be drawn.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
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
    SPECS
        .iter()
        .find(|s| symbol_set == 25 && s.standard == standard && s.entity == entity)
        .ok_or(Unsupported::Symbol {
            standard,
            symbol_set,
            entity,
        })
}

/// Every declared symbol.
pub fn all() -> &'static [SymbolSpec] {
    SPECS
}

const D: &str = "mil-std-2525d-ch1";
const E: &str = "mil-std-2525e-ch1";
const MAX: usize = 10_000;

const fn r(document: &'static str, table: &'static str, pdf_page: u16) -> StandardRef {
    StandardRef {
        document,
        table,
        pdf_page,
    }
}

use ModifierField as M;
use StandardVersion::{Mil2525Dch1, Mil2525Ech1};

static SPECS: &[SymbolSpec] = &[
    SymbolSpec {
        standard: Mil2525Dch1,
        entity: 140_300,
        name: "Phase Line",
        family: Family::PhaseLine,
        min_points: 2,
        max_points: MAX,
        modifiers: &[M::T],
        required: &[],
        draw_rule: "Line2",
        catalog_divergence: None,
        reference: r(D, "TABLE H-VII", 446),
    },
    SymbolSpec {
        standard: Mil2525Ech1,
        entity: 140_300,
        name: "Phase Line",
        family: Family::PhaseLine,
        min_points: 2,
        max_points: MAX,
        modifiers: &[M::T],
        required: &[],
        draw_rule: "Line1",
        catalog_divergence: None,
        reference: r(E, "TABLE L-VII", 478),
    },
    SymbolSpec {
        standard: Mil2525Dch1,
        entity: 120_200,
        name: "Named Area of Interest",
        family: Family::LabelledArea { prefix: "NAI" },
        min_points: 3,
        max_points: MAX,
        modifiers: &[M::T],
        required: &[],
        draw_rule: "Area1",
        catalog_divergence: None,
        reference: r(D, "TABLE H-V", 436),
    },
    SymbolSpec {
        standard: Mil2525Ech1,
        entity: 120_200,
        name: "Named Area of Interest",
        family: Family::LabelledArea { prefix: "NAI" },
        min_points: 3,
        max_points: MAX,
        modifiers: &[M::T],
        required: &[],
        draw_rule: "Area1",
        catalog_divergence: None,
        reference: r(E, "TABLE L-V", 468),
    },
];
