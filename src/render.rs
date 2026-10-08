//! Render plans: a construction resolved for one view.

use crate::antimeridian::{self, LonLat};
use crate::budget::{Budget, BudgetError, VertexMeter};
use crate::construction::{Construction, GeoGeometry, HandleKind, PartRole};
use crate::edit::HandleId;
use crate::geo::GeoPoint;
use crate::geodesy::Earth;
use crate::pick::PickRef;
use crate::style::{Fill, Stroke};

mod decoration;
mod label;
mod local;
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
pub struct View {
    /// Changes with camera position or orientation, viewport, projection or DPI.
    pub view_revision: u64,
    /// Changes with terrain or elevation data.
    pub surface_revision: u64,
    /// Font for labels.
    pub label_font: Font,
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
}

/// An edit handle for this view.
#[derive(Clone, Debug, PartialEq)]
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
    /// True when some ground position had no terrain height and used 0.
    pub terrain_missing: bool,
}

/// Why a plan could not be built.
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    /// A size limit would be exceeded.
    #[error(transparent)]
    Budget(#[from] BudgetError),
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
    let pick = |part, handle| PickRef {
        definition: construction.definition.clone(),
        part,
        handle,
    };
    let mut geo = Vec::with_capacity(construction.parts.len());
    let mut items = Vec::new();
    for part in &construction.parts {
        let shape = match &part.geometry {
            GeoGeometry::Line(p) => GeoShape::Lines(antimeridian::split_line(p)),
            GeoGeometry::Ring(p) => GeoShape::Polygons(antimeridian::split_ring(p)),
        };
        geo.push(GeoItem {
            pick: pick(part.id, None),
            role: part.role,
            shape,
            stroke: part.stroke,
            fill: part.fill,
        });
        for (shape, fill) in ctx.part(&part.geometry, part.fill)? {
            items.push(ScreenItem {
                pick: pick(part.id, None),
                role: part.role,
                shape,
                stroke: part.stroke,
                fill,
            });
        }
    }
    for d in &construction.decorations {
        items.extend(decoration::resolve(&mut ctx, d, &pick)?);
    }
    let labels = construction
        .labels
        .iter()
        .map(|l| label::place(&mut ctx, l, &view.label_font, metrics, pick(l.part, None)))
        .collect();
    let handles = construction
        .handles
        .iter()
        .map(|h| Handle {
            pick: pick(crate::construction::PartId(u16::MAX), Some(h.id)),
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
        terrain_missing: ctx.terrain_missing(),
    })
}
