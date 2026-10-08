//! The air corridor part of `createTGLightFromMilStdSymbol`: widths in
//! metres become pixel half-widths stored in each point's style.

use super::Amps;
use super::text::join_doubles;
use crate::engine::base::java_round;
use crate::engine::java_text::{double_to_string, parse_double};
use crate::engine::tactical_lines::*;
use crate::engine::tg::Tg;
use crate::engine::tg_utility::segment_colors::java_split;

/// The air corridor line types that carry widths.
pub(super) fn is_corridor(line_type: i32) -> bool {
    matches!(line_type, SC | MRR | SL | TC | LLTR | AC | SAAFR)
}

/// The width unit label upstream uses by default (`DistanceUnit.METERS`).
const DISTANCE_UNIT_LABEL: &str = "M";

/// Upstream's corridor branch. Each point's style becomes the pixel
/// half-width of the segment starting there (the widest width, for points
/// beyond the list); AM becomes the widest width in metres, rounded to a
/// tenth, with the unit. `meters_per_pixel` replaces upstream's 10 km probe.
pub(super) fn corridor_widths(tg: &mut Tg, amps: &Amps, meters_per_pixel: f64) {
    let pixels_per_meter = 1.0 / meters_per_pixel;
    if let Some(am) = &amps.am {
        tg.am = join_doubles(am);
    }
    let mut max_width_meters = 0.0;
    if !tg.am.is_empty() {
        // Text produced above, so every entry parses.
        let radii: Vec<f64> = java_split(&tg.am, ',')
            .into_iter()
            .filter_map(|t| parse_double(t).ok())
            .collect();
        let max_width = radii.iter().copied().fold(0.0, f64::max);
        max_width_meters = max_width;
        let widest = (max_width * pixels_per_meter / 2.0) as i32;
        for (j, p) in tg.pixels.iter_mut().enumerate() {
            p.style = match radii.get(j) {
                Some(r) if !r.is_nan() => (r * pixels_per_meter / 2.0) as i32,
                _ => widest,
            };
        }
    }
    let tenths = java_round(max_width_meters * 10.0);
    let truncated = f64::from(tenths as i32) / 10.0;
    tg.am = format!("{} {DISTANCE_UNIT_LABEL}", double_to_string(truncated));
}
