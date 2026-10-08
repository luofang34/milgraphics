//! Air corridors: segments of width `AM` between control points, with a
//! circle at each control point and an information block on the middle
//! segment.

use crate::construction::{
    GeoGeometry, HandleKind, HandleSpec, LabelPlacement, LabelSpec, PartRole,
};
use crate::definition::GraphicDefinition;
use crate::edit::HandleId;
use crate::family::{ConstructError, Ctx, vertex_handles};
use crate::geo::{Altitude, GeoPoint, VerticalDatum};
use crate::plane::LocalPlane;
use crate::style::Fill;

/// Vertices of each control-point circle.
const CIRCLE_VERTICES: usize = 72;
/// Feet per metre.
const FEET_PER_METRE: f64 = 3.280_84;

pub(crate) fn construct(ctx: &mut Ctx<'_>, def: &GraphicDefinition) -> Result<(), ConstructError> {
    let control: Vec<GeoPoint> = def.positions().collect();
    let widths = &def.modifiers.distances_m;
    let max_width = widths.iter().copied().fold(0.0_f64, f64::max);
    if max_width.is_nan() || max_width <= 0.0 {
        return Err(ConstructError::Degenerate {
            symbol: ctx.spec.name,
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
        symbol: ctx.spec.name,
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
    labels(ctx, def, &control, part, max_width);
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

/// The information block, stacked above the middle segment, and "AC T" on it.
fn labels(
    ctx: &mut Ctx<'_>,
    def: &GraphicDefinition,
    control: &[GeoPoint],
    part: crate::construction::PartId,
    width: f64,
) {
    let middle = control.len().div_ceil(2).saturating_sub(1);
    let (Some(&a), Some(&b)) = (control.get(middle), control.get(middle + 1)) else {
        return;
    };
    let anchor = ctx.earth.interpolate(a, b, 0.5);
    let m = &def.modifiers;
    let t = m.designation();
    let altitude = |i: usize| m.altitudes.get(i).map(altitude_text);
    let lines = [
        (-7.0, t.map(|t| format!("Name: {t}"))),
        (-6.0, Some(format!("Width: {} M", width.round()))),
        (-5.0, altitude(0).map(|a| format!("Min Alt: {a}"))),
        (-4.0, altitude(1).map(|a| format!("Max Alt: {a}"))),
        (
            -3.0,
            m.dtg_start
                .as_deref()
                .filter(|w| !w.is_empty())
                .map(|w| format!("DTG Start: {w}")),
        ),
        (
            -2.0,
            m.dtg_end
                .as_deref()
                .filter(|w| !w.is_empty())
                .map(|w| format!("DTG End: {w}")),
        ),
        (
            0.0,
            Some(t.map_or_else(|| "AC".to_owned(), |t| format!("AC {t}"))),
        ),
    ];
    for (line_offset, text) in lines {
        if let Some(text) = text {
            ctx.add_label(LabelSpec {
                part,
                text,
                anchor,
                placement: LabelPlacement::Along { toward: b },
                line_offset,
                may_hide: false,
            });
        }
    }
}

/// Altitude in whole feet with its datum, e.g. "3280 FT AMSL".
fn altitude_text(a: &Altitude) -> String {
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
        let a = |metres, datum| altitude_text(&Altitude { metres, datum });
        assert_eq!(a(1000.0, VerticalDatum::MeanSeaLevel), "3280 FT AMSL");
        assert_eq!(a(3000.0, VerticalDatum::MeanSeaLevel), "9842 FT AMSL");
        assert_eq!(a(150.0, VerticalDatum::AboveGround), "492 FT AGL");
    }
}
