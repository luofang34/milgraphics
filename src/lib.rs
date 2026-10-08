//! Multi-point military tactical graphics (MIL-STD-2525 / APP-6).
//!
//! `milgraphics` turns a persisted graphic definition — symbol identity,
//! geographic control points and typed modifiers — into map-engine-neutral
//! output: geometry, labels, symbol placements, edit handles and pick
//! references. Single-point symbols are rendered by the sibling `milsymbol`
//! crate; the two compose through data and neither depends on the other.

pub mod antimeridian;
pub mod budget;
pub mod catalog;
pub mod construction;
pub mod definition;
pub mod edit;
pub mod family;
mod generated;
pub mod geo;
mod geodesy;
pub mod geojson;
pub mod modifier;
pub mod persist;
pub mod pick;
pub mod render;
pub mod sidc;
pub mod standard;
pub mod style;
pub mod support;
pub mod svg;
mod version;

pub use budget::{Budget, BudgetError};
pub use catalog::{CatalogDrawRule, CatalogEntry, GeometryKind, VersionSet};
pub use construction::Construction;
pub use definition::{ControlPoint, GraphicDefinition, GraphicId, StyleOverrides, Validity};
pub use edit::{Edit, EditError, HandleId, apply_edit};
pub use family::{Config, ConstructError, construct};
pub use generated::catalog::ModifierKey;
pub use generated::draw_rule::{DrawRule, MoDrawRule};
pub use geo::{Altitude, GeoPoint, VerticalDatum};
pub use modifier::{ModifierField, Modifiers};
pub use persist::{PersistError, PersistedGraphic};
pub use pick::PickRef;
pub use render::{RenderError, RenderPlan, View, render};
pub use sidc::{EntityCode, SymbolId};
pub use standard::StandardVersion;
pub use support::{SymbolSpec, Unsupported};
pub use version::RENDERER_VERSION;
