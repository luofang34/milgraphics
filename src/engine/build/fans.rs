//! The range fan, radar and ellipse parts of
//! `createTGLightFromMilStdSymbol`: AM, AN and X lists become the strings
//! the range-fan code reads.

use super::Amps;
use super::text::join_doubles;
use crate::engine::base::{At, EngineError};
use crate::engine::java_text::double_to_string;
use crate::engine::tactical_lines::*;
use crate::engine::tg::Tg;

/// `RANGE_FAN_SECTOR`: AM, AN and X as lists, and LRMM (left, right, min,
/// max per sector). Upstream fails (and stops building) when no sector can
/// be formed; so does this.
pub(super) fn sector_fan(tg: &mut Tg, amps: &Amps) -> Result<(), EngineError> {
    if let Some(am) = &amps.am {
        tg.am = join_doubles(am);
    }
    if let Some(an) = &amps.an {
        tg.an = join_doubles(an);
    }
    if let Some(x) = &amps.x {
        tg.x = x.join(",");
    }
    let (Some(am), Some(an)) = (&amps.am, &amps.an) else {
        return Ok(());
    };
    let sectors = an.len() / 2;
    let mut groups = String::new();
    for j in 0..sectors {
        let left = an.at(2 * j)?;
        let right = an.at(2 * j + 1)?;
        if j + 1 == am.len() {
            break;
        }
        let min = am.at(j)?;
        let max = am.at(j + 1)?;
        groups.push_str(&join_doubles(&[left, right, min, max]));
        if j + 1 < sectors {
            groups.push(',');
        }
    }
    if groups.is_empty() {
        return Err(EngineError::Degenerate("no sector in the range fan"));
    }
    if groups.ends_with(',') {
        groups.pop();
    }
    tg.lrmm = groups;
    Ok(())
}

/// `RADAR_SEARCH`: at most two AM and two AN values, and the one sector in
/// LRMM. The comma after the second value appears when more were given.
pub(super) fn radar_search(tg: &mut Tg, amps: &Amps) -> Result<(), EngineError> {
    let limited = |values: &[f64]| {
        let mut text = String::new();
        for (j, v) in values.iter().take(2).enumerate() {
            text.push_str(&double_to_string(*v));
            if j + 1 < values.len() {
                text.push(',');
            }
        }
        text
    };
    if let Some(am) = &amps.am {
        tg.am = limited(am);
    }
    if let Some(an) = &amps.an {
        tg.an = limited(an);
    }
    if let (Some(am), Some(an)) = (&amps.am, &amps.an) {
        tg.lrmm = join_doubles(&[an.at(0)?, an.at(1)?, am.at(0)?, am.at(1)?]);
    }
    Ok(())
}

/// `LAUNCH_AREA` and the circular defended areas: major and minor axis in AM
/// and AM1, rotation in AN.
pub(super) fn ellipse(tg: &mut Tg, amps: &Amps) {
    if let Some([major, minor, ..]) = amps.am.as_deref() {
        tg.am = double_to_string(*major);
        tg.am1 = double_to_string(*minor);
    }
    if let Some([rotation, ..]) = amps.an.as_deref() {
        tg.an = double_to_string(*rotation);
    }
}

/// `RANGE_FAN`: up to three radii, each with its altitude label.
pub(super) fn circular_fan(tg: &mut Tg, amps: &Amps) {
    let (mut am_text, mut x_text) = (String::new(), String::new());
    if let Some(am) = &amps.am {
        for (j, radius) in am.iter().take(3).enumerate() {
            am_text.push_str(&double_to_string(*radius));
            if j + 1 < am.len() {
                am_text.push(',');
            }
            if let Some(label) = amps.x.as_ref().and_then(|x| x.get(j)) {
                x_text.push_str(label);
                if j + 1 < amps.x.as_ref().map_or(0, Vec::len) {
                    x_text.push(',');
                }
            }
        }
    }
    tg.am = am_text;
    tg.x = x_text;
}

/// `RECTANGULAR` and `CUED_ACQUISITION`: width, length and attitude.
pub(super) fn rectangle(tg: &mut Tg, amps: &Amps) {
    let an = amps
        .an
        .as_ref()
        .and_then(|a| a.first().copied())
        .unwrap_or(0.0);
    if let Some([width, length, ..]) = amps.am.as_deref() {
        tg.am = double_to_string(*width);
        tg.am1 = double_to_string(*length);
        tg.an = double_to_string(an);
    }
}

/// The circles and rectangles that take their size from the first AM value.
const FIRST_AM_TYPES: &[i32] = &[
    PAA_RECTANGULAR,
    RECTANGULAR_TARGET,
    FSA_RECTANGULAR,
    SHIP_AOI_RECTANGULAR,
    DEFENDED_AREA_RECTANGULAR,
    FFA_RECTANGULAR,
    ACA_RECTANGULAR,
    NFA_RECTANGULAR,
    RFA_RECTANGULAR,
    ATI_RECTANGULAR,
    CFFZ_RECTANGULAR,
    SENSOR_RECTANGULAR,
    CENSOR_RECTANGULAR,
    DA_RECTANGULAR,
    CFZ_RECTANGULAR,
    ZOR_RECTANGULAR,
    TBA_RECTANGULAR,
    TVAR_RECTANGULAR,
    CIRCULAR,
    BDZ,
    FSA_CIRCULAR,
    NOTACK,
    ACA_CIRCULAR,
    FFA_CIRCULAR,
    NFA_CIRCULAR,
    RFA_CIRCULAR,
    PAA_CIRCULAR,
    ATI_CIRCULAR,
    CFFZ_CIRCULAR,
    SENSOR_CIRCULAR,
    CENSOR_CIRCULAR,
    DA_CIRCULAR,
    CFZ_CIRCULAR,
    ZOR_CIRCULAR,
    TBA_CIRCULAR,
    TVAR_CIRCULAR,
    KILLBOXBLUE_CIRCULAR,
    KILLBOXPURPLE_CIRCULAR,
    KILLBOXBLUE_RECTANGULAR,
    KILLBOXPURPLE_RECTANGULAR,
];

/// `PAA_RECTANGULAR` and the other sized areas: width or radius in AM.
pub(super) fn first_am(tg: &mut Tg, line_type: i32, amps: &Amps) {
    if FIRST_AM_TYPES.contains(&line_type) {
        if let Some([first, ..]) = amps.am.as_deref() {
            tg.am = double_to_string(*first);
        }
    }
}
