//! Port of mil-sym-java JavaTacticalRenderer/clsUtility.isClosedPolygon and
//! LinesWithFill: which line types are closed areas and which allow a fill.

#[cfg(test)]
mod tests;

use crate::engine::tactical_lines::*;

/// Upstream `isClosedPolygon`: true if the line type is a closed area.
pub(crate) fn is_closed_polygon(line_type: i32) -> bool {
    is_closed_polygon_part_0(line_type)
        || is_closed_polygon_part_1(line_type)
        || is_closed_polygon_part_2(line_type)
}

fn is_closed_polygon_part_0(line_type: i32) -> bool {
    matches!(
        line_type,
        BBS_AREA
            | BS_BBOX
            | AT
            | DEPICT
            | DZ
            | MINED
            | FENCED
            | UXO
            | ROZ
            | AARROZ
            | UAROZ
            | WEZ
            | FEZ
            | JEZ
            | FAADZ
            | HIDACZ
            | MEZ
            | LOMEZ
            | HIMEZ
            | WFZ_REVD
            | WFZ
            | PNO
            | BATTLE
            | EA
            | EZ
            | LZ
            | PZ
            | GENERAL
            | JTAA
            | SAA
            | SGAA
            | BS_AREA
            | ASSAULT
            | ATKPOS
            | OBJ
            | AO
            | AIRHEAD
            | NAI
            | TAI
            | BASE_CAMP_REVD
            | BASE_CAMP
            | GUERILLA_BASE_REVD
            | GUERILLA_BASE
            | GENERIC_AREA
            | OBSFAREA
            | OBSAREA
            | ZONE
            | STRONG
            | DRCL
            | FSA
            | ACA
            | ASSY
            | BSA
            | NFA
            | RFA
            | FARP
            | AIRFIELD
            | LAA
            | BOMB
            | FFA
    )
}

fn is_closed_polygon_part_1(line_type: i32) -> bool {
    matches!(
        line_type,
        SMOKE
            | PAA
            | ENCIRCLE
            | DHA_REVD
            | DHA
            | KILL_ZONE
            | EPW
            | RHA
            | DSA
            | CSA
            | RSA
            | FORT_REVD
            | FORT
            | PEN
            | BIO
            | BIOT
            | NUC
            | RAD
            | RADT
            | CHEM
            | CHEMT
            | SERIES
            | ATI
            | TBA
            | TVAR
            | CFFZ
            | CENSOR
            | SENSOR
            | ZOR
            | DA
            | CFZ
            | KILLBOXBLUE
            | KILLBOXPURPLE
            | IFR
            | MVFR
            | TURBULENCE
            | ICING
            | NON_CONVECTIVE
            | CONVECTIVE
            | FROZEN
            | THUNDERSTORMS
            | FOG
            | SAND
            | FREEFORM
            | DEPTH_AREA
            | ISLAND
            | BEACH
            | WATER
            | FISH_TRAPS
            | SWEPT_AREA
            | OIL_RIG_FIELD
            | FOUL_GROUND
            | KELP
            | BEACH_SLOPE_MODERATE
            | BEACH_SLOPE_STEEP
            | ANCHORAGE_AREA
            | TRAINING_AREA
            | FORESHORE_AREA
            | DRYDOCK
            | LOADING_FACILITY_AREA
    )
}

fn is_closed_polygon_part_2(line_type: i32) -> bool {
    matches!(
        line_type,
        PERCHES
            | UNDERWATER_HAZARD
            | DISCOLORED_WATER
            | BEACH_SLOPE_FLAT
            | BEACH_SLOPE_GENTLE
            | MARITIME_AREA
            | OPERATOR_DEFINED
            | SUBMERGED_CRIB
            | VDR_LEVEL_12
            | VDR_LEVEL_23
            | VDR_LEVEL_34
            | VDR_LEVEL_45
            | VDR_LEVEL_56
            | VDR_LEVEL_67
            | VDR_LEVEL_78
            | VDR_LEVEL_89
            | VDR_LEVEL_910
            | SOLID_ROCK
            | CLAY
            | VERY_COARSE_SAND
            | COARSE_SAND
            | MEDIUM_SAND
            | FINE_SAND
            | VERY_FINE_SAND
            | VERY_FINE_SILT
            | FINE_SILT
            | MEDIUM_SILT
            | COARSE_SILT
            | BOULDERS
            | OYSTER_SHELLS
            | PEBBLES
            | SAND_AND_SHELLS
            | BOTTOM_SEDIMENTS_LAND
            | BOTTOM_SEDIMENTS_NO_DATA
            | BOTTOM_ROUGHNESS_SMOOTH
            | BOTTOM_ROUGHNESS_MODERATE
            | BOTTOM_ROUGHNESS_ROUGH
            | CLUTTER_LOW
            | CLUTTER_MEDIUM
            | CLUTTER_HIGH
            | IMPACT_BURIAL_0
            | IMPACT_BURIAL_10
            | IMPACT_BURIAL_20
            | IMPACT_BURIAL_75
            | IMPACT_BURIAL_100
            | BOTTOM_CATEGORY_A
            | BOTTOM_CATEGORY_B
            | BOTTOM_CATEGORY_C
            | BOTTOM_TYPE_A1
            | BOTTOM_TYPE_A2
            | BOTTOM_TYPE_A3
            | BOTTOM_TYPE_B1
            | BOTTOM_TYPE_B2
            | BOTTOM_TYPE_B3
            | BOTTOM_TYPE_C1
            | BOTTOM_TYPE_C2
            | BOTTOM_TYPE_C3
            | TGMF
    )
}

/// Upstream `LinesWithFill`: line types that allow a fill.
pub(crate) fn lines_with_fill(line_type: i32) -> bool {
    lines_with_fill_part_0(line_type) || lines_with_fill_part_1(line_type)
}

fn lines_with_fill_part_0(line_type: i32) -> bool {
    matches!(
        line_type,
        BS_LINE
            | PAA_RECTANGULAR
            | RECTANGULAR_TARGET
            | CFL
            | TRIP
            | DIRATKAIR
            | BOUNDARY
            | ISOLATE
            | CORDONKNOCK
            | CORDONSEARCH
            | DENY
            | OCCUPY
            | RETAIN
            | SECURE
            | CONTROL
            | LOCATE
            | AREA_DEFENSE
            | MOBILE_DEFENSE
            | FLOT
            | LC
            | PL
            | FEBA
            | LL
            | EWL
            | DIRATKGND
            | DIRATKSPT
            | INFILTRATION
            | FCL
            | HOLD
            | BRDGHD
            | HOLD_GE
            | BRDGHD_GE
            | LOA
            | LOD
            | LDLC
            | RELEASE
            | HOL
            | BHL
            | LINE
            | ABATIS
            | ATDITCH
            | ATWALL
            | SFENCE
            | DFENCE
            | UNSP
            | PLD
            | DOUBLEA
            | LWFENCE
            | HWFENCE
            | SINGLEC
            | DOUBLEC
            | TRIPLE
            | FORTL
            | LINTGT
            | LINTGTS
            | FSCL
            | BCL_REVD
            | BCL
            | ICL
            | IFF_OFF
    )
}

fn lines_with_fill_part_1(line_type: i32) -> bool {
    matches!(
        line_type,
        IFF_ON
            | GENERIC_LINE
            | NFL
            | MFP
            | RFL
            | CONVOY
            | HCONVOY
            | MSR
            | MSR_ONEWAY
            | MSR_TWOWAY
            | MSR_ALT
            | ASR
            | ASR_ONEWAY
            | ASR_TWOWAY
            | ASR_ALT
            | TRAFFIC_ROUTE
            | TRAFFIC_ROUTE_ONEWAY
            | TRAFFIC_ROUTE_ALT
    )
}
