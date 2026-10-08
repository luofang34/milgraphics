//! The version 16 Navigational Rhumb Line (25 220109): its two points are
//! joined by a rhumb line, a line of constant course, not by the geodesic
//! other graphics use. The renderer is given the rhumb line densified, so
//! it draws and labels the line where it lies.
//!
//! AN, the course, is drawn as entered; when it is not entered, the line's
//! own course from point 1 to point 2, in whole degrees, is drawn.

use crate::definition::GraphicDefinition;
use crate::family::{ConstructError, Ctx};
use crate::geo::GeoPoint;
use crate::geodesy::{Rhumb, segment_count};
use crate::modifier::Modifiers;

/// The renderer's points and amplifiers for a rhumb line, or `None` for
/// any other graphic.
pub(super) fn prepare(
    ctx: &mut Ctx<'_>,
    def: &GraphicDefinition,
) -> Result<Option<(Vec<GeoPoint>, Modifiers)>, ConstructError> {
    let s = &def.symbol;
    if (s.version_code(), s.symbol_set(), s.entity().get()) != (16, 25, 220_109) {
        return Ok(None);
    }
    let control: Vec<GeoPoint> = def.positions().collect();
    let [from, to] = control[..] else {
        return Err(ConstructError::Degenerate {
            symbol: ctx.spec.name(),
            reason: "a rhumb line joins two points",
        });
    };
    let line = Rhumb::new(from, to);
    let steps = segment_count(line.distance_m(), ctx.config.geodesic_step_m);
    ctx.take_vertices(steps.saturating_add(1))?;
    let mut points: Vec<GeoPoint> = (0..=steps)
        .map(|i| line.at(i as f64 / steps as f64))
        .collect();
    let mut modifiers = def.modifiers.clone();
    if modifiers.azimuths_deg.is_empty() {
        modifiers.azimuths_deg = vec![line.course_deg().round().rem_euclid(360.0)];
    }
    // The renderer sets AN on the left of the line as it is drawn; the
    // template wants it on the north or west side.
    if !left_is_north_west(line.course_deg()) {
        points.reverse();
    }
    Ok(Some((points, modifiers)))
}

/// Whether, travelling on `course_deg`, the left of the line is the side
/// nearer north-west: its normal there, `course - 90`, lies within 90° of
/// 315°.
pub(super) fn left_is_north_west(course_deg: f64) -> bool {
    let c = course_deg.rem_euclid(360.0);
    !(135.0..=315.0).contains(&c)
}
