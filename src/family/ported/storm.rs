//! Tropical Storm Wind Areas and Date/Time Labels (45 162004) in
//! MIL-STD-2525E change 1 (TABLE M-II): a closed area of winds, with its
//! date-time group (`W`) outside it. Upstream has no line type for the code
//! and draws the control points as an open line.
//!
//! One graphic outlines one wind area. Its template shows three nested
//! areas around the storm symbol; each is a graphic of its own, and the
//! storm itself is a single-point symbol the host places. The outline is
//! red, the template's colour for the outermost (34 knot) area.

use crate::construction::{GeoGeometry, LabelPlacement, LabelSpec, PartRole};
use crate::definition::GraphicDefinition;
use crate::family::{ConstructError, Ctx, vertex_handles};
use crate::geo::GeoPoint;
use crate::style::{Fill, Rgba, Stroke};

/// Whether `def` is drawn here rather than by the ported renderer.
pub(super) fn applies(def: &GraphicDefinition) -> bool {
    let s = &def.symbol;
    (s.version_code(), s.symbol_set(), s.entity().get()) == (15, 45, 162_004)
}

pub(super) fn construct(ctx: &mut Ctx<'_>, def: &GraphicDefinition) -> Result<(), ConstructError> {
    let control: Vec<GeoPoint> = def.positions().collect();
    let ring = ctx.geodesic_ring(&control)?;
    let top = control
        .iter()
        .copied()
        .max_by(|a, b| a.lat().total_cmp(&b.lat()))
        .ok_or(ConstructError::Degenerate {
            symbol: ctx.spec.name(),
            reason: "no control points",
        })?;
    let stroke = Stroke {
        color: Rgba::RED,
        ..ctx.palette.line
    };
    let part = ctx.add_part(
        PartRole::Boundary,
        GeoGeometry::Ring(ring),
        Some(stroke),
        Fill::None,
    );
    if let Some(dtg) = def.modifiers.dtg_start.as_deref().filter(|t| !t.is_empty()) {
        // Above the northernmost point, clear of the outline.
        ctx.add_label(LabelSpec {
            part,
            text: dtg.to_owned(),
            anchor: top,
            placement: LabelPlacement::Centered,
            line_offset: -1.0,
            may_hide: false,
        });
    }
    ctx.add_handles(vertex_handles(def));
    Ok(())
}
