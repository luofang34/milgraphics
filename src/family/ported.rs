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
use crate::engine::base::{Pt, Shape, shape_type};
use crate::engine::line_type::classes::MsInfo;
use crate::family::{ConstructError, Ctx, vertex_handles};
use crate::geo::GeoPoint;
use crate::plane::{LocalPlane, Xy};
use crate::render::{FixedAdvanceMetrics, Font, FontMetrics};
use crate::style::{DashPattern, Fill, Rgba, Stroke};

mod classify;

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
    let symbol = &def.symbol;
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
    let anchors: Vec<GeoPoint> = def.positions().collect();
    let origin = anchors.first().copied().ok_or(ConstructError::Degenerate {
        symbol: ctx.spec.name(),
        reason: "no control points",
    })?;
    let plane = LocalPlane::new(ctx.earth, origin);
    let xy: Vec<Xy> = anchors.iter().map(|&p| plane.to_xy(p)).collect();
    let extent = extent_m(&xy, &def.modifiers.distances_m);
    let mut runs = Vec::with_capacity(REFERENCE_EXTENTS_PX.len());
    for px in REFERENCE_EXTENTS_PX {
        let mpp = extent / px;
        runs.push((
            mpp,
            run(def, line_type, &xy, mpp).map_err(|e| ConstructError::Unrenderable {
                symbol: ctx.spec.name(),
                reason: e.to_string(),
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
    ctx.add_decoration(Decoration::Engine {
        line_type,
        anchors,
        symbol: def.symbol.clone(),
        modifiers: Box::new(def.modifiers.clone()),
        geographic,
        shape_count: a.shapes.len(),
        part,
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
    line_type: i32,
    xy: &[Xy],
    mpp: f64,
) -> Result<Output, crate::engine::base::EngineError> {
    let metrics = FixedAdvanceMetrics::default();
    let font = Font::default();
    let width = |t: &str| metrics.text_width_px(&font, t);
    api::draw(&Input {
        line_type,
        symbol: &def.symbol,
        pixels: xy.iter().map(|&p| to_px(p, mpp)).collect(),
        modifiers: &def.modifiers,
        meters_per_pixel: mpp,
        text_width: &width,
        ms_info: crate::family::ported::ms_info(&def.symbol),
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
        ctx.take_vertices(line.len())?;
        let (geometry, fill, stroke, role) = if shape.shape_type == shape_type::FILL {
            if line.len() > 1 && line.first() == line.last() {
                line.pop();
            }
            let fill = shape.fill_color.map_or(Fill::None, Fill::Solid);
            (GeoGeometry::Ring(line), fill, None, PartRole::Decoration)
        } else {
            (
                GeoGeometry::Line(line),
                Fill::None,
                stroke(ctx, shape),
                PartRole::Line,
            )
        };
        let id = ctx.add_part(role, geometry, stroke, fill);
        first.get_or_insert(id);
    }
    Ok(first)
}

/// A renderer shape's stroke, with the operator's line colour if set.
pub(crate) fn stroke(ctx: &Ctx<'_>, shape: &Shape) -> Option<Stroke> {
    Some(shape_stroke(shape, ctx.palette.line.color))
}

pub(crate) fn shape_stroke(shape: &Shape, fallback: Rgba) -> Stroke {
    Stroke {
        color: shape.line_color.unwrap_or(fallback),
        width_px: shape.stroke.width,
        dash: match shape.stroke.dash {
            Some(_) => DashPattern::Dashed,
            None => DashPattern::Solid,
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
