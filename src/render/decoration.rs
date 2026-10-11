//! Resolving pixel-sized decorations for a view.

use crate::budget::BudgetError;
use crate::construction::{Decoration, DecorationSize, GeoGeometry, PartRole, ScreenDecoration};
use crate::geo::GeoPoint;
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
            let Some(shape) = pointer_on_ground(ctx, from, through, head_px)? else {
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

/// A pointer whose shaft is projected as a geographic line, so it lies on
/// the ground with the fan; the head continues the shaft's last direction.
fn pointer_on_ground(
    ctx: &mut ScreenCtx<'_>,
    from: GeoPoint,
    through: GeoPoint,
    head_px: f64,
) -> Result<Option<ScreenShape>, BudgetError> {
    let Some(t) = ctx.project(through) else {
        return Ok(None);
    };
    let shaft = ctx
        .part(&GeoGeometry::Line(vec![from, through]), Fill::None)?
        .into_iter()
        .map(|(shape, _)| match shape {
            ScreenShape::Polyline(p) | ScreenShape::Polygon(p) => p,
        })
        .find(|p| p.last() == Some(&t));
    Ok(shaft.and_then(|p| pointer(p, head_px)))
}

/// `shaft` extended past its end by an open arrowhead `head_px` long.
fn pointer(mut shaft: Vec<ScreenPoint>, head_px: f64) -> Option<ScreenShape> {
    let (&t, rest) = shaft.split_last()?;
    let &before = rest.last()?;
    let u = unit(t.sub(before))?;
    // Short pointers get a proportionally smaller head.
    let dist: f64 = shaft
        .windows(2)
        .map(|w| match w {
            [a, b] => {
                let (dx, dy) = b.sub(*a);
                dx.hypot(dy)
            }
            _ => 0.0,
        })
        .sum();
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
    shaft.extend([tip, at(1.0, -1.0), tip, at(1.0, 1.0)]);
    Some(ScreenShape::Polyline(shaft))
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
