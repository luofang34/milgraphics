//! Graphics drawn by the ported upstream renderer (`crate::engine`).
//!
//! The renderer works in pixels. Here it runs twice in a local plane, at two
//! pixel sizes: a shape that comes out the same on the ground both times is
//! proportional to the control points and becomes a geographic part; the
//! others carry pixel-sized elements (teeth, ticks, fixed arrowheads) and are
//! drawn for each view from the projected control points.

use crate::catalog::CatalogDrawRule;
use crate::construction::{Decoration, GeoGeometry, PartRole};
use crate::definition::GraphicDefinition;
use crate::engine::api::{self, Input, Output};
use crate::engine::base::{Hatch as EngineHatch, Pt, Shape, shape_type};
use crate::engine::line_type::classes::MsInfo;
use crate::engine::render_utility::hatch::{HATCH_BACKWARD_DIAGONAL, HATCH_FORWARD_DIAGONAL};
use crate::family::{ConstructError, Ctx, vertex_handles};
use crate::geo::GeoPoint;
use crate::plane::{LocalPlane, Xy};
use crate::render::{FixedAdvanceMetrics, Font, FontMetrics};
use crate::style::{DashPattern, Fill, Hatch, Rgba, Stroke};

mod classify;
mod labels;
mod pattern;
mod rhumb;
mod storm;

/// Pixel extents the graphic is drawn at for classification. They lie on
/// either side of the range where upstream sizes decorations in proportion
/// to the graphic (its clamps act between tens and hundreds of pixels), so
/// a shape that a clamp holds to a pixel size at some zoom differs between
/// the two and is drawn per view.
const REFERENCE_EXTENTS_PX: [f64; 2] = [50.0, 1500.0];
/// Offset of the local plane's origin in the renderer's pixel frame, which
/// keeps coordinates positive as upstream's are.
pub(super) const ORIGIN_PX: f64 = 2000.0;

pub(crate) fn construct(ctx: &mut Ctx<'_>, def: &GraphicDefinition) -> Result<(), ConstructError> {
    if storm::applies(def) {
        return storm::construct(ctx, def);
    }
    match rhumb::prepare(ctx, def)? {
        Some((anchors, modifiers)) => draw(ctx, def, anchors, &modifiers),
        None => draw(ctx, def, def.positions().collect(), &def.modifiers),
    }
}

/// Draws `def` from the renderer's points `anchors` and amplifiers
/// `modifiers`, which are the definition's own unless the graphic is
/// prepared for the renderer.
fn draw(
    ctx: &mut Ctx<'_>,
    def: &GraphicDefinition,
    anchors: Vec<GeoPoint>,
    modifiers: &crate::modifier::Modifiers,
) -> Result<(), ConstructError> {
    let symbol = &crate::engine::intercept::engine_symbol(&def.symbol);
    let line_type = crate::engine::line_type::line_type(
        symbol.version_code(),
        symbol.symbol_set(),
        symbol.entity().get(),
    )
    // A METOC symbol without a line type of its own is drawn by upstream's
    // default case (its control points as a line), as line type -1.
    .or_else(|| matches!(symbol.symbol_set(), 45 | 46).then_some(-1))
    .ok_or(ConstructError::Degenerate {
        symbol: ctx.spec.name(),
        reason: "no upstream line type",
    })?;
    let origin = anchors.first().copied().ok_or(ConstructError::Degenerate {
        symbol: ctx.spec.name(),
        reason: "no control points",
    })?;
    let plane = LocalPlane::new(ctx.earth, origin);
    let xy: Vec<Xy> = anchors.iter().map(|&p| plane.to_xy(p)).collect();
    let extent = extent_m(&xy, &modifiers.distances_m);
    let mut runs = Vec::with_capacity(REFERENCE_EXTENTS_PX.len());
    for px in REFERENCE_EXTENTS_PX {
        let mpp = extent / px;
        runs.push((
            mpp,
            run(def, modifiers, symbol, line_type, &xy, mpp).map_err(|e| {
                ConstructError::Unrenderable {
                    symbol: ctx.spec.name(),
                    reason: e.to_string(),
                }
            })?,
        ));
    }
    let [(mpp_a, a), (mpp_b, b)] =
        <[(f64, Output); 2]>::try_from(runs).map_err(|_| ConstructError::Degenerate {
            symbol: ctx.spec.name(),
            reason: "classification runs",
        })?;
    let geographic = classify::geographic(&a.shapes, mpp_a, &b.shapes, mpp_b, extent);
    let mut first = None;
    for (shape, &geo) in a.shapes.iter().zip(&geographic) {
        if geo {
            if let Some(id) = add_part(ctx, &plane, shape, mpp_a)? {
                first.get_or_insert(id);
            }
        }
    }
    let part = first.unwrap_or_else(|| ctx.next_part());
    // The larger run follows a curve more closely.
    let drawn: Vec<GeoPoint> = b
        .shapes
        .iter()
        .flat_map(Shape::polylines)
        .flatten()
        .map(|p| plane.to_geo(from_px(p, mpp_b)))
        .collect();
    labels::add(ctx, def, part, &drawn);
    ctx.add_decoration(Decoration::Engine {
        line_type,
        anchors,
        symbol: symbol.clone(),
        modifiers: Box::new(modifiers.clone()),
        style: crate::engine::api::Style::of(&def.style),
        geographic,
        shape_count: a.shapes.len(),
        part,
        reach_m: extent,
    });
    ctx.add_handles(vertex_handles(def));
    Ok(())
}

/// Ground size the graphic spans, in metres: its control points' extent or,
/// for graphics sized by ranges, twice the largest one.
fn extent_m(xy: &[Xy], distances: &[f64]) -> f64 {
    let (mut lo, mut hi) = (Xy::new(f64::MAX, f64::MAX), Xy::new(f64::MIN, f64::MIN));
    for p in xy {
        lo = Xy::new(lo.x.min(p.x), lo.y.min(p.y));
        hi = Xy::new(hi.x.max(p.x), hi.y.max(p.y));
    }
    let span = hi.sub(lo).len();
    let ranges = 2.0 * distances.iter().copied().fold(0.0, f64::max);
    let extent = span.max(ranges);
    if extent.is_finite() && extent > 0.0 {
        extent
    } else {
        1000.0
    }
}

fn to_px(p: Xy, mpp: f64) -> Pt {
    Pt::new(ORIGIN_PX + p.x / mpp, ORIGIN_PX - p.y / mpp)
}

pub(crate) fn from_px((x, y): (f64, f64), mpp: f64) -> Xy {
    Xy::new((x - ORIGIN_PX) * mpp, (ORIGIN_PX - y) * mpp)
}

fn run(
    def: &GraphicDefinition,
    modifiers: &crate::modifier::Modifiers,
    symbol: &crate::sidc::SymbolId,
    line_type: i32,
    xy: &[Xy],
    mpp: f64,
) -> Result<Output, crate::engine::base::EngineError> {
    let metrics = FixedAdvanceMetrics::default();
    let font = Font::default();
    let width = |t: &str| metrics.text_width_px(&font, t);
    api::draw(&Input {
        line_type,
        symbol,
        pixels: xy.iter().map(|&p| to_px(p, mpp)).collect(),
        modifiers,
        meters_per_pixel: mpp,
        text_width: &width,
        ms_info: crate::family::ported::ms_info(symbol),
        style: crate::engine::api::Style::of(&def.style),
    })
}

/// One renderer shape as geographic parts; the first part's id, or `None`
/// for a shape with no points (upstream keeps such shapes).
fn add_part(
    ctx: &mut Ctx<'_>,
    plane: &LocalPlane<'_>,
    shape: &Shape,
    mpp: f64,
) -> Result<Option<crate::construction::PartId>, ConstructError> {
    let lines: Vec<Vec<GeoPoint>> = shape
        .polylines()
        .into_iter()
        .map(|l| {
            l.into_iter()
                .map(|p| plane.to_geo(from_px(p, mpp)))
                .collect()
        })
        .collect();
    let mut first = None;
    for mut line in lines {
        let outline = shape_outline(shape, ctx.palette.line.color);
        let fill = shape_fill(shape);
        let (geometry, role) = if fill == Fill::None {
            (GeoGeometry::Line(line), PartRole::Line)
        } else {
            if line.len() > 1 && line.first() == line.last() {
                line.pop();
            }
            (GeoGeometry::Ring(line), PartRole::Decoration)
        };
        let stroke = outline;
        let id = ctx.add_part(role, geometry, stroke, fill);
        first.get_or_insert(id);
    }
    Ok(first)
}

/// How an engine shape's interior is painted: the hatch or figures upstream
/// paints from an image where it has one, else its fill colour.
pub(crate) fn shape_fill(shape: &Shape) -> Fill {
    shape
        .pattern_fill
        .and_then(hatch)
        .map(Fill::Hatch)
        .or_else(|| {
            shape
                .metoc_pattern
                .and_then(pattern::metoc)
                .map(Fill::Pattern)
        })
        .or_else(|| shape.fill_color.map(Fill::Solid))
        .unwrap_or(Fill::None)
}

/// The outline of an engine shape: none for a fill shape, nor for a
/// pattern-filled area whose type clears the line colour, which upstream
/// paints without a boundary.
pub(crate) fn shape_outline(shape: &Shape, fallback: Rgba) -> Option<Stroke> {
    let unlined = shape.metoc_pattern.is_some() && shape.line_color.is_none();
    (shape.shape_type != shape_type::FILL && !unlined).then(|| shape_stroke(shape, fallback))
}

/// Upstream tiles a square `spacing` pixels wide with one diagonal line
/// (`PatternFillRenderer.MakeHatchPatternFill`), so its lines are `spacing`
/// apart along the x axis. Its "forward" diagonal falls to the right on a
/// y-down screen and its "backward" one rises.
fn hatch(h: EngineHatch) -> Option<Hatch> {
    let angle_deg = match h.style {
        HATCH_FORWARD_DIAGONAL => 135.0,
        HATCH_BACKWARD_DIAGONAL => 45.0,
        _ => return None,
    };
    Some(Hatch::new(
        h.color.unwrap_or(Rgba::BLACK),
        angle_deg,
        f64::from(h.spacing) * core::f64::consts::FRAC_1_SQRT_2,
        f64::from(h.thickness),
    ))
}

/// Upstream's dotted line style (`clsUtility.getLineStroke`).
const DOTTED_STYLE: i32 = 2;

pub(crate) fn shape_stroke(shape: &Shape, fallback: Rgba) -> Stroke {
    Stroke {
        color: shape.line_color.unwrap_or(fallback),
        width_px: shape.stroke.width,
        dash: match (&shape.stroke.dash, shape.style) {
            (None, _) => DashPattern::Solid,
            (Some(_), DOTTED_STYLE) => DashPattern::Dotted,
            (Some(_), _) => DashPattern::Dashed,
        },
    }
}

/// The symbol's draw rule and point range as upstream's `MSInfo` gives them
/// to the renderer: draw rules are numbered by family (area 100s, point
/// 200s, line 300s, corridor 400s, axis 500s, polyline 600s, ellipse 700s,
/// rectangular 800s, circular 900s, arc 1000s).
pub(crate) fn ms_info(symbol: &crate::sidc::SymbolId) -> Option<MsInfo> {
    let spec = crate::support::spec(symbol).ok()?;
    let draw_rule = match spec.catalog_entry()?.draw_rule {
        CatalogDrawRule::Standard(rule) => rule_number(rule.name()),
        CatalogDrawRule::Metoc(_) => -1,
    };
    let count = |n: usize| i32::try_from(n).unwrap_or(i32::MAX);
    Some(MsInfo {
        draw_rule,
        min_points: count(spec.min_points),
        max_points: count(spec.max_points),
    })
}

fn rule_number(name: &str) -> i32 {
    const FAMILIES: [(&str, i32); 10] = [
        ("Rectangular", 800),
        ("Circular", 900),
        ("Polyline", 600),
        ("Corridor", 400),
        ("Ellipse", 700),
        ("Point", 200),
        ("Area", 100),
        ("Line", 300),
        ("Axis", 500),
        ("Arc", 1000),
    ];
    FAMILIES
        .iter()
        .find_map(|(prefix, base)| {
            let n: i32 = name.strip_prefix(prefix)?.parse().ok()?;
            Some(base + n)
        })
        .unwrap_or(0)
}

#[cfg(test)]
mod tests;
