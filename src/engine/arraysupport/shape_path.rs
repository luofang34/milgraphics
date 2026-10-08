//! `java.awt.geom.GeneralPath` semantics for [`Shape`] paths, as `Shape2`
//! uses them: two moves in a row keep only the second, and a line needs a
//! current point.

use crate::engine::base::{EngineError, PathOp, Pt, Shape};
use crate::style::Rgba;

/// `java.awt.Color.GREEN`.
pub(crate) const GREEN: Rgba = Rgba::opaque(0, 255, 0);
/// `java.awt.Color.BLUE`.
pub(crate) const BLUE: Rgba = Rgba::opaque(0, 0, 255);
/// `java.awt.Color.RED`.
pub(crate) const RED: Rgba = Rgba::opaque(255, 0, 0);
/// `java.awt.Color.WHITE`.
pub(crate) const WHITE: Rgba = Rgba::opaque(255, 255, 255);

/// `Shape2.moveTo`: starts a subpath, replacing a move that has no line
/// after it.
pub(crate) fn move_to(shape: &mut Shape, p: Pt) {
    if let Some(last) = shape.path.last_mut() {
        if matches!(last, PathOp::MoveTo(..)) {
            *last = PathOp::MoveTo(p.x, p.y);
            return;
        }
    }
    shape.move_to(p);
}

/// `Shape2.lineTo`: Java throws `IllegalPathStateException` when the path
/// has no current point.
pub(crate) fn line_to(shape: &mut Shape, p: Pt) -> Result<(), EngineError> {
    if shape.path.is_empty() {
        return Err(EngineError::Degenerate("lineTo without a current point"));
    }
    shape.line_to(p);
    Ok(())
}
