//! Port of mil-sym-java JavaTacticalRenderer/clsUtility.IsChange1Area and
//! isAutoshape, and JavaLineArray/CELineArray.CIsChannel: predicates on a
//! line type.

use crate::engine::tactical_lines::*;

/// Upstream `IsChange1Area`: areas whose size comes from the geodesic parameters AM, AN and X.
pub(crate) fn is_change1_area(line_type: i32) -> bool {
    matches!(
        line_type,
        LAUNCH_AREA
            | DEFENDED_AREA_CIRCULAR
            | SHIP_AOI_CIRCULAR
            | PBS_ELLIPSE
            | RECTANGULAR
            | CUED_ACQUISITION
            | PBS_RECTANGLE
            | PBS_SQUARE
            | CIRCULAR
            | PBS_CIRCLE
            | BDZ
            | BBS_POINT
            | FSA_CIRCULAR
            | NOTACK
            | FFA_CIRCULAR
            | NFA_CIRCULAR
            | RFA_CIRCULAR
            | ACA_CIRCULAR
            | PAA_CIRCULAR
            | ATI_CIRCULAR
            | CFFZ_CIRCULAR
            | SENSOR_CIRCULAR
            | CENSOR_CIRCULAR
            | DA_CIRCULAR
            | CFZ_CIRCULAR
            | ZOR_CIRCULAR
            | TBA_CIRCULAR
            | TVAR_CIRCULAR
            | KILLBOXBLUE_CIRCULAR
            | KILLBOXPURPLE_CIRCULAR
            | RANGE_FAN
            | RANGE_FAN_FILL
            | RANGE_FAN_SECTOR
            | RADAR_SEARCH
            | BS_RADARC
            | BS_CAKE
            | PAA_RECTANGULAR
            | RECTANGULAR_TARGET
            | FSA_RECTANGULAR
            | SHIP_AOI_RECTANGULAR
            | DEFENDED_AREA_RECTANGULAR
            | BS_ROUTE
            | BS_TRACK
            | FFA_RECTANGULAR
            | RFA_RECTANGULAR
            | NFA_RECTANGULAR
            | ACA_RECTANGULAR
            | ATI_RECTANGULAR
            | CFFZ_RECTANGULAR
            | SENSOR_RECTANGULAR
            | CENSOR_RECTANGULAR
            | DA_RECTANGULAR
            | CFZ_RECTANGULAR
            | ZOR_RECTANGULAR
            | TBA_RECTANGULAR
            | TVAR_RECTANGULAR
            | KILLBOXBLUE_RECTANGULAR
            | KILLBOXPURPLE_RECTANGULAR
            | BS_ORBIT
            | BS_POLYARC
    )
}

/// Upstream `CIsChannel`: true if the line type is a channel type.
pub(crate) fn is_channel(line_type: i32) -> bool {
    matches!(
        line_type,
        CATK | CATKBYFIRE
            | LC
            | AIRAOA
            | AAAAA
            | MAIN
            | MAIN_STRAIGHT
            | SPT
            | SPT_STRAIGHT
            | FRONTAL_ATTACK
            | TURNING_MOVEMENT
            | MOVEMENT_TO_CONTACT
            | UNSP
            | SFENCE
            | DFENCE
            | DOUBLEA
            | LWFENCE
            | HWFENCE
            | BBS_LINE
            | SINGLEC
            | DOUBLEC
            | TRIPLE
            | CHANNEL
            | CHANNEL_FLARED
            | CHANNEL_DASHED
    )
}

/// What the symbol catalog says about a symbol, as `isAutoshape` reads it
/// (upstream `MSInfo`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MsInfo {
    /// Upstream `DrawRules` value.
    pub(crate) draw_rule: i32,
    /// Minimum control-point count.
    pub(crate) min_points: i32,
    /// Maximum control-point count.
    pub(crate) max_points: i32,
}

const DRAW_RULE_AREA26: i32 = 126;
const DRAW_RULE_LINE26: i32 = 326;
const DRAW_RULE_LINE27: i32 = 327;
const DRAW_RULE_CORRIDOR1: i32 = 401;

/// Upstream `isAutoshape`: a symbol with a fixed number of anchor points.
/// `ms_info` is the catalog entry of the graphic's symbol, `None` when the
/// catalog has none.
pub(crate) fn is_autoshape(line_type: i32, ms_info: Option<MsInfo>) -> bool {
    if matches!(
        line_type,
        BBS_RECTANGLE | BS_RECTANGLE | BS_ELLIPSE | PBS_CIRCLE | BS_CROSS | BS_BBOX | BBS_POINT
    ) {
        return true;
    }
    let Some(info) = ms_info else {
        return false;
    };
    if is_change1_area(line_type) {
        return false;
    }
    // Direction of attack symbols have two points but accept more.
    if matches!(line_type, DIRATKAIR | DIRATKGND | DIRATKSPT | INFILTRATION) {
        return false;
    }
    match info.draw_rule {
        // Two ways to draw, but a fixed number of points.
        DRAW_RULE_LINE26 | DRAW_RULE_LINE27 => true,
        // The first and second half need the same number of points.
        DRAW_RULE_AREA26 => true,
        // Each point is an air control point or communications checkpoint.
        DRAW_RULE_CORRIDOR1 => true,
        _ => info.max_points == info.min_points,
    }
}
