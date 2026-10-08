//! Port of `RenderMultipoints/clsUtility.addHatchFills`.

use crate::engine::base::{EngineError, Hatch, Shape};
use crate::engine::line_type::classes::is_change1_area;
use crate::engine::tactical_lines::*;
use crate::engine::tg::Tg;
use crate::engine::tg_utility::line_classes::is_closed_polygon;
use crate::style::Rgba;

/// `clsUtility.Hatch_ForwardDiagonal`.
pub(crate) const HATCH_FORWARD_DIAGONAL: i32 = 2;
/// `clsUtility.Hatch_BackwardDiagonal`.
pub(crate) const HATCH_BACKWARD_DIAGONAL: i32 = 3;
const YELLOW: Rgba = Rgba::opaque(255, 255, 0);
const GRAY: Rgba = Rgba::opaque(128, 128, 128);

/// Upstream `addHatchFills`: the hatch of area fills. With
/// `tg.use_hatch_fill` (the default) upstream attaches a hatch image to the
/// shape that has the hatch style; here the shape carries the `Hatch`
/// parameters instead. Without it, upstream builds the hatch lines by
/// intersecting a zig-zag with the shape's area (`java.awt.geom.Area`),
/// which is not ported: that case returns an error.
pub(crate) fn add_hatch_fills(tg: &Tg, shapes: &mut Vec<Shape>) -> Result<(), EngineError> {
    if shapes.is_empty() || !hatchable(tg.line_type) {
        return Ok(());
    }
    // Java float arithmetic.
    let scale = tg.pattern_scale as f32;
    let mut thickness = (tg.line_thickness as f32 * 0.75) * scale;
    let mut color = tg.line_color;
    let mut spacing = (thickness * 6.0) as i32;
    let mut style = tg.fill_style;
    match tg.line_type {
        NFA | NFA_CIRCULAR | NFA_RECTANGULAR | LAA => style = HATCH_BACKWARD_DIAGONAL,
        BIO | BIOT | NUC | CHEM | CHEMT | RAD | RADT => {
            style = HATCH_BACKWARD_DIAGONAL;
            color = Some(YELLOW);
            thickness = (tg.line_thickness as f32 * 0.85) * scale;
        }
        WFZ_REVD | WFZ => {
            style = HATCH_BACKWARD_DIAGONAL;
            if tg.line_color == Some(Rgba::BLACK) {
                color = Some(GRAY);
            }
            spacing /= 2;
        }
        OBSAREA => {
            // Upstream removes without stepping back, so the shape after a
            // removed one is skipped.
            let mut j = 0;
            while j < shapes.len() {
                if shapes
                    .get(j)
                    .is_some_and(|s| s.line_color.is_some_and(is_clear))
                {
                    shapes.remove(j);
                }
                j += 1;
            }
            style = HATCH_BACKWARD_DIAGONAL;
            spacing = (f64::from(spacing) * 1.25) as i32;
        }
        _ => {
            if style <= 0 {
                return Ok(());
            }
        }
    }
    let index = shapes
        .iter()
        .position(|s| s.fill_style == style)
        .unwrap_or(0);
    let fans = matches!(tg.line_type, RANGE_FAN | RANGE_FAN_SECTOR | RADAR_SEARCH);
    for k in 0..shapes.len() {
        let target = if fans { k } else { index };
        let Some(shape) = shapes.get_mut(target) else {
            return Ok(());
        };
        if fans {
            style = shape.fill_style;
        }
        if style >= HATCH_FORWARD_DIAGONAL {
            if !tg.use_hatch_fill {
                return Err(EngineError::Degenerate(
                    "hatch lines clipped to the area are not supported",
                ));
            }
            shape.pattern_fill = Some(Hatch {
                style,
                spacing,
                thickness: thickness as i32,
                color,
            });
        }
        if !fans {
            break;
        }
    }
    Ok(())
}

/// Java `Color.getRGB() == 0`: fully transparent black.
fn is_clear(c: Rgba) -> bool {
    c.a == 0 && c.r == 0 && c.g == 0 && c.b == 0
}

/// Closed areas, change 1 areas and the buffer-zone graphics take hatching.
fn hatchable(line_type: i32) -> bool {
    is_closed_polygon(line_type)
        || is_change1_area(line_type)
        || matches!(line_type, BBS_AREA | BBS_LINE | BBS_RECTANGLE)
}
