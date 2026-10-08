//! Phase Line: an open line labelled "PL T" at both ends.

use crate::construction::{GeoGeometry, LabelPlacement, LabelSpec, PartRole};
use crate::definition::GraphicDefinition;
use crate::family::{ConstructError, Ctx, vertex_handles};
use crate::geo::GeoPoint;
use crate::style::Fill;

pub(crate) fn construct(ctx: &mut Ctx<'_>, def: &GraphicDefinition) -> Result<(), ConstructError> {
    let control: Vec<GeoPoint> = def.positions().collect();
    let path = ctx.geodesic_path(&control)?;
    let text = match def.modifiers.designation() {
        Some(t) => format!("PL {t}"),
        None => "PL".to_owned(),
    };
    let ends = match (
        path.first(),
        path.get(1),
        path.last(),
        path.iter().rev().nth(1),
    ) {
        (Some(&start), Some(&after), Some(&end), Some(&before)) => [(start, after), (end, before)],
        _ => {
            return Err(ConstructError::Degenerate {
                symbol: ctx.spec.name,
                reason: "a line needs two distinct points",
            });
        }
    };
    let stroke = Some(ctx.palette.line);
    let part = ctx.add_part(PartRole::Line, GeoGeometry::Line(path), stroke, Fill::None);
    for (anchor, inward) in ends {
        ctx.add_label(LabelSpec {
            part,
            text: text.clone(),
            anchor,
            placement: LabelPlacement::LineEnd { inward },
            line_offset: 0.0,
            may_hide: false,
        });
    }
    ctx.add_handles(vertex_handles(def));
    Ok(())
}
