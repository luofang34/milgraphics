//! How large a graphic appears and whether it can reach the viewport.

use crate::construction::Construction;
use crate::geo::GeoPoint;
use crate::render::screen::ScreenCtx;

/// On-screen extent below which a graphic is drawn without decorations or
/// labels.
const MIN_DETAIL_PX: f64 = 12.0;

/// How far beyond a graphic's own extent its pixel-sized items may reach:
/// labels and decorations are anchored on the graphic.
const OVERLAY_MARGIN_PX: f64 = 512.0;

/// What a view shows of a graphic.
pub(crate) struct Extent {
    /// Large enough for decorations and labels.
    pub(crate) detailed: bool,
    /// May have something inside the viewport.
    pub(crate) in_view: bool,
}

/// The on-screen extent of a graphic, from its handles (control points and,
/// for graphics sized by ranges or widths, the handles setting them) and the
/// corners of its geographic parts' bounds.
///
/// A graphic is out of view when that box, grown by half its diagonal (a
/// curve bending less than a semicircle stays that close to its ends) and
/// by [`OVERLAY_MARGIN_PX`], misses the viewport.
pub(crate) fn of(ctx: &mut ScreenCtx<'_>, construction: &Construction) -> Extent {
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
            return Extent {
                detailed: true,
                in_view: true,
            };
        };
        lo = (lo.0.min(p.x), lo.1.min(p.y));
        hi = (hi.0.max(p.x), hi.1.max(p.y));
    }
    if lo.0 > hi.0 {
        return Extent {
            detailed: false,
            in_view: true,
        };
    }
    let diagonal = (hi.0 - lo.0).hypot(hi.1 - lo.1);
    let grow = diagonal / 2.0 + OVERLAY_MARGIN_PX;
    let in_view = ctx.viewport().is_none_or(|v| {
        lo.0 - grow <= v.max.x
            && hi.0 + grow >= v.min.x
            && lo.1 - grow <= v.max.y
            && hi.1 + grow >= v.min.y
    });
    Extent {
        detailed: diagonal >= MIN_DETAIL_PX,
        in_view,
    }
}
