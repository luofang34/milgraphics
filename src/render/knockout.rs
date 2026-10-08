//! Gaps in a graphic's outline under the labels the standard sets into it.

use crate::budget::BudgetError;
use crate::render::screen::ScreenCtx;
use crate::render::{ScreenItem, ScreenPoint, ScreenShape};
use crate::style::Fill;

/// `items` with the stroke of every unfilled item left out inside `boxes`.
/// A filled item keeps its outline, since a gap would open its interior.
pub(crate) fn cut(
    ctx: &mut ScreenCtx<'_>,
    items: Vec<ScreenItem>,
    boxes: &[[ScreenPoint; 4]],
) -> Result<Vec<ScreenItem>, BudgetError> {
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        if item.fill != Fill::None || item.stroke.is_none() {
            out.push(item);
            continue;
        }
        let mut path = item.shape.points().to_vec();
        if item.shape.is_closed() {
            if let Some(&first) = path.first() {
                path.push(first);
            }
        }
        let pieces = pieces(&path, boxes);
        // An item the boxes miss keeps its own shape.
        if let [only] = pieces.as_slice() {
            if only.len() == path.len() {
                out.push(item);
                continue;
            }
        }
        for piece in pieces {
            ctx.take(piece.len())?;
            out.push(ScreenItem {
                shape: ScreenShape::Polyline(piece),
                ..item.clone()
            });
        }
    }
    Ok(out)
}

/// The runs of `path` outside every box.
fn pieces(path: &[ScreenPoint], boxes: &[[ScreenPoint; 4]]) -> Vec<Vec<ScreenPoint>> {
    let mut out: Vec<Vec<ScreenPoint>> = Vec::new();
    for pair in path.windows(2) {
        let [a, b] = pair else { continue };
        for [p, q] in super::hatch::outside([*a, *b], boxes) {
            match out.last_mut() {
                Some(run) if run.last() == Some(&p) => run.push(q),
                _ => out.push(vec![p, q]),
            }
        }
    }
    out
}

#[cfg(test)]
mod tests;
