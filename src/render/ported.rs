//! Drawing graphics of the ported upstream renderer for a view.

mod clip;
mod frame;
#[cfg(test)]
mod tests;

use crate::budget::BudgetError;
use crate::construction::{Decoration, GeoGeometry, PartRole};
use crate::engine::api::{self, Input, Justify};
use crate::engine::base::{Pt, Shape};
use crate::family::{shape_fill, shape_outline};
use crate::pick::{PickRef, PickTarget};
use crate::render::label::{self, Label, TextAlign};
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
            items.extend(screen_items(
                ctx,
                &frame,
                shape,
                pick(PickTarget::Part(*part)),
            )?);
        }
        labels.extend(placed_labels(
            ctx,
            &frame,
            out.labels,
            font,
            metrics,
            &|| pick(PickTarget::Part(*part)),
        ));
    }
    Ok((items, labels))
}

/// Labels at their ground points, turned as the line they were set along
/// runs on screen; a label whose point is hidden is not drawn.
fn placed_labels(
    ctx: &mut ScreenCtx<'_>,
    frame: &Frame<'_>,
    labels: Vec<api::Label>,
    font: &Font,
    metrics: &dyn FontMetrics,
    pick: &dyn Fn() -> PickRef,
) -> Vec<Label> {
    labels
        .into_iter()
        .filter_map(|l| {
            let anchor = frame.to_geo((l.x, l.y));
            let screen = ctx.project(anchor)?;
            let turned = frame
                .screen_angle(ctx, (l.x, l.y), l.angle_deg)
                .unwrap_or(l.angle_deg);
            // Kept upright: a label turned past vertical reads the other
            // way, so it extends from its anchor to the other side.
            let (rotation_deg, flipped) = upright(turned);
            let align = match (l.justify, flipped) {
                (Justify::Left, false) | (Justify::Right, true) => TextAlign::Left,
                (Justify::Center, _) => TextAlign::Center,
                (Justify::Right, false) | (Justify::Left, true) => TextAlign::Right,
            };
            Some(label::at_screen(
                label::PlacedText {
                    text: l.text,
                    screen,
                    anchor,
                    rotation_deg,
                    align,
                    knockout: l.knockout,
                },
                font,
                metrics,
                pick(),
            ))
        })
        .collect()
}

/// `deg` brought within (-90, 90], and whether that turned it half round.
fn upright(deg: f64) -> (f64, bool) {
    let d = (deg + 180.0).rem_euclid(360.0) - 180.0;
    if d > 90.0 {
        (d - 180.0, true)
    } else if d <= -90.0 {
        (d + 180.0, true)
    } else {
        (d, false)
    }
}

/// An engine shape's lines as screen items, each vertex carried to the
/// ground and the edges between them projected like geographic lines.
fn screen_items(
    ctx: &mut ScreenCtx<'_>,
    frame: &Frame<'_>,
    shape: &Shape,
    pick: PickRef,
) -> Result<Vec<ScreenItem>, BudgetError> {
    let mut items = Vec::new();
    let outline = shape_outline(shape, Rgba::BLACK);
    let fill = shape_fill(shape);
    for line in shape.polylines() {
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
        for (shape, fill) in ctx.part(&geometry, fill)? {
            items.push(ScreenItem {
                pick: pick.clone(),
                role: PartRole::Decoration,
                shape,
                stroke: outline,
                fill,
                decoration: true,
            });
        }
    }
    Ok(items)
}
