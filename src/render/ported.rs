//! Drawing graphics of the ported upstream renderer for a view.

mod clip;
#[cfg(test)]
mod tests;

use crate::budget::BudgetError;
use crate::construction::{Decoration, PartRole};
use crate::engine::api::{self, Input, Justify};
use crate::engine::base::{Pt, Shape};
use crate::family::{shape_fill, shape_outline};
use crate::pick::{PickRef, PickTarget};
use crate::render::label::{self, Label, TextAlign};
use crate::render::screen::ScreenCtx;
use crate::render::{Font, FontMetrics, ScreenItem, ScreenPoint, ScreenShape};
use crate::style::{Fill, Rgba};

/// Ground distance over which the view's scale at a graphic is measured.
const SCALE_PROBE_M: f64 = 100.0;

/// Screen items and labels of an engine decoration; nothing when a control
/// point is hidden, since the renderer's pixel geometry cannot be clipped at
/// the horizon.
pub(crate) fn resolve(
    ctx: &mut ScreenCtx<'_>,
    decoration: &Decoration,
    font: &Font,
    metrics: &dyn FontMetrics,
    pick: &impl Fn(PickTarget) -> PickRef,
) -> Result<(Vec<ScreenItem>, Vec<Label>), BudgetError> {
    let Decoration::Engine {
        line_type,
        anchors,
        symbol,
        modifiers,
        style,
        geographic,
        shape_count,
        part,
        ..
    } = decoration
    else {
        return Ok((Vec::new(), Vec::new()));
    };
    let projected: Option<Vec<ScreenPoint>> = anchors.iter().map(|&a| ctx.project(a)).collect();
    let (Some(projected), Some(&first)) = (projected, anchors.first()) else {
        return Ok((Vec::new(), Vec::new()));
    };
    let Some(mpp) = meters_per_pixel(ctx, first) else {
        return Ok((Vec::new(), Vec::new()));
    };
    let width = |t: &str| metrics.text_width_px(font, t);
    let (mut items, mut labels) = (Vec::new(), Vec::new());
    for run in clip::runs(*line_type, projected, ctx.viewport()) {
        let input = Input {
            line_type: *line_type,
            symbol,
            pixels: run.iter().map(|p| Pt::new(p.x, p.y)).collect(),
            modifiers,
            meters_per_pixel: mpp,
            text_width: &width,
            ms_info: crate::family::ms_info(symbol),
            style: *style,
        };
        let Ok(out) = api::draw(&input) else {
            continue;
        };
        let paired = out.shapes.len() == *shape_count;
        for (i, shape) in out.shapes.iter().enumerate() {
            if paired && geographic.get(i).copied().unwrap_or(false) {
                continue;
            }
            items.extend(screen_items(ctx, shape, pick(PickTarget::Part(*part)))?);
        }
        labels.extend(placed_labels(
            ctx,
            out.labels,
            first,
            font,
            metrics,
            &|| pick(PickTarget::Part(*part)),
        ));
    }
    Ok((items, labels))
}

fn placed_labels(
    ctx: &mut ScreenCtx<'_>,
    labels: Vec<api::Label>,
    first: crate::geo::GeoPoint,
    font: &Font,
    metrics: &dyn FontMetrics,
    pick: &dyn Fn() -> PickRef,
) -> Vec<Label> {
    labels
        .into_iter()
        .map(|l| {
            let screen = ScreenPoint { x: l.x, y: l.y };
            let anchor = ctx.unproject(screen).unwrap_or(first);
            let align = match l.justify {
                Justify::Left => TextAlign::Left,
                Justify::Center => TextAlign::Center,
                Justify::Right => TextAlign::Right,
            };
            label::at_screen(
                label::PlacedText {
                    text: l.text,
                    screen,
                    anchor,
                    rotation_deg: l.angle_deg,
                    align,
                    knockout: l.knockout,
                },
                font,
                metrics,
                pick(),
            )
        })
        .collect()
}

/// Ground metres per screen pixel eastward of `at`.
fn meters_per_pixel(ctx: &mut ScreenCtx<'_>, at: crate::geo::GeoPoint) -> Option<f64> {
    let east = ctx.earth().direct(at, 90.0, SCALE_PROBE_M);
    let (a, b) = (ctx.project(at)?, ctx.project(east)?);
    let (dx, dy) = b.sub(a);
    let px = dx.hypot(dy);
    (px.is_finite() && px > 0.0).then(|| SCALE_PROBE_M / px)
}

fn screen_items(
    ctx: &mut ScreenCtx<'_>,
    shape: &Shape,
    pick: PickRef,
) -> Result<Vec<ScreenItem>, BudgetError> {
    let mut items = Vec::new();
    for line in shape.polylines() {
        ctx.take(line.len())?;
        let mut points: Vec<ScreenPoint> = line
            .into_iter()
            .map(|(x, y)| ScreenPoint { x, y })
            .collect();
        let outline = shape_outline(shape, Rgba::BLACK);
        let fill = shape_fill(shape);
        let shape = if fill == Fill::None {
            ScreenShape::Polyline(points)
        } else {
            if points.len() > 1 && points.first() == points.last() {
                points.pop();
            }
            ScreenShape::Polygon(points)
        };
        let item = ScreenItem {
            pick: pick.clone(),
            role: PartRole::Decoration,
            shape,
            stroke: outline,
            fill,
            decoration: true,
        };
        items.push(item);
    }
    Ok(items)
}
