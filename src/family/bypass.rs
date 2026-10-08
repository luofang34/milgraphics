//! Obstacle bypass (easy): an open-ended box from the opening (points 1 and
//! 2, the arrow tips) back to the rear (point 3), with filled arrowheads at
//! the tips.
//!
//! The rear is the opening translated by the vector from the opening's
//! midpoint to point 3, so it has the opening's length; it is perpendicular
//! to the sides when point 3 lies on the opening's perpendicular bisector.

use crate::construction::{DecorationSize, GeoGeometry, PartRole, ScreenDecoration};
use crate::definition::GraphicDefinition;
use crate::family::{ConstructError, Ctx, vertex_handles};
use crate::plane::LocalPlane;
use crate::style::Fill;

/// Arrowhead wing length as a fraction of (opening + depth) on screen.
const HEAD_FRACTION: f64 = 1.0 / 20.0;
/// Smallest and largest arrowhead wing, in pixels.
const HEAD_PX: (f64, f64) = (2.5, 20.0);

pub(crate) fn construct(ctx: &mut Ctx<'_>, def: &GraphicDefinition) -> Result<(), ConstructError> {
    let degenerate = ConstructError::Degenerate {
        symbol: ctx.spec.name,
        reason: "a bypass needs two arrow tips and a rear point",
    };
    let mut points = def.positions();
    let (Some(p0), Some(p1), Some(p2)) = (points.next(), points.next(), points.next()) else {
        return Err(degenerate);
    };
    let mid = ctx.earth.interpolate(p0, p1, 0.5);
    let plane = LocalPlane::new(ctx.earth, mid);
    let depth = plane.to_xy(p2);
    let c0 = plane.to_geo(plane.to_xy(p0).add(depth));
    let c1 = plane.to_geo(plane.to_xy(p1).add(depth));
    let outline = ctx.geodesic_path(&[p1, c1, c0, p0])?;
    let part = ctx.add_part(
        PartRole::Line,
        GeoGeometry::Line(outline),
        Some(ctx.palette.line),
        Fill::None,
    );
    let size = DecorationSize::Proportional {
        segments: [(p0, p1), (mid, p2)],
        fraction: HEAD_FRACTION,
        min_px: HEAD_PX.0,
        max_px: HEAD_PX.1,
    };
    for (tip, toward) in [(p0, c0), (p1, c1)] {
        ctx.add_decoration(ScreenDecoration::Arrowhead {
            id: part,
            tip,
            toward,
            size,
            half_angle_deg: 45.0,
            filled: true,
            stroke: ctx.palette.solid_line,
        });
    }
    ctx.add_handles(vertex_handles(def));
    Ok(())
}
