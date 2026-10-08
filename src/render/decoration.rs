//! Resolving pixel-sized decorations for a view.

use crate::budget::BudgetError;
use crate::construction::{PartId, PartRole, ScreenDecoration};
use crate::edit::HandleId;
use crate::pick::PickRef;
use crate::render::screen::ScreenCtx;
use crate::render::{ScreenItem, ScreenPoint, ScreenShape};
use crate::style::Fill;

pub(crate) fn resolve(
    ctx: &mut ScreenCtx<'_>,
    decoration: &ScreenDecoration,
    pick: &impl Fn(PartId, Option<HandleId>) -> PickRef,
) -> Result<Option<ScreenItem>, BudgetError> {
    match *decoration {
        ScreenDecoration::Arrowhead {
            id,
            tip,
            toward,
            size_px,
            half_angle_deg,
            filled,
            stroke,
        } => {
            let (Some(t), Some(w)) = (ctx.project(tip), ctx.project(toward)) else {
                return Ok(None);
            };
            let (dx, dy) = w.sub(t);
            let len = dx.hypot(dy);
            if len.is_nan() || len <= 0.0 {
                return Ok(None);
            }
            let (ux, uy) = (dx / len, dy / len);
            let (s, c) = half_angle_deg.to_radians().sin_cos();
            let wing = |sign: f64| ScreenPoint {
                x: t.x + size_px * (ux * c - sign * uy * s),
                y: t.y + size_px * (sign * ux * s + uy * c),
            };
            let points = vec![wing(1.0), t, wing(-1.0)];
            let (shape, fill) = if filled {
                (ScreenShape::Polygon(points), Fill::Solid(stroke.color))
            } else {
                (ScreenShape::Polyline(points), Fill::None)
            };
            Ok(Some(ScreenItem {
                pick: pick(id, None),
                role: PartRole::Arrowhead,
                shape,
                stroke: Some(stroke),
                fill,
            }))
        }
    }
}
