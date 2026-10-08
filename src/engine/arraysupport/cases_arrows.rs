//! Placeholder for the arrows group of line types.

use super::work::Work;
use crate::engine::base::EngineError;

/// Builds the points of the line types of this group; false when the line
/// type belongs to another group.
pub(crate) fn build(w: &mut Work<'_>) -> Result<bool, EngineError> {
    let _ = w;
    Ok(false)
}
