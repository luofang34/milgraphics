//! Port of mil-sym-java JavaLineArray/arraysupport.java (with the array
//! sizing of countsupport.java): the point and shape builders of the
//! non-channel line types. [`get_line_array2`] turns a graphic's control
//! points into the symbol's points and appends its shapes.
//!
//! The points live in a fixed-size array like upstream's, sized by
//! [`count::get_counters_double`], because builders index from its end and
//! read its length. Clip bounds and geographic converters are not used.

pub(crate) mod cases_arrows;
pub(crate) mod cases_control;
pub(crate) mod cases_mission;
pub(crate) mod cases_spikes;
pub(crate) mod cases_weather;
pub(crate) mod count;
pub(crate) mod count_sizes;
pub(crate) mod driver;
pub(crate) mod inside_outside;
pub(crate) mod points;
pub(crate) mod shape_path;
pub(crate) mod shapes;
pub(crate) mod work;

#[cfg(test)]
mod tests;

use crate::engine::base::{EngineError, Pt, Shape};
use crate::engine::lineutility::slope::calc_true_slope_double;
use crate::engine::metoc::MetocSupport;
use crate::engine::settings::Settings;
use crate::engine::tg::Tg;

/// Upstream `GetLineArray2` for no clip bounds and no converter: builds the
/// shapes of `tg.line_type` from the control `pixels` and returns the point
/// list (empty for most line types). Fails where upstream draws nothing.
pub(crate) fn get_line_array2(
    tg: &mut Tg,
    pixels: &[Pt],
    shapes: &mut Vec<Shape>,
    settings: &Settings,
) -> Result<Vec<Pt>, EngineError> {
    let count = count::get_counters_double(tg, pixels, settings)?;
    let size = usize::try_from(count).map_err(|_| EngineError::LineType(tg.line_type))?;
    if size == 0 {
        return Err(EngineError::LineType(tg.line_type));
    }
    let mut p = vec![Pt::default(); size];
    let mut save =
        i32::try_from(pixels.len()).map_err(|_| EngineError::Degenerate("too many points"))?;
    if let Some(len) = i32::try_from(p.len()).ok().filter(|len| save > *len) {
        save = len;
    }
    for (slot, src) in p
        .iter_mut()
        .zip(pixels.iter().take(usize::try_from(save).unwrap_or(0)))
    {
        *slot = Pt::styled(src.x, src.y, src.style);
    }
    driver::get_line_array2_double(tg, p, count, save, shapes, settings)
}

/// Upstream `SupplyRouteArrowSide`: which side of the segment the arrows of
/// a supply route go on (a direction code).
pub(crate) fn supply_route_arrow_side(pt0: Pt, pt1: Pt) -> i32 {
    let (vertical, m) = calc_true_slope_double(pt0, pt1);
    if pt0.x < pt1.x {
        return if m < 1.0 { 2 } else { 1 };
    }
    if pt0.x > pt1.x {
        return if m < 1.0 { 3 } else { 0 };
    }
    if vertical == 0 && pt0.y <= pt1.y {
        return 1;
    }
    0
}

/// The METOC builders' view of this module.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ArraySupport<'a> {
    /// Renderer settings the builders read.
    pub(crate) settings: &'a Settings,
}

impl MetocSupport for ArraySupport<'_> {
    fn line_array(&self, tg: &mut Tg, shapes: &mut Vec<Shape>) -> Result<(), EngineError> {
        let pixels = tg.pixels.clone();
        get_line_array2(tg, &pixels, shapes, self.settings).map(|_| ())
    }

    fn partitions(&self, _tg: &Tg) -> Result<Vec<crate::engine::metoc::Partition>, EngineError> {
        Err(EngineError::Degenerate("channels not ported"))
    }

    fn channel_points(
        &self,
        _line: &[f64],
        _out: &mut [f64],
        _channel_width: i32,
    ) -> Result<(), EngineError> {
        Err(EngineError::Degenerate("channels not ported"))
    }
}
