//! Pure edit operations: a definition and an edit give a new definition.

use serde::{Deserialize, Serialize};

use crate::construction::Construction;
use crate::definition::{ControlPoint, GraphicDefinition};
use crate::family::Config;
use crate::geo::GeoPoint;
use crate::modifier::{ModifierField, ModifierValue, ModifierValueError, Modifiers};
use crate::support::{self, SymbolSpec, Unsupported};

#[cfg(test)]
mod tests;

/// Identity of an edit handle, stable across renders of one revision.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[non_exhaustive]
pub enum HandleId {
    /// Control point `n` (0-based).
    Vertex(u32),
    /// The width of an axis or corridor.
    Width,
    /// Range value `n` of `AM`.
    Range(u32),
    /// Azimuth value `n` of `AN`.
    Azimuth(u32),
}

/// A change requested by the user, in geographic terms. Hosts convert the
/// cursor to a geographic position with their projection before editing.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Edit {
    /// Drag a handle to a new position.
    Move {
        /// The handle.
        handle: HandleId,
        /// Where it was dropped.
        to: GeoPoint,
    },
    /// Insert a control point before index `index`.
    InsertVertex {
        /// Position in the control-point list.
        index: u32,
        /// The new point.
        at: GeoPoint,
    },
    /// Remove control point `index`.
    DeleteVertex {
        /// Position in the control-point list.
        index: u32,
    },
    /// Set one amplifier, or clear it with `None`.
    SetModifier {
        /// The field.
        field: ModifierField,
        /// Its new value.
        value: Option<ModifierValue>,
    },
    /// Replace every amplifier at once, as a form editor does on save.
    SetModifiers(Box<Modifiers>),
}

/// Why an edit was refused. The original definition is never modified.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum EditError {
    /// The symbol is not supported, so its edit rules are unknown.
    #[error(transparent)]
    Unsupported(#[from] Unsupported),
    /// The handle does not exist on this graphic.
    #[error("graphic has no handle {handle:?}")]
    NoSuchHandle {
        /// The requested handle.
        handle: HandleId,
    },
    /// The symbol takes a fixed set of control points, so none can be
    /// inserted or deleted.
    #[error("{symbol} does not allow inserting or deleting control points")]
    VertexEditsNotAllowed {
        /// Symbol name.
        symbol: &'static str,
    },
    /// The value's shape does not fit the amplifier field.
    #[error(transparent)]
    Modifier(#[from] ModifierValueError),
    /// The edited graphic could not be constructed (e.g. a zero width).
    #[error("edit would leave an invalid graphic: {0}")]
    Invalid(#[source] crate::family::ConstructError),
    /// The edit would leave a point count the symbol does not allow.
    #[error("edit would leave {count} control points; the symbol allows {min}..={max}")]
    PointCount {
        /// Points after the edit.
        count: usize,
        /// Minimum allowed.
        min: usize,
        /// Maximum allowed.
        max: usize,
    },
}

/// What an edit is checked against: the configuration its result is
/// constructed with.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct EditContext {
    config: Config,
}

impl EditContext {
    /// A context that constructs edited graphics with `config`, which
    /// should be the configuration the host draws them with.
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// The construction configuration.
    pub fn config(&self) -> &Config {
        &self.config
    }
}

/// An accepted edit: the new definition and its construction, which was
/// built to check the edit and can be cached in place of the old one.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Edited {
    /// The edited definition, its revision advanced.
    pub definition: GraphicDefinition,
    /// Its construction under the context's configuration.
    pub construction: Construction,
}

/// Applies `edit` to `definition`, returning the edited copy with its
/// revision advanced and its construction.
///
/// The result is constructed with the context's configuration, so an
/// accepted edit never yields a graphic that cannot be drawn; a refused
/// edit leaves nothing changed.
pub fn apply_edit(
    definition: &GraphicDefinition,
    edit: &Edit,
    context: &EditContext,
) -> Result<Edited, EditError> {
    let spec = support::spec(&definition.symbol)?;
    let mut next = definition.clone();
    change(spec, &mut next, edit)?;
    let count = next.points.len();
    if count < spec.min_points || count > spec.max_points {
        return Err(EditError::PointCount {
            count,
            min: spec.min_points,
            max: spec.max_points,
        });
    }
    next.revision = next.revision.wrapping_add(1);
    // The edited graphic must be one that can be drawn: the same checks as
    // construction (amplifier values, geometry) decide.
    let construction =
        crate::family::construct(&next, &context.config).map_err(EditError::Invalid)?;
    Ok(Edited {
        definition: next,
        construction,
    })
}

/// Applies `edit` to `next` without checking the result.
fn change(spec: &SymbolSpec, next: &mut GraphicDefinition, edit: &Edit) -> Result<(), EditError> {
    match edit {
        Edit::Move { handle, to } => crate::family::move_handle(spec, next, *handle, *to)?,
        Edit::InsertVertex { index, at } => {
            vertex_edits(spec)?;
            let at_index = usize::try_from(*index)
                .ok()
                .filter(|i| *i <= next.points.len());
            let Some(i) = at_index else {
                return Err(EditError::NoSuchHandle {
                    handle: HandleId::Vertex(*index),
                });
            };
            next.points.insert(i, ControlPoint::ground(*at));
        }
        Edit::DeleteVertex { index } => {
            vertex_edits(spec)?;
            let at_index = usize::try_from(*index)
                .ok()
                .filter(|i| *i < next.points.len());
            let Some(i) = at_index else {
                return Err(EditError::NoSuchHandle {
                    handle: HandleId::Vertex(*index),
                });
            };
            next.points.remove(i);
        }
        Edit::SetModifier { field, value: None } => next.modifiers.clear(*field),
        Edit::SetModifier {
            field,
            value: Some(value),
        } => next.modifiers.set(*field, value.clone())?,
        Edit::SetModifiers(modifiers) => next.modifiers = Modifiers::clone(modifiers),
    }
    Ok(())
}

fn vertex_edits(spec: &SymbolSpec) -> Result<(), EditError> {
    if spec.allows_vertex_edits() {
        Ok(())
    } else {
        Err(EditError::VertexEditsNotAllowed {
            symbol: spec.name(),
        })
    }
}

/// Moves control point `index` to `to`, keeping its altitude.
pub(crate) fn move_vertex(
    definition: &mut GraphicDefinition,
    index: u32,
    to: GeoPoint,
) -> Result<(), EditError> {
    let point = usize::try_from(index)
        .ok()
        .and_then(|i| definition.points.get_mut(i))
        .ok_or(EditError::NoSuchHandle {
            handle: HandleId::Vertex(index),
        })?;
    point.position = to;
    Ok(())
}
