//! Resolving pixel-sized decorations for a view.

use crate::budget::BudgetError;
use crate::construction::{DecorationSize, PartRole, ScreenDecoration};
use crate::pick::{PickRef, PickTarget};
use crate::render::screen::ScreenCtx;
use crate::render::{ScreenItem, ScreenPoint, ScreenShape};
use crate::style::Fill;

pub(crate) fn resolve(
    ctx: &mut ScreenCtx<'_>,
    decoration: &ScreenDecoration,
    pick: &impl Fn(PickTarget) -> PickRef,
) -> Result<Option<ScreenItem>, BudgetError> {
    let item = match *decoration {
        ScreenDecoration::Arrowhead {
            id,
            tip,
            toward,
            size,
            half_angle_deg,
            filled,
            stroke,
        } => {
            let (Some(t), Some(w)) = (ctx.project(tip), ctx.project(toward)) else {
                return Ok(None);
            };
            let Some(u) = unit(w.sub(t)) else {
                return Ok(None);
            };
            let Some(size_px) = resolve_size(ctx, size) else {
                return Ok(None);
            };
            let (s, c) = half_angle_deg.to_radians().sin_cos();
            let wing = |sign: f64| ScreenPoint {
                x: t.x + size_px * (u.0 * c - sign * u.1 * s),
                y: t.y + size_px * (sign * u.0 * s + u.1 * c),
            };
            let points = vec![wing(1.0), t, wing(-1.0)];
            let (shape, fill) = if filled {
                (ScreenShape::Polygon(points), Fill::Solid(stroke.color))
            } else {
                (ScreenShape::Polyline(points), Fill::None)
            };
            ScreenItem {
                pick: pick(PickTarget::Part(id)),
                role: PartRole::Arrowhead,
                shape,
                stroke: Some(stroke),
                fill,
                decoration: true,
            }
        }
        ScreenDecoration::Pointer {
            id,
            from,
            through,
            head_px,
            stroke,
        } => {
            let (Some(f), Some(t)) = (ctx.project(from), ctx.project(through)) else {
                return Ok(None);
            };
            let (dx, dy) = t.sub(f);
            let Some(u) = unit((dx, dy)) else {
                return Ok(None);
            };
            // Short pointers get a proportionally smaller head.
            let dist = dx.hypot(dy);
            let base = if dist < 10.0 * head_px {
                (dist / 10.0).max(head_px / 2.0)
            } else {
                head_px
            };
            let at = |k: f64, side: f64| ScreenPoint {
                x: t.x + u.0 * k * base - u.1 * side * base,
                y: t.y + u.1 * k * base + u.0 * side * base,
            };
            let tip = at(2.0, 0.0);
            ScreenItem {
                pick: pick(PickTarget::Part(id)),
                role: PartRole::Orientation,
                shape: ScreenShape::Polyline(vec![f, tip, at(1.0, -1.0), tip, at(1.0, 1.0)]),
                stroke: Some(stroke),
                fill: Fill::None,
                decoration: true,
            }
        }
    };
    let count = match &item.shape {
        ScreenShape::Polyline(p) | ScreenShape::Polygon(p) => p.len(),
    };
    ctx.take(count)?;
    Ok(Some(item))
}

fn unit((dx, dy): (f64, f64)) -> Option<(f64, f64)> {
    let len = dx.hypot(dy);
    (len.is_finite() && len > 0.0).then(|| (dx / len, dy / len))
}

fn resolve_size(ctx: &mut ScreenCtx<'_>, size: DecorationSize) -> Option<f64> {
    match size {
        DecorationSize::Px(px) => Some(px),
        DecorationSize::Proportional {
            segments,
            fraction,
            min_px,
            max_px,
        } => {
            let mut total = 0.0;
            for (a, b) in segments {
                let (dx, dy) = ctx.project(b)?.sub(ctx.project(a)?);
                total += dx.hypot(dy);
            }
            Some((total * fraction).clamp(min_px, max_px))
        }
    }
}
