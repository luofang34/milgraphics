//! `GetIntegralTextShapes` of Modifier2.java for the "ge" client: the shapes
//! of a boundary, whose line upstream would break for its labels in other
//! clients but draws whole here.

use crate::engine::base::{EngineError, Shape, shape_type};
use crate::engine::tactical_lines as tl;
use crate::engine::tg::Tg;
use crate::engine::tg_utility::segment_colors::msr_segment_colors;
use crate::engine::tg_utility::shape_properties::get_line_stroke;

/// Upstream `GetIntegralTextShapes`: adds the boundary's polyline, split
/// into one shape per segment that has its own colour (from the T1-style
/// segment colour list) and one for the rest. Other line types add nothing.
pub(crate) fn get_integral_text_shapes(
    tg: &Tg,
    shapes: &mut Vec<Shape>,
) -> Result<(), EngineError> {
    if tg.line_type != tl::BOUNDARY || tg.pixels.is_empty() {
        return Ok(());
    }
    let stroke = get_line_stroke(tg.line_thickness, tg.line_style);
    let mut shape = Shape::new(shape_type::POLYLINE);
    shape.line_color = tg.line_color;
    shape.style = tg.line_style;
    shape.stroke = stroke.clone();
    let colors = msr_segment_colors(tg).filter(|c| !c.is_empty());
    let Some(colors) = colors else {
        for (j, p) in tg.pixels.iter().enumerate() {
            if j == 0 {
                shape.move_to(*p);
            } else {
                shape.line_to(*p);
            }
        }
        shapes.push(shape);
        return Ok(());
    };
    for (j, pair) in tg.pixels.windows(2).enumerate() {
        let [pt0, pt1] = pair else { continue };
        let own = i32::try_from(j).ok().and_then(|k| colors.get(&k));
        if let Some(color) = own {
            let mut seg = Shape::new(shape_type::POLYLINE);
            seg.line_color = *color;
            seg.style = tg.line_style;
            seg.stroke = stroke.clone();
            seg.move_to(*pt0);
            seg.line_to(*pt1);
            shapes.push(seg);
        } else {
            shape.move_to(*pt0);
            shape.line_to(*pt1);
        }
    }
    shapes.push(shape);
    Ok(())
}
