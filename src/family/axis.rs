//! Axes of advance (Main Attack): a two-sided body along the centreline with
//! a barbed, notched arrowhead.
//!
//! Point 1 is the arrow tip, points 2..N-1 run back along the centreline,
//! and point N sets the width (its distance from the last centreline
//! segment) and the arrowhead length (its distance from the tip along that
//! segment). Everything is sized on the ground, so it scales with the map.

use crate::construction::{GeoGeometry, LabelPlacement, LabelSpec, PartRole};
use crate::definition::GraphicDefinition;
use crate::family::{ConstructError, Ctx, vertex_handles};
use crate::geo::GeoPoint;
use crate::plane::{LocalPlane, Xy, offset_polyline};
use crate::style::Fill;

/// The axis geometry in the local plane.
struct Shape {
    lower: Vec<Xy>,
    upper: Vec<Xy>,
    head: Vec<Xy>,
}

pub(crate) fn construct(ctx: &mut Ctx<'_>, def: &GraphicDefinition) -> Result<(), ConstructError> {
    let control: Vec<GeoPoint> = def.positions().collect();
    let degenerate = |reason| ConstructError::Degenerate {
        symbol: ctx.spec.name,
        reason,
    };
    let origin = *control.first().ok_or(degenerate("an axis needs a tip"))?;
    let plane = LocalPlane::new(ctx.earth, origin);
    let xy: Vec<Xy> = control.iter().map(|&p| plane.to_xy(p)).collect();
    let shape = shape(&xy).ok_or(degenerate(
        "the tip, second and width points must not coincide",
    ))?;
    ctx.take_vertices(shape.lower.len() + shape.upper.len() + shape.head.len())?;
    let to_geo = |pts: &[Xy]| pts.iter().map(|&p| plane.to_geo(p)).collect::<Vec<_>>();
    let (lower, upper, head) = (
        to_geo(&shape.lower),
        to_geo(&shape.upper),
        to_geo(&shape.head),
    );
    let stroke = Some(ctx.palette.line);
    let body = ctx.add_part(PartRole::Line, GeoGeometry::Line(lower), stroke, Fill::None);
    ctx.add_part(PartRole::Line, GeoGeometry::Line(upper), stroke, Fill::None);
    ctx.add_part(
        PartRole::Arrowhead,
        GeoGeometry::Line(head),
        stroke,
        Fill::None,
    );
    if let Some(t) = def.modifiers.designation() {
        // Below the first segment for one-segment axes, below the second for
        // two, and just above the third for longer ones (as mil-sym-java).
        let (segment, line_offset) = match control.len() {
            0..=3 => (0, 2.0),
            4 => (1, 2.0),
            _ => (2, -0.5),
        };
        if let (Some(&a), Some(&b)) = (control.get(segment), control.get(segment + 1)) {
            ctx.add_label(LabelSpec {
                part: body,
                text: t.to_owned(),
                anchor: ctx.earth.interpolate(a, b, 0.5),
                placement: LabelPlacement::Along { toward: b },
                line_offset,
                may_hide: false,
            });
        }
    }
    ctx.add_handles(
        vertex_handles(def)
            .into_iter()
            .enumerate()
            .map(|(i, mut h)| {
                if i + 1 == control.len() {
                    h.kind = crate::construction::HandleKind::Width;
                }
                h
            }),
    );
    Ok(())
}

fn shape(points: &[Xy]) -> Option<Shape> {
    let (centre, width_point) = points.split_at(points.len().checked_sub(1)?);
    let (&tip, &width_point) = (centre.first()?, width_point.first()?);
    let second = *centre.get(1)?;
    let axis = second.sub(tip).unit()?;
    let rel = width_point.sub(tip);
    let half = rel.cross(axis).abs() / 2.0;
    let head_len = rel.dot(axis).abs();
    if !(half > 0.0 && head_len > 0.0) {
        return None;
    }
    let base = tip.add(axis.scale(head_len));
    // Body centreline from the tail to the arrowhead base.
    let mut line: Vec<Xy> = centre.iter().skip(1).rev().copied().collect();
    line.push(base);
    let lower = offset_polyline(&line, -half)?;
    let upper = offset_polyline(&line, half)?;
    let (&l, &u) = (lower.last()?, upper.last()?);
    let across = u.sub(l).unit()?;
    let inner = tip.add(base.sub(tip).unit()?.scale(half));
    let barb_l = l.sub(across.scale(half));
    let barb_u = u.add(across.scale(half));
    Some(Shape {
        lower,
        upper,
        head: vec![tip, barb_l, l, inner, u, barb_u, tip],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn straight_axis_proportions() {
        // Tip at the origin pointing west, second point 10 km east, width
        // point 1 km behind the tip and 400 m off the axis.
        let pts = [
            Xy::new(0.0, 0.0),
            Xy::new(10_000.0, 0.0),
            Xy::new(1_000.0, 400.0),
        ];
        let s = shape(&pts).unwrap();
        assert_eq!(
            s.lower,
            vec![Xy::new(10_000.0, 200.0), Xy::new(1_000.0, 200.0)]
        );
        assert_eq!(
            s.upper,
            vec![Xy::new(10_000.0, -200.0), Xy::new(1_000.0, -200.0)]
        );
        assert_eq!(
            s.head,
            vec![
                Xy::new(0.0, 0.0),
                Xy::new(1_000.0, 400.0),
                Xy::new(1_000.0, 200.0),
                Xy::new(200.0, 0.0),
                Xy::new(1_000.0, -200.0),
                Xy::new(1_000.0, -400.0),
                Xy::new(0.0, 0.0),
            ]
        );
    }

    #[test]
    fn bends_keep_every_centreline_point() {
        let pts = [
            Xy::new(0.0, 0.0),
            Xy::new(5_000.0, 0.0),
            Xy::new(5_000.0, 5_000.0),
            Xy::new(500.0, 100.0),
        ];
        let s = shape(&pts).unwrap();
        // Tail, mitred bend at point 2, then the arrowhead base.
        assert_eq!(s.lower.len(), 3);
        assert_eq!(s.lower[1], Xy::new(4_950.0, 50.0));
        assert_eq!(s.upper[1], Xy::new(5_050.0, -50.0));
    }

    #[test]
    fn degenerate_widths_are_refused() {
        let on_axis = [Xy::new(0.0, 0.0), Xy::new(10.0, 0.0), Xy::new(5.0, 0.0)];
        assert!(shape(&on_axis).is_none());
        let same = [Xy::new(0.0, 0.0), Xy::new(0.0, 0.0), Xy::new(5.0, 5.0)];
        assert!(shape(&same).is_none());
    }
}
