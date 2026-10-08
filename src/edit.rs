//! Pure edit operations: a definition and an edit give a new definition.

use serde::{Deserialize, Serialize};

use crate::definition::{ControlPoint, GraphicDefinition};
use crate::geo::GeoPoint;
use crate::support::{self, Unsupported};

#[cfg(test)]
mod tests;

/// Identity of an edit handle, stable across renders of one revision.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum HandleId {
    /// Control point `n` (0-based).
    Vertex(u16),
    /// The width of an axis or corridor.
    Width,
    /// Range value `n` of `AM`.
    Range(u16),
    /// Azimuth value `n` of `AN`.
    Azimuth(u16),
}

/// A change requested by the user, in geographic terms. Hosts convert the
/// cursor to a geographic position with their projection before editing.
#[derive(Clone, Debug, PartialEq)]
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
        index: u16,
        /// The new point.
        at: GeoPoint,
    },
    /// Remove control point `index`.
    DeleteVertex {
        /// Position in the control-point list.
        index: u16,
    },
}

/// Why an edit was refused. The original definition is never modified.
#[derive(Debug, thiserror::Error)]
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

/// Applies `edit` to `definition`, returning the edited copy with its
/// revision advanced.
///
/// The result is checked like a construction with the default
/// [`crate::Config`], so an accepted edit never yields a graphic that cannot
/// be drawn; a refused edit leaves nothing changed.
pub fn apply_edit(
    definition: &GraphicDefinition,
    edit: &Edit,
) -> Result<GraphicDefinition, EditError> {
    let spec = support::spec(&definition.symbol)?;
    let mut next = definition.clone();
    match *edit {
        Edit::Move { handle, to } => crate::family::move_handle(spec, &mut next, handle, to)?,
        Edit::InsertVertex { index, at } => {
            let index = usize::from(index);
            if index > next.points.len() || !spec.family.allows_vertex_edits() {
                return Err(EditError::NoSuchHandle {
                    handle: HandleId::Vertex(index as u16),
                });
            }
            next.points.insert(index, ControlPoint::ground(at));
        }
        Edit::DeleteVertex { index } => {
            let at = usize::from(index);
            if at >= next.points.len() || !spec.family.allows_vertex_edits() {
                return Err(EditError::NoSuchHandle {
                    handle: HandleId::Vertex(index),
                });
            }
            next.points.remove(at);
        }
    }
    let count = next.points.len();
    if count < spec.min_points || count > spec.max_points {
        return Err(EditError::PointCount {
            count,
            min: spec.min_points,
            max: spec.max_points,
        });
    }
    // The edited graphic must be one that can be drawn: the same checks as
    // construction (amplifier values, geometry) decide, before the revision
    // advances.
    crate::family::construct(&next, &crate::family::Config::default())
        .map_err(EditError::Invalid)?;
    next.revision = next.revision.wrapping_add(1);
    Ok(next)
}

/// Moves control point `index` to `to`, keeping its altitude.
pub(crate) fn move_vertex(
    definition: &mut GraphicDefinition,
    index: u16,
    to: GeoPoint,
) -> Result<(), EditError> {
    let point = definition
        .points
        .get_mut(usize::from(index))
        .ok_or(EditError::NoSuchHandle {
            handle: HandleId::Vertex(index),
        })?;
    point.position = to;
    Ok(())
}
