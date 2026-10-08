//! Air corridors and routes: segments of width `AM` between control points,
//! an information block beside the first segment and the name along each.
//! Editions whose template draws only the two sides join them at the bends;
//! others put a circle at each control point.

use crate::construction::{
    GeoGeometry, HandleKind, HandleSpec, LabelPlacement, LabelSpec, PartId, PartRole,
};
use crate::definition::GraphicDefinition;
use crate::edit::HandleId;
use crate::family::{ConstructError, Ctx, vertex_handles};
use crate::geo::{Altitude, GeoPoint, VerticalDatum};
use crate::plane::LocalPlane;
use crate::style::{Fill, Stroke};

/// Vertices of each control-point circle.
const CIRCLE_VERTICES: usize = 72;
/// Feet per metre.
const FEET_PER_METRE: f64 = 3.280_84;

/// Longest miter at a bend, in half-widths; sharper bends are bevelled.
const MITER_LIMIT: f64 = 2.0;

pub(crate) fn construct(
    ctx: &mut Ctx<'_>,
    def: &GraphicDefinition,
    prefix: &str,
    open: bool,
) -> Result<(), ConstructError> {
    let control: Vec<GeoPoint> = def.positions().collect();
    let widths = &def.modifiers.distances_m;
    let max_width = widths.iter().copied().fold(0.0_f64, f64::max);
    if max_width.is_nan() || max_width <= 0.0 {
        return Err(ConstructError::Degenerate {
            symbol: ctx.spec.name(),
            reason: "AM width must be positive",
        });
    }
    // One width per point, or the widest for every point when fewer are given.
    let radius = |i: usize| {
        let w = if widths.len() >= control.len() {
            widths.get(i).copied().unwrap_or(max_width)
        } else {
            max_width
        };
        w / 2.0
    };
    let stroke = Some(ctx.palette.line);
    let part = if open {
        open_sides(ctx, &control, &radius, stroke)?
    } else {
        sides_and_circles(ctx, &control, &radius, stroke)?
    };
    labels(ctx, def, &control, part, max_width, prefix);
    ctx.add_handles(vertex_handles(def));
    if let Some((&a, &b)) = control.first().zip(control.get(1)) {
        let az = ctx.earth.inverse(a, b).azimuth1 - 90.0;
        ctx.add_handles([HandleSpec {
            id: HandleId::Width,
            kind: HandleKind::Width,
            at: ctx.earth.direct(a, az, radius(0)),
        }]);
    }
    Ok(())
}

/// Each leg's two sides, and a circle at each control point.
fn sides_and_circles(
    ctx: &mut Ctx<'_>,
    control: &[GeoPoint],
    radius: &impl Fn(usize) -> f64,
    stroke: Option<Stroke>,
) -> Result<PartId, ConstructError> {
    let mut first = None;
    for (j, pair) in control.windows(2).enumerate() {
        let [a, b] = match pair {
            [a, b] => [*a, *b],
            _ => continue,
        };
        let plane = LocalPlane::new(ctx.earth, a);
        let along = plane.to_xy(b);
        let Some(side) = along.unit().map(|u| u.left().scale(radius(j))) else {
            continue;
        };
        for sign in [1.0, -1.0] {
            let s = side.scale(sign);
            let edge = ctx.geodesic_path(&[plane.to_geo(s), plane.to_geo(along.add(s))])?;
            let id = ctx.add_part(
                PartRole::Boundary,
                GeoGeometry::Line(edge),
                stroke,
                Fill::None,
            );
            first.get_or_insert(id);
        }
    }
    let part = first.ok_or(ConstructError::Degenerate {
        symbol: ctx.spec.name(),
        reason: "a corridor needs two distinct points",
    })?;
    for (i, &p) in control.iter().enumerate() {
        ctx.take_vertices(CIRCLE_VERTICES)?;
        let ring = (0..CIRCLE_VERTICES)
            .map(|k| {
                ctx.earth
                    .direct(p, 360.0 * k as f64 / CIRCLE_VERTICES as f64, radius(i))
            })
            .collect();
        ctx.add_part(
            PartRole::Decoration,
            GeoGeometry::Ring(ring),
            stroke,
            Fill::None,
        );
    }
    Ok(part)
}

/// The two sides as continuous lines, each control point's half-width from
/// the centre line: mitred at a bend, bevelled where the miter would reach
/// past [`MITER_LIMIT`] half-widths, open at the ends.
fn open_sides(
    ctx: &mut Ctx<'_>,
    control: &[GeoPoint],
    radius: &impl Fn(usize) -> f64,
    stroke: Option<Stroke>,
) -> Result<PartId, ConstructError> {
    let mut first = None;
    for side in [-90.0, 90.0] {
        let mut points = Vec::new();
        for (i, &p) in control.iter().enumerate() {
            let r = radius(i);
            let before = i.checked_sub(1).and_then(|j| control.get(j));
            let incoming = before.map(|&q| ctx.earth.inverse(q, p).azimuth2);
            let outgoing = control
                .get(i + 1)
                .map(|&q| ctx.earth.inverse(p, q).azimuth1);
            match (incoming, outgoing) {
                (Some(a), Some(b)) => {
                    let turn = (b - a + 540.0).rem_euclid(360.0) - 180.0;
                    let half = (turn / 2.0).to_radians().cos();
                    if half * MITER_LIMIT >= 1.0 {
                        points.push(ctx.earth.direct(p, a + turn / 2.0 + side, r / half));
                    } else {
                        points.push(ctx.earth.direct(p, a + side, r));
                        points.push(ctx.earth.direct(p, b + side, r));
                    }
                }
                (Some(a), None) | (None, Some(a)) => points.push(ctx.earth.direct(p, a + side, r)),
                (None, None) => {}
            }
        }
        if points.len() < 2 {
            continue;
        }
        let line = ctx.geodesic_path(&points)?;
        let id = ctx.add_part(
            PartRole::Boundary,
            GeoGeometry::Line(line),
            stroke,
            Fill::None,
        );
        first.get_or_insert(id);
    }
    first.ok_or(ConstructError::Degenerate {
        symbol: ctx.spec.name(),
        reason: "a corridor needs two distinct points",
    })
}

/// The information block outside the first segment (between points 1 and
/// 2, clear of the corridor, as the standard asks), and "AC T" inside each
/// segment.
fn labels(
    ctx: &mut Ctx<'_>,
    def: &GraphicDefinition,
    control: &[GeoPoint],
    part: PartId,
    width: f64,
    prefix: &str,
) {
    let m = &def.modifiers;
    let t = m.designation();
    let altitude = |i: usize| m.altitudes.get(i).map(altitude_text);
    let value = |v: &Option<String>| v.as_deref().filter(|w| !w.is_empty()).map(str::to_owned);
    let block: Vec<String> = [
        t.map(|t| format!("Name: {t}")),
        Some(format!("Width: {} M", width.round())),
        altitude(0).map(|a| format!("Min Alt: {a}")),
        altitude(1).map(|a| format!("Max Alt: {a}")),
        value(&m.dtg_start).map(|w| format!("DTG Start: {w}")),
        value(&m.dtg_end).map(|w| format!("DTG End: {w}")),
    ]
    .into_iter()
    .flatten()
    .collect();
    let (Some(&a), Some(&b)) = (control.first(), control.get(1)) else {
        return;
    };
    let mid = ctx.earth.interpolate(a, b, 0.5);
    let bearing = ctx.earth.inverse(mid, b).azimuth1;
    let half = width / 2.0;
    let edges = [
        ctx.earth.direct(mid, bearing - 90.0, half),
        ctx.earth.direct(mid, bearing + 90.0, half),
    ];
    let lines = block.len();
    for (i, text) in block.into_iter().enumerate() {
        ctx.add_label(LabelSpec {
            part,
            text,
            anchor: mid,
            placement: LabelPlacement::OutsideEdge { toward: b, edges },
            // First line on top; lines 1.2 em apart, the last 1.1 em outside
            // the edge so its descenders clear the stroke.
            line_offset: -(1.1 + 1.2 * (lines - 1 - i) as f64),
            may_hide: false,
        });
    }
    let name = t.map_or_else(|| prefix.to_owned(), |t| format!("{prefix} {t}"));
    for pair in control.windows(2) {
        if let [a, b] = pair {
            ctx.add_label(LabelSpec {
                part,
                text: name.clone(),
                anchor: ctx.earth.interpolate(*a, *b, 0.5),
                placement: LabelPlacement::Along { toward: *b },
                line_offset: 0.0,
                may_hide: true,
            });
        }
    }
}

/// Altitude in whole feet with its datum, e.g. "3280 FT AMSL".
pub(super) fn altitude_text(a: &Altitude) -> String {
    let feet = ((a.metres * FEET_PER_METRE * 10.0).round() / 10.0).trunc();
    let datum = match a.datum {
        VerticalDatum::MeanSeaLevel => "AMSL",
        VerticalDatum::AboveGround => "AGL",
        VerticalDatum::Ellipsoid => "HAE",
    };
    format!("{feet} FT {datum}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn altitudes_are_whole_feet_with_datum() {
        let a = |metres, datum| altitude_text(&Altitude::new(metres, datum));
        assert_eq!(a(1000.0, VerticalDatum::MeanSeaLevel), "3280 FT AMSL");
        assert_eq!(a(3000.0, VerticalDatum::MeanSeaLevel), "9842 FT AMSL");
        assert_eq!(a(150.0, VerticalDatum::AboveGround), "492 FT AGL");
    }
}
