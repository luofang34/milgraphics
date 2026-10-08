//! Multi-point military tactical graphics (MIL-STD-2525 / APP-6).
//!
//! `milgraphics` turns a persisted graphic definition — symbol identity,
//! geographic control points and typed modifiers — into map-engine-neutral
//! output: geometry, labels, symbol placements, edit handles and pick
//! references. Single-point symbols are rendered by the sibling `milsymbol`
//! crate; the two compose through data and neither depends on the other.

mod catalog;
mod generated;
mod version;

pub use catalog::{CatalogDrawRule, CatalogEntry, GeometryKind, VersionSet, entries, lookup};
pub use generated::catalog::ModifierKey;
pub use generated::draw_rule::{DrawRule, MoDrawRule};
pub use version::RENDERER_VERSION;
