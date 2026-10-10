//! Render plans: a construction resolved for one view.

use crate::antimeridian;
use crate::budget::{Budget, BudgetError, VertexMeter};
use crate::construction::{Construction, GeoGeometry, HandleKind, PartRole};
use crate::edit::HandleId;
use crate::geo::GeoPoint;
use crate::geodesy::Earth;
use crate::pick::{PickRef, PickTarget};
use crate::style::{Fill, Stroke};

mod decoration;
mod extent;
mod hatch;
mod knockout;
mod label;
mod local;
mod pattern;
mod ported;
mod projection;
mod screen;
mod symbols;

pub use crate::geo::LonLat;
pub use label::{Label, TextAlign};
pub use local::LocalEquirectangular;
pub use projection::{FixedAdvanceMetrics, Font, FontMetrics, Projection, ScreenPoint, ScreenRect};
pub use symbols::SymbolPlacement;

#[cfg(test)]
mod tests;

/// The view a plan is rendered for. The host bumps the revisions whenever
/// anything they summarise changes; they are part of every plan's cache key.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct View {
    /// Changes with camera position or orientation, viewport, projection or DPI.
    pub view_revision: u64,
    /// Changes with terrain or elevation data.
    pub surface_revision: u64,
    /// Font for labels.
    pub label_font: Font,
    /// What the plan resolves.
    pub content: PlanContent,
    /// Limits on the plan's size.
    pub budget: Budget,
}

/// What a plan resolves.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PlanContent {
    /// Both tiers: the graphic's parts in degrees and projected, with its
    /// pixel-sized decorations, labels and handles.
    #[default]
    Full,
    /// Only what a map engine drawing the geographic tier cannot draw:
    /// pixel-sized decorations, labels and handles. `geo` is empty and
    /// `screen` holds decorations only. For hosts that draw [`geographic`]
    /// every frame and need projected parts only now and then, such as to
    /// pick, when they resolve a full plan.
    Overlay,
}

impl View {
    /// A view with these revisions, the default label font and the default
    /// [`Budget`].
    pub fn new(view_revision: u64, surface_revision: u64) -> Self {
        Self {
            view_revision,
            surface_revision,
            label_font: Font::default(),
            content: PlanContent::Full,
            budget: Budget::default(),
        }
    }

    /// The same view with `budget` limiting its plans.
    pub fn with_budget(self, budget: Budget) -> Self {
        Self { budget, ..self }
    }
}

/// Geographic-tier geometry, split at the antimeridian.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum GeoShape {
    /// Open polylines.
    Lines(Vec<Vec<LonLat>>),
    /// Closed rings (first point repeated), one polygon each.
    Polygons(Vec<Vec<LonLat>>),
}

impl GeoShape {
    /// The polylines or rings.
    pub fn pieces(&self) -> &[Vec<LonLat>] {
        match self {
            Self::Lines(p) | Self::Polygons(p) => p,
        }
    }

    /// Whether the pieces are closed rings.
    pub fn is_closed(&self) -> bool {
        matches!(self, Self::Polygons(_))
    }
}

/// An item the map engine projects and drapes itself.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct GeoItem {
    /// What it is.
    pub pick: PickRef,
    /// Role of the part.
    pub role: PartRole,
    /// Geometry in degrees.
    pub shape: GeoShape,
    /// Outline.
    pub stroke: Option<Stroke>,
    /// Interior.
    pub fill: Fill,
}

/// Screen-tier geometry in pixels.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum ScreenShape {
    /// An open polyline.
    Polyline(Vec<ScreenPoint>),
    /// A closed polygon, first point not repeated.
    Polygon(Vec<ScreenPoint>),
}

impl ScreenShape {
    /// The shape's points, in drawing order.
    pub fn points(&self) -> &[ScreenPoint] {
        match self {
            Self::Polyline(p) | Self::Polygon(p) => p,
        }
    }

    /// Whether the last point joins back to the first.
    pub fn is_closed(&self) -> bool {
        matches!(self, Self::Polygon(_))
    }
}

/// An item already projected, densified and clipped for this view.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct ScreenItem {
    /// What it is.
    pub pick: PickRef,
    /// Role of the part.
    pub role: PartRole,
    /// Geometry in pixels.
    pub shape: ScreenShape,
    /// Outline.
    pub stroke: Option<Stroke>,
    /// Interior; dropped when a ring is cut by the horizon.
    pub fill: Fill,
    /// A pixel-sized decoration, which the geographic tier does not contain;
    /// adapters that draw the geographic tier in the map engine overlay only
    /// these.
    pub decoration: bool,
}

/// An edit handle for this view.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Handle {
    /// What it is.
    pub pick: PickRef,
    /// Identity for edits.
    pub id: HandleId,
    /// What it changes.
    pub kind: HandleKind,
    /// Geographic position.
    pub at: GeoPoint,
    /// Screen position, if visible.
    pub screen: Option<ScreenPoint>,
}

/// A construction resolved for one view.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct RenderPlan {
    /// Renderer version that built it.
    pub renderer_version: &'static str,
    /// Definition revision it shows.
    pub revision: u64,
    /// View revision it was built for.
    pub view_revision: u64,
    /// Surface revision it was built for.
    pub surface_revision: u64,
    /// [`FontMetrics::identity`] of the metrics labels were measured with.
    pub metrics_identity: String,
    /// Items for the map engine.
    pub geo: Vec<GeoItem>,
    /// Items for a screen overlay, in drawing order.
    pub screen: Vec<ScreenItem>,
    /// Labels.
    pub labels: Vec<Label>,
    /// Edit handles.
    pub handles: Vec<Handle>,
    /// Single-point symbols drawn as part of the graphic.
    pub symbols: Vec<SymbolPlacement>,
}

/// Why a plan could not be built.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum RenderError {
    /// A size limit would be exceeded.
    #[error(transparent)]
    Budget(#[from] BudgetError),
}

/// The geographic tier alone: what a map engine draws and drapes itself.
/// It does not depend on the view, so engines update it only when a
/// graphic changes.
///
/// Only `view.budget` is used, so the result can be kept across view
/// changes that keep the budget.
pub fn geographic(construction: &Construction, view: &View) -> Result<Vec<GeoItem>, RenderError> {
    let mut meter = VertexMeter::new(&view.budget);
    construction
        .parts
        .iter()
        .map(|part| {
            let item = geo_item(construction, part);
            meter.take(item.vertex_count())?;
            Ok(item)
        })
        .collect()
}

fn geo_item(construction: &Construction, part: &crate::construction::GeoPart) -> GeoItem {
    let shape = match &part.geometry {
        GeoGeometry::Line(p) => GeoShape::Lines(antimeridian::split_line(p)),
        GeoGeometry::Ring(p) => GeoShape::Polygons(antimeridian::split_ring(p)),
    };
    GeoItem {
        pick: PickRef {
            definition: construction.definition.clone(),
            target: PickTarget::Part(part.id),
        },
        role: part.role,
        shape,
        stroke: part.stroke,
        fill: part.fill,
    }
}

impl GeoItem {
    /// Vertices in the item's geometry.
    pub fn vertex_count(&self) -> usize {
        let (GeoShape::Lines(pieces) | GeoShape::Polygons(pieces)) = &self.shape;
        pieces.iter().map(Vec::len).sum()
    }
}

/// A construction's parts for one plan.
struct Parts {
    /// The geographic tier (full plans only).
    geo: Vec<GeoItem>,
    /// The parts projected (full plans only).
    items: Vec<ScreenItem>,
    /// Hatched and pattern-filled parts projected for an overlay plan, which
    /// draws their hatch lines and figures over the map's outline but not the
    /// parts themselves.
    hatched: Vec<ScreenItem>,
}

fn parts(
    ctx: &mut screen::ScreenCtx<'_>,
    construction: &Construction,
    full: bool,
    in_view: bool,
    pick: &impl Fn(PickTarget) -> PickRef,
) -> Result<Parts, RenderError> {
    let mut out = Parts {
        geo: Vec::new(),
        items: Vec::new(),
        hatched: Vec::new(),
    };
    for part in &construction.parts {
        if full {
            let item = geo_item(construction, part);
            ctx.take(item.vertex_count())?;
            out.geo.push(item);
        }
        let screen_fill = matches!(part.fill, Fill::Hatch(_) | Fill::Pattern(_));
        if !in_view || !(full || screen_fill) {
            continue;
        }
        for (shape, fill) in ctx.part(&part.geometry, part.fill)? {
            let item = ScreenItem {
                pick: pick(PickTarget::Part(part.id)),
                role: part.role,
                shape,
                stroke: part.stroke,
                fill,
                decoration: false,
            };
            if full {
                out.items.push(item);
            } else {
                out.hatched.push(item);
            }
        }
    }
    Ok(out)
}

/// The screen tier once labels are placed: outlines cut where knockout
/// labels sit, and hatch lines and pattern figures added clear of every
/// label.
fn fills_and_gaps(
    ctx: &mut screen::ScreenCtx<'_>,
    mut items: Vec<ScreenItem>,
    hatched: &[ScreenItem],
    labels: &[Label],
) -> Result<Vec<ScreenItem>, RenderError> {
    let gaps: Vec<[ScreenPoint; 4]> = labels
        .iter()
        .filter(|l| l.knockout)
        .filter_map(|l| l.corners)
        .collect();
    if !gaps.is_empty() {
        items = knockout::cut(ctx, items, &gaps)?;
    }
    let boxes: Vec<[ScreenPoint; 4]> = labels.iter().filter_map(|l| l.corners).collect();
    let lines = hatch::items(items.iter().chain(hatched), &boxes);
    ctx.take(lines.len() * 2)?;
    let figures = pattern::items(items.iter().chain(hatched), &boxes);
    ctx.take(figures.iter().map(|f| f.shape.points().len()).sum())?;
    items.extend(lines);
    items.extend(figures);
    Ok(items)
}

/// Resolves `construction` for `view`.
pub fn render(
    construction: &Construction,
    view: &View,
    projection: &dyn Projection,
    metrics: &dyn FontMetrics,
) -> Result<RenderPlan, RenderError> {
    let earth = Earth::wgs84();
    let mut meter = VertexMeter::new(&view.budget);
    let mut ctx = screen::ScreenCtx::new(&earth, projection, &mut meter);
    let pick = |target| PickRef {
        definition: construction.definition.clone(),
        target,
    };
    let extent = extent::of(&mut ctx, construction);
    let full = view.content == PlanContent::Full;
    let Parts {
        geo,
        mut items,
        hatched,
    } = parts(&mut ctx, construction, full, extent.in_view, &pick)?;
    // Decorations and labels have minimum pixel sizes; on a graphic a few
    // pixels across they would dwarf it, so it draws its geographic tier only.
    let detailed = extent.detailed && extent.in_view;
    let mut engine_labels = Vec::new();
    for d in construction.decorations.iter().filter(|_| detailed) {
        items.extend(decoration::resolve(&mut ctx, d, &pick)?);
        let (more, labels) = ported::resolve(&mut ctx, &d.0, &view.label_font, metrics, &pick)?;
        items.extend(more);
        engine_labels.extend(labels);
    }
    let mut labels: Vec<Label> = construction
        .labels
        .iter()
        .filter(|_| detailed)
        .map(|l| {
            label::place(
                &mut ctx,
                l,
                &view.label_font,
                metrics,
                pick(PickTarget::Part(l.part)),
            )
        })
        .collect();
    labels.extend(engine_labels);
    if detailed {
        items = fills_and_gaps(&mut ctx, items, &hatched, &labels)?;
    }
    let symbols = if detailed {
        symbols::resolve(&mut ctx, construction, &pick)
    } else {
        Vec::new()
    };
    let handles = construction
        .handles
        .iter()
        .map(|h| Handle {
            pick: pick(PickTarget::Handle(h.id)),
            id: h.id,
            kind: h.kind,
            at: h.at,
            screen: ctx.project(h.at),
        })
        .collect();
    Ok(RenderPlan {
        renderer_version: construction.renderer_version,
        revision: construction.revision,
        view_revision: view.view_revision,
        surface_revision: view.surface_revision,
        metrics_identity: metrics.identity().to_owned(),
        geo,
        screen: items,
        labels,
        handles,
        symbols,
    })
}
