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
mod context;
mod phase_line;

pub(crate) use context::Ctx;

/// How a family of symbols is constructed and edited.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Family {
    /// A line labelled "PL T" at both ends.
    PhaseLine,
    /// A closed area labelled "`prefix` T" at its centre.
    LabelledArea {
        /// Label prefix, e.g. "NAI".
        prefix: &'static str,
    },
}

impl Family {
    /// Whether control points may be inserted and deleted.
    pub fn allows_vertex_edits(self) -> bool {
        match self {
            Self::PhaseLine | Self::LabelledArea { .. } => true,
        }
    }
}

/// Construction settings.
#[derive(Clone, Copy, Debug, PartialEq)]
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
        /// Field name.
        field: String,
    },
    /// A required amplifier is missing.
    #[error("{symbol} requires amplifier {field}")]
    MissingModifier {
        /// Symbol name.
        symbol: &'static str,
        /// Field name.
        field: &'static str,
    },
    /// An amplifier value is not usable.
    #[error("amplifier {field} value {value} is invalid: {reason}")]
    InvalidModifier {
        /// Field name.
        field: &'static str,
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
    /// A size limit would be exceeded.
    #[error(transparent)]
    Budget(#[from] BudgetError),
}

/// Validates `definition` and builds its geographic construction.
pub fn construct(
    definition: &GraphicDefinition,
    config: &Config,
) -> Result<Construction, ConstructError> {
    let spec = support::spec(&definition.symbol)?;
    validate(spec, definition, &config.budget)?;
    let palette: Palette = style::palette(&definition.symbol, &definition.style);
    let mut ctx = Ctx::new(Earth::wgs84(), config, palette, spec);
    match spec.family {
        Family::PhaseLine => phase_line::construct(&mut ctx, definition)?,
        Family::LabelledArea { prefix } => area::construct(&mut ctx, definition, prefix)?,
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
        (Family::PhaseLine | Family::LabelledArea { .. }, _) => {
            Err(EditError::NoSuchHandle { handle })
        }
    }
}

fn validate(
    spec: &SymbolSpec,
    def: &GraphicDefinition,
    budget: &Budget,
) -> Result<(), ConstructError> {
    let count = def.points.len();
    if count > budget.max_control_points {
        return Err(BudgetError::ControlPoints {
            count,
            limit: budget.max_control_points,
        }
        .into());
    }
    if count < spec.min_points || count > spec.max_points {
        return Err(ConstructError::PointCount {
            symbol: spec.name,
            count,
            min: spec.min_points,
            max: spec.max_points,
        });
    }
    let m = &def.modifiers;
    if let Some(field) = m.unknown.keys().next() {
        return Err(ConstructError::UnsupportedModifier {
            symbol: spec.name,
            field: field.clone(),
        });
    }
    for field in ModifierField::ALL {
        if field.is_set(m) && !spec.modifiers.contains(&field) {
            return Err(ConstructError::UnsupportedModifier {
                symbol: spec.name,
                field: field.name().to_owned(),
            });
        }
    }
    if let Some(field) = spec.required.iter().find(|f| !f.is_set(m)) {
        return Err(ConstructError::MissingModifier {
            symbol: spec.name,
            field: field.name(),
        });
    }
    validate_sizes(def, budget)
}

fn validate_sizes(def: &GraphicDefinition, budget: &Budget) -> Result<(), ConstructError> {
    let m = &def.modifiers;
    let texts = [
        ("T", &m.designation),
        ("T1", &m.designation2),
        ("H", &m.additional_info),
        ("W", &m.dtg_start),
        ("W1", &m.dtg_end),
    ];
    for (field, text) in texts {
        let count = text.as_deref().map_or(0, |t| t.chars().count());
        if count > budget.max_text_chars {
            let limit = budget.max_text_chars;
            return Err(BudgetError::Text {
                field,
                count,
                limit,
            }
            .into());
        }
    }
    let lists = [
        ("AM", m.distances_m.len()),
        ("AN", m.azimuths_deg.len()),
        ("X", m.altitudes.len()),
    ];
    for (field, count) in lists {
        if count > budget.max_modifier_values {
            let limit = budget.max_modifier_values;
            return Err(BudgetError::ModifierValues {
                field,
                count,
                limit,
            }
            .into());
        }
    }
    for &value in &m.distances_m {
        if !(value.is_finite() && value >= 0.0) {
            let reason = "distances must be finite and non-negative";
            return Err(ConstructError::InvalidModifier {
                field: "AM",
                value,
                reason,
            });
        }
    }
    for &value in m
        .azimuths_deg
        .iter()
        .chain(m.altitudes.iter().map(|a| &a.metres))
    {
        if !value.is_finite() {
            let reason = "must be finite";
            return Err(ConstructError::InvalidModifier {
                field: "AN/X",
                value,
                reason,
            });
        }
    }
    Ok(())
}
