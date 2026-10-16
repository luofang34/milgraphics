//! Drawing graphics of the ported upstream renderer for a view.

mod baseline;
mod clip;
mod frame;
mod labels;
#[cfg(test)]
mod tests;

use crate::budget::BudgetError;
use crate::construction::{Decoration, GeoGeometry, PartRole};
use crate::engine::api::{self, Input};
use crate::engine::base::{Pt, Shape};
use crate::family::{shape_fill, shape_outline};
use crate::pick::{PickRef, PickTarget};
use crate::render::label::Label;
use crate::render::screen::ScreenCtx;
use crate::render::{Font, FontMetrics, ScreenItem, ScreenPoint};
use crate::style::{Fill, Rgba};

use frame::Frame;

/// Screen items and labels of an engine decoration, drawn in the graphic's
/// frame for this view ([`frame`]) and projected vertex by vertex, so they
/// follow the ground and stop at the horizon like the geographic tier.
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
        baseline,
        stable,
        part,
        ..
    } = decoration
    else {
        return Ok((Vec::new(), Vec::new()));
    };
    let Some(frame) = Frame::new(ctx, anchors) else {
        return Ok((Vec::new(), Vec::new()));
    };
    let pixels: Vec<ScreenPoint> = anchors.iter().map(|&a| frame.to_px(a)).collect();
    let edges = baseline::paths(
        &pixels,
        baseline,
        stable
            .iter()
            .map(|c| c.iter().map(|&p| frame.to_px(p)).collect()),
    );
    let kind = (*line_type, clip::limits_itself(*line_type, symbol));
    let near = if kind.1 || clip::length(&pixels) > clip::MAX_LINE_PX {
        frame.near(ctx)
    } else {
        None
    };
    let width = |t: &str| metrics.text_width_px(font, t);
    let (mut items, mut labels) = (Vec::new(), Vec::new());
    for run in clip::runs(kind, pixels, near) {
        let input = Input {
            line_type: *line_type,
            symbol,
            pixels: run.iter().map(|p| Pt::new(p.x, p.y)).collect(),
            modifiers,
            meters_per_pixel: frame.meters_per_pixel(),
            text_width: &width,
            line_height: Some(metrics.line_height_px(font)),
            ms_info: crate::family::ms_info(symbol),
            style: *style,
            visible: near,
        };
        let Ok(out) = api::draw(&input) else {
            continue;
        };
        let paired = out.shapes.len() == *shape_count;
        for (i, shape) in out.shapes.iter().enumerate() {
            if paired && geographic.get(i).copied().unwrap_or(false) {
                continue;
            }
            let lines = if crate::family::ported_baseline::stroked(shape) {
                baseline::strip(shape.polylines(), edges.as_ref())
            } else {
                shape.polylines()
            };
            items.extend(screen_items(
                ctx,
                &frame,
                shape,
                lines,
                pick(PickTarget::Part(*part)),
            )?);
        }
        labels.extend(labels::placed(
            ctx,
            &frame,
            &run,
            out.labels,
            font,
            metrics,
            &|| pick(PickTarget::Part(*part)),
        ));
    }
    Ok((items, labels))
}

/// Lines `lines` of an engine shape as screen items, each vertex carried to
/// the ground and the edges between them projected like geographic lines.
fn screen_items(
    ctx: &mut ScreenCtx<'_>,
    frame: &Frame<'_>,
    shape: &Shape,
    lines: Vec<Vec<(f64, f64)>>,
    pick: PickRef,
) -> Result<Vec<ScreenItem>, BudgetError> {
    let mut items = Vec::new();
    let outline = shape_outline(shape, Rgba::BLACK);
    let fill = shape_fill(shape);
    for line in lines {
        ctx.take(line.len())?;
        let mut ground: Vec<_> = line.into_iter().map(|p| frame.to_geo(p)).collect();
        let geometry = if fill == Fill::None {
            GeoGeometry::Line(ground)
        } else {
            if ground.len() > 1 && ground.first() == ground.last() {
                ground.pop();
            }
            GeoGeometry::Ring(ground)
        };
        for (shape, fill) in ctx.dense_part(&geometry, fill)? {
            items.push(ScreenItem {
                pick: pick.clone(),
                role: PartRole::Decoration,
                shape,
                stroke: outline,
                fill,
                decoration: true,
                depth: Vec::new(),
            });
        }
    }
    Ok(items)
}
