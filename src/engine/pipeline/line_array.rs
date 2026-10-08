//! Upstream `clsRenderer2.GetLineArray`: the shapes of one graphic from its
//! prepared points, by the family of its line type, then the labels placed
//! from the drawn points.

use super::support::Wiring;
use crate::engine::api::Input;
use crate::engine::arraysupport::get_line_array2;
use crate::engine::arraysupport::inside_outside::get_inside_outside_double2;
use crate::engine::base::{EngineError, Pt, Shape};
use crate::engine::channel_utility::draw::draw_channel;
use crate::engine::cpof::areas::change1_tactical_areas;
use crate::engine::line_type::classes::{is_change1_area, is_channel};
use crate::engine::lineutility::basics::calc_distance_double;
use crate::engine::metoc::get_shape::get_me_toc_shape;
use crate::engine::modifier;
use crate::engine::modifier::post::SectorFrame;
use crate::engine::render_utility::msr::get_msr_shapes;
use crate::engine::settings::Settings;
use crate::engine::tactical_lines as tl;
use crate::engine::tg::Tg;
use crate::engine::tg_utility::shape_properties::set_shape_properties;

/// The shapes of `tg`; `control` are its control points before preparation,
/// which the change 1 areas are built from.
pub(super) fn get_line_array(
    tg: &mut Tg,
    input: &Input<'_>,
    settings: &Settings,
    control: &[Pt],
) -> Result<Vec<Shape>, EngineError> {
    let mut shapes = Vec::new();
    if tg.pixels.is_empty() {
        return Ok(shapes);
    }
    let input_type = tg.line_type;
    let line_type = downgraded(tg, settings)?;
    tg.line_type = line_type;
    if line_type == tl::SINGLEC {
        tg.pixels.reverse();
    }
    let symbol = input.symbol;
    let metoc = crate::engine::line_type::is_weather(symbol.symbol_set(), symbol.entity().get())
        .is_some_and(|t| t > 0);
    let min_points = input.ms_info.map_or(-1, |m| m.min_points);
    let enough = i32::try_from(tg.pixels.len()).unwrap_or(i32::MAX) >= min_points;
    let wiring = Wiring { settings };
    if enough && is_change1_area(line_type) {
        tg.pixels.clear();
        let inside_outside =
            |a, b, pts: &[Pt], n, i, lt| get_inside_outside_double2(a, b, pts, n, i, lt);
        change1_tactical_areas(
            tg,
            line_type,
            control,
            input.meters_per_pixel,
            &mut shapes,
            &inside_outside,
        );
    } else if metoc {
        if tg.pixels.len() < 2 {
            return Ok(Vec::new());
        }
        let pattern = get_me_toc_shape(tg, &mut shapes, settings, &wiring)
            .ok()
            .flatten();
        if let Some(p) = pattern {
            if let Some(shape) = shapes.get_mut(p.shape_index) {
                shape.metoc_pattern = Some(p.line_type);
            }
        }
    } else {
        if tg.pixels.len() < 2 && line_type != tl::BS_CROSS {
            return Ok(Vec::new());
        }
        geometry(tg, line_type, settings, &wiring, &mut shapes)?;
    }
    let routes = matches!(line_type, tl::ASR | tl::MSR | tl::TRAFFIC_ROUTE);
    if !metoc && !routes {
        set_shape_properties(tg, &mut shapes);
    }
    tg.line_type = input_type;
    let frame = SectorFrame {
        meters_per_pixel: input.meters_per_pixel,
        origin: control.first().copied().unwrap_or_default(),
    };
    modifier::post::add_modifiers2(tg, settings, input.text_width, frame)?;
    modifier::integral_shapes::get_integral_text_shapes(tg, &mut shapes)?;
    Ok(shapes)
}

/// Short convoys and follow-and-assume arrows are drawn as supporting
/// attacks.
fn downgraded(tg: &Tg, settings: &Settings) -> Result<i32, EngineError> {
    let line_type = tg.line_type;
    if !matches!(line_type, tl::FOLLA | tl::FOLSP | tl::CONVOY) {
        return Ok(line_type);
    }
    let (Some(&a), Some(&b)) = (tg.pixels.first(), tg.pixels.get(1)) else {
        return Err(EngineError::Index {
            index: 1,
            len: tg.pixels.len(),
        });
    };
    let short = calc_distance_double(a, b) <= 30.0 * settings.dpi_scale_factor();
    Ok(if short { tl::DIRATKSPT } else { line_type })
}

fn geometry(
    tg: &mut Tg,
    line_type: i32,
    settings: &Settings,
    wiring: &Wiring<'_>,
    shapes: &mut Vec<Shape>,
) -> Result<(), EngineError> {
    if is_channel(line_type) {
        draw_channel(tg, line_type, settings, wiring, shapes)
    } else if matches!(line_type, tl::ASR | tl::MSR | tl::TRAFFIC_ROUTE) {
        get_msr_shapes(tg, shapes);
        Ok(())
    } else {
        let pixels = tg.pixels.clone();
        tg.pixels = get_line_array2(tg, &pixels, shapes, settings)?;
        Ok(())
    }
}
