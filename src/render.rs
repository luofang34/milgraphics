//! Render plans: a construction resolved for one view.

use crate::antimeridian::{self, LonLat};
use crate::budget::{Budget, BudgetError, VertexMeter};
use crate::construction::{Construction, GeoGeometry, HandleKind, PartRole};
use crate::edit::HandleId;
use crate::geo::GeoPoint;
use crate::geodesy::Earth;
use crate::pick::{PickRef, PickTarget};
use crate::style::{Fill, Stroke};

mod decoration;
mod label;
mod local;
mod ported;
mod projection;
mod screen;

pub use label::{Label, TextAlign};
pub use local::LocalEquirectangular;
pub use projection::{FixedAdvanceMetrics, Font, FontMetrics, Projection, ScreenPoint};

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
}

impl View {
    /// A view with these revisions and the default label font.
    pub fn new(view_revision: u64, surface_revision: u64) -> Self {
        Self {
            view_revision,
            surface_revision,
            label_font: Font::default(),
        }
    }
}

/// Geographic-tier geometry, split at the antimeridian.
#[derive(Clone, Debug, PartialEq)]
pub enum GeoShape {
    /// Open polylines.
    Lines(Vec<Vec<LonLat>>),
    /// Closed rings (first point repeated), one polygon each.
    Polygons(Vec<Vec<LonLat>>),
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
pub enum ScreenShape {
    /// An open polyline.
    Polyline(Vec<ScreenPoint>),
    /// A closed polygon, first point not repeated.
    Polygon(Vec<ScreenPoint>),
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
    /// Font-metrics identity used for labels.
    pub metrics: String,
    /// Items for the map engine.
    pub geo: Vec<GeoItem>,
    /// Items for a screen overlay, in drawing order.
    pub screen: Vec<ScreenItem>,
    /// Labels.
    pub labels: Vec<Label>,
    /// Edit handles.
    pub handles: Vec<Handle>,
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
pub fn geographic(
    construction: &Construction,
    budget: &Budget,
) -> Result<Vec<GeoItem>, RenderError> {
    let mut meter = VertexMeter::new(budget);
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

/// On-screen extent below which a graphic is drawn without decorations or
/// labels.
const MIN_DETAIL_PX: f64 = 12.0;

/// The on-screen extent of a graphic: of its handles (control points and,
/// for graphics sized by ranges or widths, the handles setting them) and of
/// the corners of its geographic parts' bounds.
fn extent_px(ctx: &mut screen::ScreenCtx<'_>, construction: &Construction) -> f64 {
    let (mut west, mut south, mut east, mut north) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for q in construction.parts.iter().flat_map(|p| p.geometry.points()) {
        (west, south) = (west.min(q.lon()), south.min(q.lat()));
        (east, north) = (east.max(q.lon()), north.max(q.lat()));
    }
    let corners = [(west, south), (east, north), (west, north), (east, south)]
        .into_iter()
        .filter_map(|(lon, lat)| GeoPoint::new(lon, lat).ok());
    let points: Vec<GeoPoint> = construction
        .handles
        .iter()
        .map(|h| h.at)
        .chain(corners)
        .collect();
    let (mut lo, mut hi) = ((f64::MAX, f64::MAX), (f64::MIN, f64::MIN));
    for at in points {
        // A graphic reaching past the horizon is large in any view.
        let Some(p) = ctx.project(at) else {
            return f64::INFINITY;
        };
        lo = (lo.0.min(p.x), lo.1.min(p.y));
        hi = (hi.0.max(p.x), hi.1.max(p.y));
    }
    if lo.0 > hi.0 {
        return 0.0;
    }
    (hi.0 - lo.0).hypot(hi.1 - lo.1)
}

/// Resolves `construction` for `view`.
pub fn render(
    construction: &Construction,
    view: &View,
    projection: &dyn Projection,
    metrics: &dyn FontMetrics,
    budget: &Budget,
) -> Result<RenderPlan, RenderError> {
    let earth = Earth::wgs84();
    let mut meter = VertexMeter::new(budget);
    let mut ctx = screen::ScreenCtx::new(&earth, projection, &mut meter);
    let pick = |target| PickRef {
        definition: construction.definition.clone(),
        target,
    };
    let mut geo = Vec::with_capacity(construction.parts.len());
    let mut items = Vec::new();
    for part in &construction.parts {
        let item = geo_item(construction, part);
        ctx.take(item.vertex_count())?;
        geo.push(item);
        for (shape, fill) in ctx.part(&part.geometry, part.fill)? {
            items.push(ScreenItem {
                pick: pick(PickTarget::Part(part.id)),
                role: part.role,
                shape,
                stroke: part.stroke,
                fill,
                decoration: false,
            });
        }
    }
    // Decorations and labels have minimum pixel sizes; on a graphic a few
    // pixels across they would dwarf it, so it draws its geographic tier only.
    let detailed = extent_px(&mut ctx, construction) >= MIN_DETAIL_PX;
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
        metrics: metrics.identity().to_owned(),
        geo,
        screen: items,
        labels,
        handles,
    })
}
