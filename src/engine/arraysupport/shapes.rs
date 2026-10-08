//! Placeholder for the shape-building switch.

use super::work::Work;
use crate::engine::base::{EngineError, Shape};

/// Builds the shapes of the line type from the finished points.
pub(crate) fn build(w: &mut Work<'_>, shapes: &mut Vec<Shape>) -> Result<(), EngineError> {
    let _ = (w, shapes);
    Ok(())
}
