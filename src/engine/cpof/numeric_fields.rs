//! Port of `clsUtilityCPOF.GetNumericFields`: the numeric amplifiers of the
//! change 1 areas, read from the graphic's AM, AM1 and AN strings.

use super::groups::{CIRCLES, ELLIPSES, SEGMENT_RECTANGLES};
use super::planar;
use crate::engine::base::Pt;
use crate::engine::java_text::{is_number, parse_double};
use crate::engine::tactical_lines::*;
use crate::engine::tg::Tg;
use crate::engine::tg_utility::segment_colors::java_split;

/// Radius, width, length (metres) and attitude (degrees) of an area.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct NumericFields {
    pub(super) radius: f64,
    pub(super) width: f64,
    pub(super) length: f64,
    /// Two values only for `BS_POLYARC` (start and end azimuth).
    pub(super) attitude: [f64; 2],
}

fn number(text: &str) -> Option<f64> {
    if is_number(text) {
        parse_double(text).ok()
    } else {
        None
    }
}

/// Upstream `GetNumericFields`. `control` are the control points (upstream's
/// `LatLongs`). A value that fails to parse leaves the field 0, as the
/// upstream exception handler leaves the remaining fields untouched.
pub(super) fn get_numeric_fields(tg: &Tg, line_type: i32, control: &[Pt]) -> NumericFields {
    let mut f = NumericFields::default();
    if line_type == RANGE_FAN_FILL {
        return f;
    }
    let am = number(&tg.am);
    let am1 = number(&tg.am1);
    let an = number(&tg.an);
    if CIRCLES.contains(&line_type) {
        f.radius = am.unwrap_or(0.0);
    } else if ELLIPSES.contains(&line_type) {
        f.length = am1.unwrap_or(0.0);
        f.width = am.unwrap_or(0.0);
        f.attitude[0] = an.unwrap_or(0.0);
    } else if SEGMENT_RECTANGLES.contains(&line_type)
        || line_type == RECTANGULAR_TARGET
        || line_type == BS_ORBIT
    {
        if let (Some(p0), Some(p1)) = (control.first(), control.get(1)) {
            f.attitude[0] = planar::azimuth(*p0, *p1);
        }
        f.width = am.unwrap_or(0.0);
    } else {
        other_fields(&mut f, tg, line_type, (am, am1, an));
    }
    f
}

/// The rectangles with AM1/AN and the poly-arc.
fn other_fields(
    f: &mut NumericFields,
    tg: &Tg,
    line_type: i32,
    (am, am1, an): (Option<f64>, Option<f64>, Option<f64>),
) {
    match line_type {
        RECTANGULAR => {
            f.length = am1.unwrap_or(0.0);
            f.width = am.unwrap_or(0.0);
            // The attitude is given in mils.
            f.attitude[0] = an.map_or(0.0, |v| v * (360.0 / 6400.0));
        }
        PBS_RECTANGLE | PBS_SQUARE => {
            f.length = am1.unwrap_or(0.0);
            f.width = am.unwrap_or(0.0);
            f.attitude[0] = an.unwrap_or(0.0);
        }
        CUED_ACQUISITION => {
            f.length = am.unwrap_or(0.0);
            f.width = am1.unwrap_or(0.0);
            // 0 degrees points north instead of east.
            f.attitude[0] = an.map_or(0.0, |v| v + 270.0);
        }
        BS_POLYARC => {
            f.length = am.unwrap_or(0.0);
            let parts = java_split(&tg.an, ',');
            if let Some(a0) = parts.first().and_then(|t| parse_double(t).ok()) {
                f.attitude[0] = a0;
                if let Some(a1) = parts.get(1).and_then(|t| parse_double(t).ok()) {
                    f.attitude[1] = a1;
                }
            }
        }
        _ => {}
    }
}
