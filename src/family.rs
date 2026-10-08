//! Symbol families: shared construction and edit rules, and the entry point
//! that validates a definition and builds its construction.

use crate::budget::{Budget, BudgetError};
use crate::construction::{Construction, HandleKind, HandleSpec};
use crate::definition::GraphicDefinition;
use crate::edit::{self, EditError, HandleId};
use crate::geo::GeoPoint;
use crate::geodesy::Earth;
use crate::modifier::ModifierField;
use crate::style::{self, Palette};
use crate::support::{self, SymbolSpec, Unsupported};
use crate::version::RENDERER_VERSION;

mod area;
mod axis;
mod bypass;
mod context;
mod corridor;
mod phase_line;
mod ported;
mod range_fan;
mod validate;

#[cfg(test)]
mod tests;

pub(crate) use context::Ctx;
pub(crate) use ported::shape_stroke;

/// How a family of symbols is constructed and edited.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Family {
    /// A line labelled "PL T" at both ends.
    PhaseLine,
    /// A closed area labelled "`prefix` T" at its centre.
    LabelledArea {
        /// Label prefix, e.g. "NAI".
        prefix: &'static str,
    },
    /// An axis of advance with a notched arrowhead (Main Attack).
    Axis,
    /// An air corridor of width `AM`.
    Corridor,
    /// A sector range fan from `AM` ranges and `AN` azimuths.
    RangeFanSector,
    /// An obstacle bypass box with arrowheads at the opening.
    Bypass,
    /// Drawn by the ported upstream renderer.
    Ported,
}

impl Family {
    /// Whether control points may be inserted and deleted.
    pub(crate) fn allows_vertex_edits(self) -> bool {
        match self {
            Self::PhaseLine | Self::LabelledArea { .. } | Self::Corridor => true,
            Self::Axis | Self::RangeFanSector | Self::Bypass | Self::Ported => false,
        }
    }
}

/// Construction settings.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct Config {
    /// Limits on input and output size.
    pub budget: Budget,
    /// Longest straight piece of a densified geodesic edge, in metres. At
    /// 10 km a chord drawn in Web Mercator strays from the geodesic by at
    /// most a few metres at mid latitudes.
    pub geodesic_step_m: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            budget: Budget::default(),
            geodesic_step_m: 10_000.0,
        }
    }
}

/// Why a construction could not be built.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ConstructError {
    /// The symbol is not supported.
    #[error(transparent)]
    Unsupported(#[from] Unsupported),
    /// The control-point count is outside what the symbol allows.
    #[error("{symbol} needs {min}..={max} control points; {count} given")]
    PointCount {
        /// Symbol name.
        symbol: &'static str,
        /// Points given.
        count: usize,
        /// Minimum.
        min: usize,
        /// Maximum.
        max: usize,
    },
    /// An amplifier is set that the symbol does not draw.
    #[error("{symbol} does not support amplifier {field}")]
    UnsupportedModifier {
        /// Symbol name.
        symbol: &'static str,
        /// The field.
        field: ModifierField,
    },
    /// An amplifier this version does not model is set.
    #[error("{symbol} does not support amplifier {key}")]
    UnknownModifier {
        /// Symbol name.
        symbol: &'static str,
        /// The stored key.
        key: String,
    },
    /// A required amplifier is missing.
    #[error("{symbol} requires amplifier {field}")]
    MissingModifier {
        /// Symbol name.
        symbol: &'static str,
        /// The field.
        field: ModifierField,
    },
    /// An amplifier holds more or fewer values than the symbol takes.
    #[error("{symbol} takes {min}..={max} values of amplifier {field}; {count} given")]
    ModifierCount {
        /// Symbol name.
        symbol: &'static str,
        /// The field.
        field: ModifierField,
        /// Values given.
        count: usize,
        /// Minimum.
        min: usize,
        /// Maximum.
        max: usize,
    },
    /// An amplifier value is not usable.
    #[error("amplifier {field} value {value} at position {index} is invalid: {reason}")]
    InvalidModifier {
        /// The field.
        field: ModifierField,
        /// Position of the value in the field.
        index: usize,
        /// The value.
        value: f64,
        /// Why.
        reason: &'static str,
    },
    /// The control points do not define the shape (e.g. coincident points).
    #[error("{symbol}: {reason}")]
    Degenerate {
        /// Symbol name.
        symbol: &'static str,
        /// Why.
        reason: &'static str,
    },
    /// A control point has an altitude. Graphics are drawn clamped to the
    /// ground; drawing an altitude there would misplace it, so it is refused
    /// until altitudes and their datums are carried through construction.
    #[error(
        "{symbol}: control point {index} has an altitude; only ground-clamped points are supported"
    )]
    UnsupportedAltitude {
        /// Symbol name.
        symbol: &'static str,
        /// Index of the first such point.
        index: usize,
    },
    /// A size limit would be exceeded.
    #[error(transparent)]
    Budget(#[from] BudgetError),
    /// The symbol's renderer draws nothing for these control points and
    /// amplifiers.
    #[error("{symbol} cannot be drawn: {reason}")]
    Unrenderable {
        /// Symbol name.
        symbol: &'static str,
        /// Why.
        reason: String,
    },
}

/// Validates `definition` and builds its geographic construction.
pub fn construct(
    definition: &GraphicDefinition,
    config: &Config,
) -> Result<Construction, ConstructError> {
    let spec = support::spec(&definition.symbol)?;
    validate::validate(spec, definition, &config.budget)?;
    let palette: Palette = style::palette(&definition.symbol, &definition.style);
    let earth = Earth::wgs84();
    let mut ctx = Ctx::new(&earth, config, palette, spec);
    match spec.family {
        Family::PhaseLine => phase_line::construct(&mut ctx, definition)?,
        Family::LabelledArea { prefix } => area::construct(&mut ctx, definition, prefix)?,
        Family::Axis => axis::construct(&mut ctx, definition)?,
        Family::Corridor => corridor::construct(&mut ctx, definition)?,
        Family::RangeFanSector => range_fan::construct(&mut ctx, definition)?,
        Family::Bypass => bypass::construct(&mut ctx, definition)?,
        Family::Ported => ported::construct(&mut ctx, definition)?,
    }
    ctx.finish(definition, RENDERER_VERSION)
}

/// Default handles: one per control point.
pub(crate) fn vertex_handles(definition: &GraphicDefinition) -> Vec<HandleSpec> {
    definition
        .positions()
        .enumerate()
        .map(|(i, at)| HandleSpec {
            id: HandleId::Vertex(i as u16),
            kind: HandleKind::Vertex,
            at,
        })
        .collect()
}

/// Applies a handle move according to the symbol's family.
pub(crate) fn move_handle(
    spec: &SymbolSpec,
    definition: &mut GraphicDefinition,
    handle: HandleId,
    to: GeoPoint,
) -> Result<(), EditError> {
    match (spec.family, handle) {
        (_, HandleId::Vertex(i)) => edit::move_vertex(definition, i, to),
        (Family::RangeFanSector, HandleId::Range(_) | HandleId::Azimuth(_)) => {
            range_fan::move_handle(definition, handle, to)
        }
        (Family::Corridor, HandleId::Width) => {
            let first = definition
                .positions()
                .next()
                .ok_or(EditError::NoSuchHandle { handle })?;
            let half = Earth::wgs84().inverse(first, to).distance_m;
            definition.modifiers.distances_m = vec![2.0 * half];
            Ok(())
        }
        _ => Err(EditError::NoSuchHandle { handle }),
    }
}
