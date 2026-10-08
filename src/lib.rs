//! Multi-point military tactical graphics (MIL-STD-2525 / APP-6).
//!
//! `milgraphics` turns a persisted graphic definition — symbol identity,
//! geographic control points and typed modifiers — into map-engine-neutral
//! output: geometry, labels, symbol placements, edit handles and pick
//! references. Single-point symbols are rendered by the sibling `milsymbol`
//! crate; the two compose through data and neither depends on the other.
//!
//! ```
//! use milgraphics::render::{FixedAdvanceMetrics, Font, LocalEquirectangular};
//! use milgraphics::{Budget, Config, ControlPoint, GeoPoint, GraphicDefinition, GraphicId};
//! use milgraphics::{PersistedGraphic, SymbolId, View, construct, render};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // A 2525D change 1 phase line "PL ALPHA".
//! let points = [(20.0, 50.0), (20.1, 50.02)]
//!     .into_iter()
//!     .map(|(lon, lat)| GeoPoint::new(lon, lat).map(ControlPoint::ground))
//!     .collect::<Result<Vec<_>, _>>()?;
//! let symbol = SymbolId::parse("11032500001403000000")?;
//! let mut def = GraphicDefinition::new(GraphicId::new("pl-1")?, symbol, points);
//! def.modifiers.designation = Some("ALPHA".into());
//!
//! // Geographic construction: cache it until the definition changes.
//! let construction = construct(&def, &Config::default())?;
//! // Resolve it for a view; hosts pass their own projection and font metrics.
//! let frame = LocalEquirectangular::new(19.95, 50.07, 50_000.0, 96.0);
//! let view = View { view_revision: 1, surface_revision: 0, label_font: Font::default() };
//! let plan = render(&construction, &view, &frame, &FixedAdvanceMetrics::default(), &Budget::default())?;
//! assert_eq!(plan.labels.len(), 2);
//!
//! // Store it; unknown fields from newer versions survive later edits.
//! let stored = PersistedGraphic::from_definition(&def)?;
//! assert_eq!(stored.decode()?, def);
//! # Ok(())
//! # }
//! ```

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
mod plane;
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
pub use pick::{PickRef, PickTarget};
pub use render::{RenderError, RenderPlan, View, render};
pub use sidc::{EntityCode, SymbolId};
pub use standard::StandardVersion;
pub use support::{SymbolSpec, Unsupported};
pub use version::RENDERER_VERSION;

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;
