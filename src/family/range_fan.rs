//! Weapon/sensor range fan, sector: annular sectors around one point, from
//! `AM` ranges (metres) and `AN` azimuth pairs (degrees from true north).

use crate::construction::{
    Decoration, GeoGeometry, HandleKind, HandleSpec, LabelPlacement, LabelSpec, PartId, PartRole,
};
use crate::definition::GraphicDefinition;
use crate::edit::{EditError, HandleId};
use crate::family::{ConstructError, Ctx, vertex_handles};
use crate::geo::{Altitude, GeoPoint};
use crate::geodesy::Earth;
use crate::style::Fill;

/// Largest angular step between arc vertices, in degrees.
const ARC_STEP_DEG: f64 = 1.0;
/// Orientation pointer head size in pixels.
const POINTER_HEAD_PX: f64 = 10.0;

/// One sector, azimuths in degrees with `right >= left`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Sector {
    pub(crate) left: f64,
    pub(crate) right: f64,
    pub(crate) min_m: f64,
    pub(crate) max_m: f64,
}

/// Sectors from the amplifiers. Sector k spans `AN[2k]..AN[2k+1]` from
/// range `AM[k]` to `AM[k+1]`; when there is one range fewer than that
/// needs, the first sector starts at the centre.
pub(crate) fn sectors(ranges: &[f64], azimuths: &[f64]) -> Vec<Sector> {
    let pairs = azimuths.len() / 2;
    let mut am = ranges.to_vec();
    if am.len() < pairs + 1 && am.first().is_some_and(|&r| r != 0.0) {
        am.insert(0, 0.0);
    }
    azimuths
        .chunks_exact(2)
        .zip(am.windows(2))
        .filter_map(|(an, r)| match (an, r) {
            ([left, right], [min, max]) => {
                let span = (right - left).rem_euclid(360.0);
                let span = if span == 0.0 { 360.0 } else { span };
                Some(Sector {
                    left: *left,
                    right: left + span,
                    min_m: *min,
                    max_m: *max,
                })
            }
            _ => None,
        })
        .collect()
}

/// Mid-azimuth of the sector reaching furthest (the first on ties).
fn orientation(sectors: &[Sector]) -> Option<Sector> {
    sectors
        .iter()
        .copied()
        .fold(None, |best: Option<Sector>, s| match best {
            Some(b) if b.max_m >= s.max_m => Some(b),
            _ => Some(s),
        })
}

fn arc(earth: &Earth, center: GeoPoint, from: f64, to: f64, radius: f64) -> Vec<GeoPoint> {
    if radius <= 0.0 {
        return vec![center];
    }
    let steps = ((to - from).abs() / ARC_STEP_DEG).ceil().max(1.0) as usize;
    (0..=steps)
        .map(|i| earth.direct(center, from + (to - from) * i as f64 / steps as f64, radius))
        .collect()
}

pub(crate) fn construct(ctx: &mut Ctx<'_>, def: &GraphicDefinition) -> Result<(), ConstructError> {
    let degenerate = |reason| ConstructError::Degenerate {
        symbol: ctx.spec.name(),
        reason,
    };
    let center = def
        .positions()
        .next()
        .ok_or(degenerate("a range fan needs its origin"))?;
    let m = &def.modifiers;
    let fan = sectors(&m.distances_m, &m.azimuths_deg);
    let main = orientation(&fan).ok_or(degenerate("a sector needs two ranges and two azimuths"))?;
    if main.max_m.is_nan() || main.max_m <= 0.0 {
        return Err(degenerate("the outer range must be positive"));
    }
    let stroke = Some(ctx.palette.line);
    let mut first = None;
    for s in &fan {
        let mut ring = arc(ctx.earth, center, s.left, s.right, s.min_m);
        if s.min_m <= 0.0 && s.right - s.left >= 360.0 {
            ring.clear();
        }
        ring.extend(arc(ctx.earth, center, s.right, s.left, s.max_m));
        ctx.take_vertices(ring.len())?;
        let id = ctx.add_part(
            PartRole::Boundary,
            GeoGeometry::Ring(ring),
            stroke,
            Fill::None,
        );
        first.get_or_insert(id);
    }
    let part = first.ok_or(degenerate("no sector"))?;
    let bearing = main.left + (main.right - main.left) / 2.0;
    let pointer = ctx.next_part();
    ctx.add_decoration(Decoration::Pointer {
        id: pointer,
        from: center,
        through: ctx.earth.direct(center, bearing, 1.1 * main.max_m),
        head_px: POINTER_HEAD_PX,
        stroke: ctx.palette.line,
    });
    add_labels(ctx, &fan, center, bearing, part, &m.altitudes);
    let earth = ctx.earth;
    ctx.add_handles(vertex_handles(def));
    let handles = handles(earth, center, &m.distances_m, &m.azimuths_deg, bearing);
    ctx.add_handles(handles);
    Ok(())
}

/// Per sector: `ALT` and `RG` stacked on the orientation bearing at mid
/// range, and its azimuths on its edges.
fn add_labels(
    ctx: &mut Ctx<'_>,
    fan: &[Sector],
    center: GeoPoint,
    bearing: f64,
    part: PartId,
    altitudes: &[Altitude],
) {
    let earth = ctx.earth;
    for (k, s) in fan.iter().enumerate() {
        let mid = (s.min_m + s.max_m) / 2.0;
        if let Some(a) = altitudes.get(k) {
            ctx.add_label(LabelSpec {
                part,
                text: format!("ALT {}", super::corridor::altitude_text(a)),
                anchor: earth.direct(center, bearing, mid),
                placement: LabelPlacement::Centered,
                line_offset: 0.0,
                may_hide: true,
            });
        }
        let labels = [
            (
                format!("RG {}", s.max_m.round()),
                earth.direct(center, bearing, mid),
                -1.0,
            ),
            (azimuth_text(s.left), earth.direct(center, s.left, mid), 0.0),
            (
                azimuth_text(s.right.rem_euclid(360.0)),
                earth.direct(center, s.right, mid),
                0.0,
            ),
        ];
        for (text, anchor, line_offset) in labels {
            ctx.add_label(LabelSpec {
                part,
                text,
                anchor,
                placement: LabelPlacement::Centered,
                line_offset,
                may_hide: true,
            });
        }
    }
}

fn handles(
    earth: &Earth,
    center: GeoPoint,
    ranges: &[f64],
    azimuths: &[f64],
    bearing: f64,
) -> Vec<HandleSpec> {
    let fan = sectors(ranges, azimuths);
    let ranges = ranges.iter().enumerate().map(|(i, &r)| HandleSpec {
        id: HandleId::Range(i as u16),
        kind: HandleKind::Range,
        at: earth.direct(center, bearing, r),
    });
    let azimuths = azimuths.iter().enumerate().map(|(i, &a)| HandleSpec {
        id: HandleId::Azimuth(i as u16),
        kind: HandleKind::Azimuth,
        at: earth.direct(center, a, fan.get(i / 2).map_or(0.0, |s| s.max_m)),
    });
    ranges.chain(azimuths).collect()
}

/// Moves a range or azimuth handle: the new value is the distance or
/// azimuth from the fan's origin to `to`.
pub(crate) fn move_handle(
    def: &mut GraphicDefinition,
    handle: HandleId,
    to: GeoPoint,
) -> Result<(), EditError> {
    let missing = EditError::NoSuchHandle { handle };
    let center = def.positions().next().ok_or(missing)?;
    let inv = Earth::wgs84().inverse(center, to);
    let m = &mut def.modifiers;
    let slot = match handle {
        HandleId::Range(i) => m
            .distances_m
            .get_mut(usize::from(i))
            .map(|v| (v, inv.distance_m)),
        HandleId::Azimuth(i) => m
            .azimuths_deg
            .get_mut(usize::from(i))
            .map(|v| (v, inv.azimuth1.rem_euclid(360.0))),
        HandleId::Vertex(_) | HandleId::Width => None,
    };
    let (value, new) = slot.ok_or(EditError::NoSuchHandle { handle })?;
    *value = new;
    Ok(())
}

/// An azimuth as the operator would write it: no trailing ".0".
fn azimuth_text(deg: f64) -> String {
    let rounded = (deg * 10.0).round() / 10.0;
    if rounded.fract() == 0.0 {
        format!("{rounded:.0}")
    } else {
        format!("{rounded:.1}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_range_starts_at_the_centre() {
        let s = sectors(&[5000.0], &[30.0, 90.0]);
        assert_eq!(
            s,
            vec![Sector {
                left: 30.0,
                right: 90.0,
                min_m: 0.0,
                max_m: 5000.0
            }]
        );
    }

    #[test]
    fn consecutive_sectors_share_ranges() {
        let s = sectors(&[1000.0, 5000.0, 8000.0], &[30.0, 90.0, 300.0, 350.0]);
        assert_eq!(s.len(), 2);
        assert_eq!((s[0].min_m, s[0].max_m), (1000.0, 5000.0));
        assert_eq!(
            (s[1].min_m, s[1].max_m, s[1].left, s[1].right),
            (5000.0, 8000.0, 300.0, 350.0)
        );
        assert_eq!(orientation(&s).map(|o| o.max_m), Some(8000.0));
    }

    #[test]
    fn sectors_across_north_and_full_circles() {
        let s = sectors(&[0.0, 1000.0], &[350.0, 10.0]);
        assert_eq!((s[0].left, s[0].right), (350.0, 370.0));
        let full = sectors(&[0.0, 1000.0], &[45.0, 45.0]);
        assert_eq!(full[0].right - full[0].left, 360.0);
    }

    #[test]
    fn azimuth_labels_drop_trailing_zeros() {
        assert_eq!(azimuth_text(30.0), "30");
        assert_eq!(azimuth_text(12.25), "12.3");
    }
}
