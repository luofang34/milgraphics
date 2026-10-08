//! The order upstream's `clsRenderer.render_GE` runs the renderer in, with
//! clipping off: prepare the points, place the labels known before the
//! geometry, build the shapes (`GetLineArray`), add fills, then turn the
//! labels and shapes into the output.

use super::api::{Input, Justify, Label, Output};
use super::base::{EngineError, Pt, Shape};
use super::build::{Overrides, build_tg};
use super::cpof::filter::{clear_pixels_style, filter_points2, lines_with_separate_fill};
use super::cpof::range_fan::circular_range_fan_fill_tg;
use super::lineutility::exterior::get_deep_copy;
use super::modifier::{self, PlacedLabel};
use super::render_utility::fdi::{add_fdi, has_fdi};
use super::render_utility::fills::{
    add_abatis_fill, lines_with_fill_shapes, resolve_post_clipped_shapes,
};
use super::render_utility::hatch::add_hatch_fills;
use super::render_utility::interpolate::interpolate_pixels;
use super::render_utility::msr::get_autoshape_fill_shape;
use super::render_utility::polylines::{set_spline_linetype, shapes_to_polylines};
use super::settings::Settings;
use super::tactical_lines as tl;
use super::tg::Tg;
use super::tg_utility::axad_filter::filter_axad_points;
use super::tg_utility::lc_points::{reverse_usas_lc_points_by_quadrant, segment_lc_points};
use super::tg_utility::points::{
    filter_vertical_segments, remove_duplicate_points, reverse_points_rev_d,
};

mod line_array;
mod support;

#[cfg(test)]
mod tests;

/// Draws one graphic as `render_GE` does.
pub(crate) fn render(input: &Input<'_>) -> Result<Output, EngineError> {
    let settings = Settings::default();
    let overrides = Overrides {
        line_color: input.style.line_color,
        fill_color: input.style.fill_color,
        ..Overrides::default()
    };
    let mut tg = build_tg(input, &settings, &overrides)?;
    let control = tg.pixels.clone();
    prepare(&mut tg, input, &settings)?;
    let fill_shapes = fills_of_original(&mut tg, &control);
    let saved_fill_style = tg.fill_style;
    if tg.line_type == tl::RANGE_FAN {
        tg.fill_style = 0;
    }
    let mut shapes = line_array::get_line_array(&mut tg, input, &settings, &control)?;
    if input.symbol.symbol_set() == 25 && has_fdi(input.symbol.hq_tf_dummy()) {
        add_fdi(&tg, &mut shapes, input.ms_info, input.symbol.entity().get());
    }
    add_fan_fill(
        &mut tg,
        input,
        &settings,
        &control,
        saved_fill_style,
        &mut shapes,
    )?;
    lines_with_separate_fill(tg.line_type, &mut shapes);
    add_abatis_fill(&tg, &mut shapes);
    if let Some(mut fill) = fill_shapes.filter(|f| !f.is_empty()) {
        fill.append(&mut shapes);
        shapes = fill;
    }
    resolve_post_clipped_shapes(&tg, &mut shapes);
    let labels = modifier::display::display_modifiers2(&tg, input.text_width, false);
    add_hatch_fills(&tg, &mut shapes)?;
    shapes_to_polylines(&tg, &mut shapes);
    Ok(Output {
        shapes,
        labels: labels.into_iter().map(label).collect(),
    })
}

/// Point preparation before the geometry, and the labels placed from the
/// control points (`AddModifiersGeo`).
fn prepare(tg: &mut Tg, input: &Input<'_>, settings: &Settings) -> Result<(), EngineError> {
    reverse_points_rev_d(tg);
    if tg.line_type == tl::LC {
        reverse_usas_lc_points_by_quadrant(tg)?;
        segment_lc_points(tg)?;
    }
    remove_duplicate_points(tg, input.ms_info);
    set_spline_linetype(tg);
    if settings.use_line_interpolation {
        interpolate_pixels(tg);
    }
    tg.modifiers.clear();
    modifier::geo::add_modifiers_geo(tg, settings, input.text_width)?;
    filter_points2(tg);
    filter_vertical_segments(tg);
    filter_axad_points(tg)?;
    clear_pixels_style(tg);
    Ok(())
}

/// `LinesWithFill`, run on the points as given, before any filtering.
fn fills_of_original(tg: &mut Tg, original: &[Pt]) -> Option<Vec<Shape>> {
    let saved = std::mem::replace(&mut tg.pixels, get_deep_copy(original));
    let shapes = lines_with_fill_shapes(tg);
    tg.pixels = saved;
    shapes
}

/// The fill of a range fan drawn as its own graphic, under the outline, or
/// the autoshape fill of other graphics.
fn add_fan_fill(
    tg: &mut Tg,
    input: &Input<'_>,
    settings: &Settings,
    control: &[Pt],
    saved_fill_style: i32,
    shapes: &mut Vec<Shape>,
) -> Result<(), EngineError> {
    if !matches!(
        tg.line_type,
        tl::RANGE_FAN | tl::RANGE_FAN_SECTOR | tl::RADAR_SEARCH
    ) {
        get_autoshape_fill_shape(tg, shapes);
        return Ok(());
    }
    if tg.fill_color.is_none_or(|c| c.a < 2) {
        return Ok(());
    }
    let (mut fan, fan_control) = circular_range_fan_fill_tg(tg, settings, control)?;
    fan.fill_style = saved_fill_style;
    fan.set_symbol_id(input.symbol.as_str(), &|_| None)?;
    let mut fill = line_array::get_line_array(&mut fan, input, settings, &fan_control)?;
    fill.append(shapes);
    *shapes = fill;
    Ok(())
}

fn label(l: PlacedLabel) -> Label {
    Label {
        text: l.text,
        x: l.x,
        y: l.y,
        angle_deg: l.angle_deg,
        justify: match l.justify {
            modifier::JUSTIFY_CENTER => Justify::Center,
            modifier::JUSTIFY_RIGHT => Justify::Right,
            _ => Justify::Left,
        },
    }
}
