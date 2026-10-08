//! Labelled areas (e.g. Named Area of Interest): a closed geodesic ring with
//! "`prefix` T" at its centre.

use crate::construction::{GeoGeometry, LabelPlacement, LabelSpec, PartRole};
use crate::definition::GraphicDefinition;
use crate::family::{ConstructError, Ctx, vertex_handles};
use crate::geo::{GeoPoint, wrap_longitude};
use crate::style::Fill;

pub(crate) fn construct(
    ctx: &mut Ctx<'_>,
    def: &GraphicDefinition,
    prefix: &str,
) -> Result<(), ConstructError> {
    let control: Vec<GeoPoint> = def.positions().collect();
    let ring = ctx.geodesic_ring(&control)?;
    let center = bounds_center(&control).ok_or(ConstructError::Degenerate {
        symbol: ctx.spec.name(),
        reason: "an area needs at least three points",
    })?;
    let text = match def.modifiers.designation() {
        Some(t) => format!("{prefix} {t}"),
        None => prefix.to_owned(),
    };
    let fill = ctx.palette.fill.map_or(Fill::None, Fill::Solid);
    let stroke = Some(ctx.palette.line);
    let part = ctx.add_part(PartRole::Boundary, GeoGeometry::Ring(ring), stroke, fill);
    ctx.add_label(LabelSpec {
        part,
        text,
        anchor: center,
        placement: LabelPlacement::Centered,
        line_offset: 0.0,
        may_hide: false,
    });
    ctx.add_handles(vertex_handles(def));
    Ok(())
}

/// Centre of the longitude/latitude bounds, with longitudes taken the short
/// way round so areas across the antimeridian centre correctly. The
/// standard asks for the centre of the area; this is the reference
/// implementation's choice of centre, kept so labels agree with it.
pub(crate) fn bounds_center(points: &[GeoPoint]) -> Option<GeoPoint> {
    let first = points.first()?;
    let (mut west, mut east) = (first.lon(), first.lon());
    let (mut south, mut north) = (first.lat(), first.lat());
    let mut prev = first.lon();
    for p in points.iter().skip(1) {
        let lon = prev + wrap_longitude(p.lon() - prev);
        west = west.min(lon);
        east = east.max(lon);
        south = south.min(p.lat());
        north = north.max(p.lat());
        prev = lon;
    }
    GeoPoint::new((west + east) / 2.0, (south + north) / 2.0).ok()
}
