//! Port of `countsupport.GetCountersDouble`: the number of points upstream
//! allocates for a line type. Builders index from the end of the array and
//! read its length, so the sizes are kept exactly. The channel line types
//! (triple, double, LC, ...) are sized in the channel code, not here.

use crate::engine::base::{At, EngineError, Pt};
use crate::engine::flot::FlotStyle;
use crate::engine::flot::anchorage::get_anchorage_count_double;
use crate::engine::flot::flot_line::get_flot_count_double;
use crate::engine::flot::flot_wf::get_flot_count2_double;
use crate::engine::flot::occluded::get_occluded_count_double;
use crate::engine::flot::ofy::get_ofy_count_double;
use crate::engine::flot::sf::get_sf_count_double;
use crate::engine::lineutility::slope::calc_distance_to_line_double;
use crate::engine::settings::Settings;
use crate::engine::tactical_lines as lt;
use crate::engine::tg::Tg;

use super::count_sizes as sz;
use super::work::scaled_size;

/// Upstream `GetCountersDouble` for no clip bounds: the size of the point
/// array for `tg.line_type` given the control points `pts`.
pub(crate) fn get_counters_double(
    tg: &Tg,
    pts: &[Pt],
    settings: &Settings,
) -> Result<i32, EngineError> {
    let vbl = i32::try_from(pts.len()).map_err(|_| EngineError::Degenerate("too many points"))?;
    if let Some(count) = fixed_count(tg.line_type, vbl) {
        return Ok(count);
    }
    match tg.line_type {
        lt::TRIPLE
        | lt::DOUBLEC
        | lt::SINGLEC
        | lt::HWFENCE
        | lt::LWFENCE
        | lt::UNSP
        | lt::DOUBLEA
        | lt::SFENCE
        | lt::DFENCE
        | lt::LC => Err(EngineError::LineType(tg.line_type)),
        _ => measured_count(tg, pts, vbl, settings),
    }
}

/// The line types whose size depends only on the number of control points.
fn fixed_count(line_type: i32, vbl: i32) -> Option<i32> {
    Some(match line_type {
        lt::BS_ELLIPSE => 37,
        lt::BS_CROSS => 4,
        lt::OVERHEAD_WIRE => vbl * 15,
        lt::TRAINING_AREA => vbl + 30,
        lt::FOLLA | lt::FOLSP | lt::SARA => 16,
        lt::ROADBLK | lt::FERRY | lt::EXPLOIT | lt::RAFT | lt::MFLANE => 8,
        lt::NAVIGATION
        | lt::IL
        | lt::PLANNED
        | lt::ESR1
        | lt::ESR2
        | lt::FORDSITE
        | lt::FOXHOLE
        | lt::DECEIVE
        | lt::BLOCK
        | lt::MNFLDBLK => 4,
        lt::AMBUSH => 53,
        lt::CLUSTER => 28,
        lt::CONTAIN => 40,
        lt::BYIMP => 18,
        lt::SPTBYFIRE => 16,
        lt::PAA_RECTANGULAR => 5,
        lt::RECTANGULAR_TARGET => 9,
        lt::PENETRATE => 7,
        lt::ASLTXING | lt::GAP | lt::BYPASS | lt::EASY | lt::BREACH | lt::CANALIZE => 12,
        lt::MNFLDDIS => 22,
        lt::WITHDRAW
        | lt::DISENGAGE
        | lt::WDRAWUP
        | lt::DELAY
        | lt::RETIRE
        | lt::FPOL
        | lt::RPOL
        | lt::ENVELOPMENT => 23,
        lt::PURSUIT => 25,
        lt::SEIZE | lt::CAPTURE | lt::EVACUATE => 37,
        lt::RIP | lt::DEMONSTRATE => 29,
        lt::MOBILE_DEFENSE => 44,
        lt::DIRATKSPT | lt::ABATIS => vbl + 3,
        lt::EXFILTRATION | lt::INFILTRATION => vbl + 10 + 3,
        lt::FPF | lt::LINTGT | lt::LINTGTS => vbl + 4,
        lt::CHANNEL | lt::CHANNEL_FLARED | lt::CHANNEL_DASHED | lt::BBS_LINE => 2 * vbl,
        lt::COVER | lt::SCREEN | lt::GUARD | lt::PDF | lt::ATKBYFIRE => 14,
        lt::ESCORT => 6,
        lt::DIRATKGND => vbl + 10,
        lt::DIRATKAIR => vbl + 9,
        lt::DISRUPT | lt::CLEAR => 20,
        lt::MSDZ => 300,
        lt::CONVOY | lt::HCONVOY => 10,
        lt::ISOLATE | lt::CORDONKNOCK | lt::CORDONSEARCH | lt::DENY => 50,
        lt::AREA_DEFENSE => 67,
        lt::OCCUPY | lt::CONTROL | lt::LOCATE => 32,
        lt::SECURE => 29,
        lt::RETAIN => 75,
        lt::TURN_REVD | lt::TURN => 29,
        lt::AIRFIELD => vbl + 5,
        lt::MSR_ALT | lt::ASR_ALT | lt::TRAFFIC_ROUTE_ALT => vbl * 9,
        lt::MSR_TWOWAY | lt::ASR_TWOWAY => vbl * 11,
        lt::MSR_ONEWAY | lt::ASR_ONEWAY | lt::TRAFFIC_ROUTE_ONEWAY => vbl * 6,
        lt::CATK | lt::MAIN | lt::MAIN_STRAIGHT | lt::AIRAOA | lt::SPT | lt::SPT_STRAIGHT => {
            2 * vbl + 8
        }
        lt::FRONTAL_ATTACK => 2 * vbl + 15,
        lt::TURNING_MOVEMENT => 2 * vbl + 14,
        lt::MOVEMENT_TO_CONTACT => 2 * vbl + 24,
        lt::CATKBYFIRE => 2 * vbl + 17,
        lt::AAAAA => 2 * vbl + 19,
        lt::LLTR | lt::SAAFR | lt::AC | lt::SC | lt::MRR | lt::SL | lt::TC => {
            6 * (vbl - 1) + 26 * vbl * 2
        }
        _ => return None,
    })
}

/// The line types whose size is computed from the control points'
/// geometry; any other type has one point per control point.
fn measured_count(tg: &Tg, pts: &[Pt], vbl: i32, settings: &Settings) -> Result<i32, EngineError> {
    let line_type = tg.line_type;
    let style = FlotStyle::new(line_type, f64::from(tg.line_thickness), tg.pattern_scale);
    let s = |size: f64| scaled_size(tg, size);
    Ok(match line_type {
        lt::REEF => sz::reef_count(pts, s(40.0), vbl)?,
        lt::RESTRICTED_AREA => sz::restricted_area_count(pts, s(15.0), vbl)?,
        lt::PIPE => sz::pipe_count(pts, s(20.0), vbl)?,
        lt::ANCHORAGE_AREA | lt::ANCHORAGE_LINE => get_anchorage_count_double(pts, s(20.0), vbl)?,
        lt::LRO => sz::x_points_count(pts, s(30.0), vbl)? + sz::lvo_count(pts, s(30.0), vbl)?,
        lt::LVO => sz::lvo_count(pts, s(20.0), vbl)?,
        lt::ICING => sz::icing_count(pts, s(15.0), vbl)?,
        lt::FLOT | lt::MVFR | lt::UNDERCAST => get_flot_count_double(pts, s(20.0), vbl)?,
        lt::ITD => sz::itd_qty(pts, s(15.0), vbl)? + vbl,
        lt::CONVERGENCE => sz::convergence_qty(pts, s(10.0), vbl)? + vbl,
        lt::TROUGH | lt::UPPER_TROUGH | lt::INSTABILITY | lt::SHEAR => {
            sz::squall_qty(pts, 6, s(30.0), vbl)?
        }
        lt::CABLE => sz::squall_qty(pts, 6, s(20.0), vbl)?,
        lt::SQUALL => sz::squall_qty(pts, 5, s(30.0), vbl)? + 2 * vbl,
        lt::USF | lt::SFG | lt::SFY | lt::SF => get_sf_count_double(pts, vbl)?,
        lt::OFY => get_ofy_count_double(pts, s(80.0), vbl)?,
        lt::UCF | lt::CF | lt::CFG | lt::CFY => sz::fortl_count(tg, pts, vbl)? + vbl,
        lt::WF | lt::UWF => get_flot_count2_double(&style, pts, vbl)? + vbl,
        lt::WFG | lt::WFY => get_flot_count2_double(&style, pts, vbl)?,
        lt::OCCLUDED | lt::UOF => get_occluded_count_double(pts, vbl)? + vbl,
        lt::FORDIF => fordif_count(tg, pts)?,
        lt::ATDITCH | lt::ATDITCHC | lt::ATDITCHM => sz::ditch_count(pts, vbl, line_type)?,
        lt::RIDGE | lt::ATWALL | lt::LINE | lt::FORTL => sz::fortl_count(tg, pts, vbl)?,
        lt::OBSAREA
        | lt::OBSFAREA
        | lt::STRONG
        | lt::ZONE
        | lt::ENCIRCLE
        | lt::FORT_REVD
        | lt::FORT => sz::zone_count(tg, pts, vbl, settings.visible.as_ref())?,
        lt::FIX | lt::MNFLDFIX | lt::BYDIF => dism_fix(pts, settings)?,
        _ => vbl,
    })
}

/// FORDIF's size: the ford width over half the spike, three points each.
fn fordif_count(tg: &Tg, pts: &[Pt]) -> Result<i32, EngineError> {
    let radius = calc_distance_to_line_double(pts.at(0)?, pts.at(1)?, pts.at(2)?);
    let spike = scaled_size(tg, 10.0);
    Ok(((radius / (spike / 2.0)) * 3.0) as i32 + 6)
}

/// The size of FIX and BYDIF: 20 points and three per jag, or 0 when
/// there are fewer than two points.
fn dism_fix(pts: &[Pt], settings: &Settings) -> Result<i32, EngineError> {
    if pts.len() > 1 {
        Ok(sz::dism_fix_count(
            pts.at(0)?,
            pts.at(1)?,
            settings.dpi_scale_factor(),
        ))
    } else {
        Ok(0)
    }
}
