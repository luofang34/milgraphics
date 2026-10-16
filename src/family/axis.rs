//! Axes of advance: a two-sided body along the centreline with an arrowhead,
//! drawn in the form of each kind ([`AxisKind`]).
//!
//! Point 1 is the arrow tip, points 2..N-1 run back along the centreline,
//! and point N sets the width (its distance from the last centreline
//! segment) and the arrowhead length (its distance from the tip along that
//! segment). Everything is sized on the ground, so it scales with the map
//! and the whole graphic belongs to the geographic tier. The rear end stays
//! open.

use crate::construction::{GeoGeometry, LabelPlacement, LabelSpec, PartId, PartRole};
use crate::definition::GraphicDefinition;
use crate::family::{ConstructError, Ctx, vertex_handles};
use crate::geo::GeoPoint;
use crate::plane::{LocalPlane, Xy, offset_polyline};
use crate::style::{DashPattern, Fill, Stroke};

mod figures;
#[cfg(test)]
mod tests;

/// Which axis of advance is drawn; each is the same body and arrowhead
/// sized from the same points, with its own head and figures.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum AxisKind {
    /// Main Attack: the arrowhead is closed by a notch behind the tip.
    Main,
    /// Supporting Attack: a plain, unfilled arrowhead.
    Supporting,
    /// Airborne/Aviation: the two sides cross over the last body segment.
    Airborne,
    /// Attack Helicopter: the Airborne form with a rotor at the crossing.
    AttackHelicopter,
    /// Feint: the Supporting form with a dashed chevron ahead of the head.
    Feint,
}

impl AxisKind {
    fn crossed(self) -> bool {
        matches!(self, Self::Airborne | Self::AttackHelicopter)
    }
}

/// The axis geometry in the local plane.
struct Shape {
    lower: Vec<Xy>,
    upper: Vec<Xy>,
    head: Vec<Xy>,
    /// Extra figures of the kind, each an open polyline, with whether it is
    /// dashed.
    figures: Vec<(Vec<Xy>, bool)>,
}

pub(crate) fn construct(
    ctx: &mut Ctx<'_>,
    def: &GraphicDefinition,
    kind: AxisKind,
) -> Result<(), ConstructError> {
    let control: Vec<GeoPoint> = def.positions().collect();
    let degenerate = |reason| ConstructError::Degenerate {
        symbol: ctx.spec.name(),
        reason,
    };
    let origin = *control.first().ok_or(degenerate("an axis needs a tip"))?;
    let plane = LocalPlane::new(ctx.earth, origin);
    let xy: Vec<Xy> = control.iter().map(|&p| plane.to_xy(p)).collect();
    let shape = shape(&xy, kind).ok_or(degenerate(
        "the tip, second and width points must not coincide",
    ))?;
    let figure_vertices: usize = shape.figures.iter().map(|(f, _)| f.len()).sum();
    ctx.take_vertices(shape.lower.len() + shape.upper.len() + shape.head.len() + figure_vertices)?;
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
    for (figure, dashed) in &shape.figures {
        let stroke = if *dashed {
            Stroke {
                dash: DashPattern::Dashed,
                ..ctx.palette.solid_line
            }
        } else {
            ctx.palette.line
        };
        ctx.add_part(
            PartRole::Decoration,
            GeoGeometry::Line(to_geo(figure)),
            Some(stroke),
            Fill::None,
        );
    }
    add_labels(ctx, def, &control, body);
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

fn shape(points: &[Xy], kind: AxisKind) -> Option<Shape> {
    let (centre, width_point) = points.split_at(points.len().checked_sub(1)?);
    let (&tip, &width_point) = (centre.first()?, width_point.first()?);
    let second = *centre.get(1)?;
    let axis = second.sub(tip).unit()?;
    let rel = width_point.sub(tip);
    let half = rel.cross(axis).abs() / 2.0;
    let head_len = rel.dot(axis).abs();
    // Widths and lengths below a millionth of the first segment are rounding
    // noise of a width point placed on the axis, not a drawable shape.
    let eps = 1e-6 * second.sub(tip).len();
    if !(half > eps && head_len > eps) {
        return None;
    }
    let base = tip.add(axis.scale(head_len));
    // Body centreline from the tail to the arrowhead base.
    let mut line: Vec<Xy> = centre.iter().skip(1).rev().copied().collect();
    line.push(base);
    let mut lower = offset_polyline(&line, -half)?;
    let mut upper = offset_polyline(&line, half)?;
    let (l, u) = (*lower.last()?, *upper.last()?);
    let across = u.sub(l).unit()?;
    let barb_l = l.sub(across.scale(half));
    let barb_u = u.add(across.scale(half));
    let head = if kind == AxisKind::Main {
        let inner = tip.add(base.sub(tip).unit()?.scale(half));
        vec![tip, barb_l, l, inner, u, barb_u, tip]
    } else {
        vec![l, barb_l, tip, barb_u, u]
    };
    let mut figures = Vec::new();
    if kind.crossed() {
        // Each side ends on the other's side of the arrowhead base.
        *lower.last_mut()? = u;
        *upper.last_mut()? = l;
    }
    match kind {
        AxisKind::AttackHelicopter => {
            let rotor = figures::rotor(&lower, &upper, &line, half)?;
            figures.extend(rotor.into_iter().map(|f| (f, false)));
        }
        AxisKind::Feint => figures.push((figures::feint(tip, base, barb_l, barb_u), true)),
        AxisKind::Main | AxisKind::Supporting | AxisKind::Airborne => {}
    }
    Some(Shape {
        lower,
        upper,
        head,
        figures,
    })
}

/// `W - `, `W1` and `T` above and below the middle of the axis: on the first
/// segment for one-segment axes and the second for two, as three stacked
/// lines; for longer ones the date-time groups straddle the second segment
/// and `T` sits just above the third (as mil-sym-java).
fn add_labels(ctx: &mut Ctx<'_>, def: &GraphicDefinition, control: &[GeoPoint], part: PartId) {
    let m = &def.modifiers;
    let dtg = non_empty(&m.dtg_start).map(|w| format!("{w} - "));
    let dtg1 = non_empty(&m.dtg_end).map(str::to_owned);
    let name = m.designation().map(str::to_owned);
    let lines = match control.len() {
        0..=3 => [(dtg, 0, 0.0), (dtg1, 0, 1.0), (name, 0, 2.0)],
        4 => [(dtg, 1, 0.0), (dtg1, 1, 1.0), (name, 1, 2.0)],
        _ => [(dtg, 1, -0.5), (dtg1, 1, 0.5), (name, 2, -0.5)],
    };
    for (text, segment, line_offset) in lines {
        let (Some(text), Some(&a), Some(&b)) =
            (text, control.get(segment), control.get(segment + 1))
        else {
            continue;
        };
        ctx.add_label(LabelSpec {
            part,
            text,
            anchor: ctx.earth.interpolate(a, b, 0.5),
            placement: LabelPlacement::Along { toward: b },
            line_offset,
            may_hide: false,
        });
    }
}

fn non_empty(text: &Option<String>) -> Option<&str> {
    text.as_deref().filter(|t| !t.is_empty())
}
