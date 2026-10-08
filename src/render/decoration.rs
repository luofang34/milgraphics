//! Resolving pixel-sized decorations for a view.

use crate::budget::BudgetError;
use crate::construction::{Decoration, DecorationSize, PartRole, ScreenDecoration};
use crate::pick::{PickRef, PickTarget};
use crate::render::screen::ScreenCtx;
use crate::render::{ScreenItem, ScreenPoint, ScreenShape};
use crate::style::Fill;

pub(crate) fn resolve(
    ctx: &mut ScreenCtx<'_>,
    decoration: &ScreenDecoration,
    pick: &impl Fn(PickTarget) -> PickRef,
) -> Result<Option<ScreenItem>, BudgetError> {
    let item = match &decoration.0 {
        Decoration::Engine { .. } => return Ok(None),
        Decoration::Glyph {
            id,
            anchor,
            offset_px,
            points,
            closed,
            stroke,
        } => {
            let Some(at) = ctx.project(*anchor) else {
                return Ok(None);
            };
            glyph(
                pick(PickTarget::Part(*id)),
                at,
                *offset_px,
                points,
                (*closed, *stroke),
            )
        }
        &Decoration::Arrowhead {
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
            let points = resolve_size(ctx, size).and_then(|px| arrowhead(t, w, px, half_angle_deg));
            let Some(points) = points else {
                return Ok(None);
            };
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
        &Decoration::Pointer {
            id,
            from,
            through,
            head_px,
            stroke,
        } => {
            let (Some(f), Some(t)) = (ctx.project(from), ctx.project(through)) else {
                return Ok(None);
            };
            let Some(shape) = pointer(f, t, head_px) else {
                return Ok(None);
            };
            ScreenItem {
                pick: pick(PickTarget::Part(id)),
                role: PartRole::Orientation,
                shape,
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

/// Two wings `size_px` long from tip `t`, opening toward `w`.
fn arrowhead(
    t: ScreenPoint,
    w: ScreenPoint,
    size_px: f64,
    half_angle_deg: f64,
) -> Option<Vec<ScreenPoint>> {
    let u = unit(w.sub(t))?;
    let (s, c) = half_angle_deg.to_radians().sin_cos();
    let wing = |sign: f64| ScreenPoint {
        x: t.x + size_px * (u.0 * c - sign * u.1 * s),
        y: t.y + size_px * (sign * u.0 * s + u.1 * c),
    };
    Some(vec![wing(1.0), t, wing(-1.0)])
}

/// A line from `f` past `t` to an open arrowhead `head_px` long.
fn pointer(f: ScreenPoint, t: ScreenPoint, head_px: f64) -> Option<ScreenShape> {
    let (dx, dy) = t.sub(f);
    let u = unit((dx, dy))?;
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
    Some(ScreenShape::Polyline(vec![
        f,
        tip,
        at(1.0, -1.0),
        tip,
        at(1.0, 1.0),
    ]))
}

/// A glyph's outline placed at `at`.
fn glyph(
    pick: PickRef,
    at: ScreenPoint,
    offset_px: [f64; 2],
    points: &[[f64; 2]],
    (closed, stroke): (bool, crate::style::Stroke),
) -> ScreenItem {
    let points: Vec<ScreenPoint> = points
        .iter()
        .map(|p| ScreenPoint {
            x: at.x + offset_px[0] + p[0],
            y: at.y + offset_px[1] + p[1],
        })
        .collect();
    ScreenItem {
        pick,
        role: PartRole::Decoration,
        shape: if closed {
            ScreenShape::Polygon(points)
        } else {
            ScreenShape::Polyline(points)
        },
        stroke: Some(stroke),
        fill: Fill::None,
        decoration: true,
    }
}

fn unit((dx, dy): (f64, f64)) -> Option<(f64, f64)> {
    let len = dx.hypot(dy);
    (len.is_finite() && len > 0.0).then(|| (dx / len, dy / len))
}

fn resolve_size(ctx: &mut ScreenCtx<'_>, size: DecorationSize) -> Option<f64> {
    match size {
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
