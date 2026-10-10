//! The symbol catalog: which graphics exist, in which standard versions, with
//! which geometry, draw rule and modifiers. The rows are generated from
//! upstream's data tables (see `UPSTREAM.md`); this module holds their types
//! and lookup.

use crate::generated::catalog::ENTRIES;
pub use crate::generated::draw_rule::{DrawRule, MoDrawRule};
use crate::modifier::ModifierField;
use crate::sidc::EntityCode;
use crate::standard::StandardVersion;

#[cfg(test)]
mod tests;

/// The standard editions a catalog row applies to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VersionSet(u32);

impl VersionSet {
    /// Wraps a bit mask; used by the generated catalog.
    pub(crate) const fn from_bits(bits: u32) -> Self {
        Self(bits)
    }

    /// Whether `standard` is in the set.
    pub const fn contains(self, standard: StandardVersion) -> bool {
        self.contains_code(standard.code())
    }

    /// Whether version code `code` (bit `code` of the mask) is in the set.
    pub(crate) const fn contains_code(self, code: u8) -> bool {
        code < 32 && (self.0 >> code) & 1 == 1
    }

    /// The editions in the set, in version-code order.
    pub fn iter(self) -> impl Iterator<Item = StandardVersion> {
        StandardVersion::ALL
            .iter()
            .copied()
            .filter(move |s| self.contains(*s))
    }
}

/// Spatial form of a graphic.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum GeometryKind {
    /// Anchored at one or more discrete points.
    Point,
    /// An open polyline or curve.
    Line,
    /// A closed region.
    Area,
}

/// The draw rule of a catalog row, from the rule family of its symbol set.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CatalogDrawRule {
    /// Control measures (symbol set 25).
    Standard(DrawRule),
    /// Meteorological and oceanographic symbols (sets 45 and 46).
    Metoc(MoDrawRule),
}

/// One graphic as listed in upstream's data tables.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct CatalogEntry {
    /// Two-digit symbol set.
    pub symbol_set: u8,
    /// Six-digit entity code.
    pub entity: EntityCode,
    /// Display name of the entity.
    pub name: &'static str,
    /// Names of the enclosing hierarchy levels, outermost first.
    pub path: &'static [&'static str],
    /// Version codes this row applies to.
    pub versions: VersionSet,
    /// Spatial form.
    pub geometry: GeometryKind,
    /// How upstream draws it.
    pub draw_rule: CatalogDrawRule,
    /// Modifiers the standard allows. Empty for METOC rows, whose modifier
    /// column upstream uses for colour instead.
    pub modifiers: &'static [ModifierField],
}

/// Every catalog row, ordered by `(symbol_set, entity, first version)`.
pub fn entries() -> &'static [CatalogEntry] {
    ENTRIES
}

/// The row for `entity` in `symbol_set` under `standard`.
pub fn lookup(
    standard: StandardVersion,
    symbol_set: u8,
    entity: EntityCode,
) -> Option<&'static CatalogEntry> {
    let all = entries();
    let start = all.partition_point(|e| (e.symbol_set, e.entity) < (symbol_set, entity));
    all.iter()
        .skip(start)
        .take_while(|e| e.symbol_set == symbol_set && e.entity == entity)
        .find(|e| e.versions.contains(standard))
}
