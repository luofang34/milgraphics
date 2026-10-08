//! The line types each label phase handles, copied from the entry switches
//! of `AddModifiersGeo` and `AddModifiers2` in Modifier2.java.

use crate::engine::tactical_lines as tl;

/// Types `AddModifiersGeo` labels; every other type returns early.
pub(super) fn handled_before_geometry(line_type: i32) -> bool {
    handled_before_geometry_0(line_type)
        || handled_before_geometry_1(line_type)
        || handled_before_geometry_2(line_type)
        || handled_before_geometry_3(line_type)
}

fn handled_before_geometry_0(line_type: i32) -> bool {
    matches!(
        line_type,
        tl::SERIES
            | tl::STRIKWARN
            | tl::MSR
            | tl::MSR_ONEWAY
            | tl::MSR_TWOWAY
            | tl::MSR_ALT
            | tl::ASR
            | tl::ASR_ONEWAY
            | tl::ASR_TWOWAY
            | tl::ASR_ALT
            | tl::TRAFFIC_ROUTE
            | tl::TRAFFIC_ROUTE_ONEWAY
            | tl::TRAFFIC_ROUTE_ALT
            | tl::DHA_REVD
            | tl::DHA
            | tl::KILL_ZONE
            | tl::EPW
            | tl::UXO
            | tl::FARP
            | tl::BSA
            | tl::DSA
            | tl::CSA
            | tl::RSA
            | tl::THUNDERSTORMS
            | tl::ICING
            | tl::FREEFORM
            | tl::RHA
            | tl::LINTGT
            | tl::LINTGTS
            | tl::FPF
            | tl::GAP
            | tl::DEPICT
            | tl::AIRHEAD
            | tl::FSA
            | tl::DIRATKAIR
            | tl::OBJ
            | tl::AO
            | tl::ACA
            | tl::FFA
            | tl::PAA
            | tl::NFA
            | tl::RFA
            | tl::ATI
            | tl::CFFZ
            | tl::CFZ
            | tl::TBA
            | tl::TVAR
            | tl::KILLBOXBLUE
            | tl::KILLBOXPURPLE
            | tl::ZOR
            | tl::DA
            | tl::SENSOR
            | tl::CENSOR
            | tl::SMOKE
            | tl::BATTLE
    )
}

fn handled_before_geometry_1(line_type: i32) -> bool {
    matches!(
        line_type,
        tl::PNO
            | tl::PDF
            | tl::NAI
            | tl::TAI
            | tl::BASE_CAMP_REVD
            | tl::BASE_CAMP
            | tl::GUERILLA_BASE_REVD
            | tl::GUERILLA_BASE
            | tl::GENERIC_AREA
            | tl::ATKPOS
            | tl::ASSAULT
            | tl::WFZ_REVD
            | tl::WFZ
            | tl::OBSFAREA
            | tl::OBSAREA
            | tl::ROZ
            | tl::AARROZ
            | tl::UAROZ
            | tl::WEZ
            | tl::FEZ
            | tl::JEZ
            | tl::FAADZ
            | tl::HIDACZ
            | tl::MEZ
            | tl::LOMEZ
            | tl::HIMEZ
            | tl::SAAFR
            | tl::AC
            | tl::MRR
            | tl::SL
            | tl::TC
            | tl::SC
            | tl::LLTR
            | tl::AIRFIELD
            | tl::GENERAL
            | tl::JTAA
            | tl::SAA
            | tl::SGAA
            | tl::FORT_REVD
            | tl::FORT
            | tl::ENCIRCLE
            | tl::ASSY
            | tl::EA
            | tl::DZ
            | tl::EZ
            | tl::LZ
            | tl::PZ
            | tl::LAA
            | tl::BOUNDARY
            | tl::MINED
            | tl::FENCED
            | tl::PL
            | tl::DECISION_LINE
            | tl::FEBA
            | tl::FCL
    )
}

fn handled_before_geometry_2(line_type: i32) -> bool {
    matches!(
        line_type,
        tl::HOLD
            | tl::BRDGHD
            | tl::HOLD_GE
            | tl::BRDGHD_GE
            | tl::LOA
            | tl::LOD
            | tl::LL
            | tl::EWL
            | tl::RELEASE
            | tl::HOL
            | tl::BHL
            | tl::LDLC
            | tl::PLD
            | tl::NFL
            | tl::MFP
            | tl::FSCL
            | tl::BCL_REVD
            | tl::BCL
            | tl::ICL
            | tl::IFF_OFF
            | tl::IFF_ON
            | tl::GENERIC_LINE
            | tl::CFL
            | tl::TRIP
            | tl::RFL
            | tl::FLOT
            | tl::LC
            | tl::CATK
            | tl::CATKBYFIRE
            | tl::IL
            | tl::DRCL
            | tl::RETIRE
            | tl::PURSUIT
            | tl::FPOL
            | tl::RPOL
            | tl::WITHDRAW
            | tl::DISENGAGE
            | tl::WDRAWUP
            | tl::BEARING
            | tl::BEARING_J
            | tl::BEARING_RDF
            | tl::ELECTRO
            | tl::BEARING_EW
            | tl::ACOUSTIC
            | tl::ACOUSTIC_AMB
            | tl::TORPEDO
            | tl::OPTICAL
            | tl::RIP
            | tl::DEMONSTRATE
            | tl::BOMB
            | tl::ZONE
            | tl::AT
            | tl::STRONG
            | tl::MSDZ
            | tl::SCREEN
    )
}

fn handled_before_geometry_3(line_type: i32) -> bool {
    matches!(
        line_type,
        tl::COVER
            | tl::GUARD
            | tl::DELAY
            | tl::TGMF
            | tl::BIO
            | tl::BIOT
            | tl::CHEM
            | tl::CHEMT
            | tl::NUC
            | tl::RAD
            | tl::RADT
            | tl::MINE_LINE
            | tl::ANCHORAGE_LINE
            | tl::ANCHORAGE_AREA
            | tl::SPT
            | tl::FRONTAL_ATTACK
            | tl::TURNING_MOVEMENT
            | tl::MOVEMENT_TO_CONTACT
            | tl::AIRAOA
            | tl::AAAAA
            | tl::MAIN
            | tl::DIRATKSPT
            | tl::DIRATKGND
            | tl::LAUNCH_AREA
            | tl::DEFENDED_AREA_CIRCULAR
            | tl::RECTANGULAR
            | tl::CIRCULAR
            | tl::RECTANGULAR_TARGET
            | tl::LINE
            | tl::ASLTXING
            | tl::BS_LINE
            | tl::BS_AREA
            | tl::BBS_LINE
            | tl::BBS_AREA
            | tl::PBS_CIRCLE
            | tl::PBS_ELLIPSE
            | tl::PBS_RECTANGLE
            | tl::BBS_POINT
    )
}

/// Types `AddModifiers2` labels; every other type returns early.
pub(super) fn handled_after_geometry(line_type: i32) -> bool {
    handled_after_geometry_0(line_type) || handled_after_geometry_1(line_type)
}

fn handled_after_geometry_0(line_type: i32) -> bool {
    matches!(
        line_type,
        tl::BS_RECTANGLE
            | tl::BBS_RECTANGLE
            | tl::CONVOY
            | tl::HCONVOY
            | tl::BREACH
            | tl::BYPASS
            | tl::CANALIZE
            | tl::PENETRATE
            | tl::CLEAR
            | tl::DISRUPT
            | tl::FIX
            | tl::ISOLATE
            | tl::OCCUPY
            | tl::RETAIN
            | tl::SECURE
            | tl::CONTROL
            | tl::LOCATE
            | tl::AREA_DEFENSE
            | tl::CONTAIN
            | tl::SEIZE
            | tl::CAPTURE
            | tl::EVACUATE
            | tl::TURN
            | tl::CORDONKNOCK
            | tl::CORDONSEARCH
            | tl::DENY
            | tl::ESCORT
            | tl::EXFILTRATION
            | tl::INFILTRATION
            | tl::FOLLA
            | tl::FOLSP
            | tl::ACA_RECTANGULAR
            | tl::ACA_CIRCULAR
            | tl::RECTANGULAR
            | tl::CUED_ACQUISITION
            | tl::CIRCULAR
            | tl::BDZ
            | tl::BBS_POINT
            | tl::FSA_CIRCULAR
            | tl::NOTACK
            | tl::ATI_CIRCULAR
            | tl::CFFZ_CIRCULAR
            | tl::SENSOR_CIRCULAR
            | tl::CENSOR_CIRCULAR
            | tl::DA_CIRCULAR
            | tl::CFZ_CIRCULAR
            | tl::ZOR_CIRCULAR
            | tl::TBA_CIRCULAR
            | tl::TVAR_CIRCULAR
            | tl::FFA_CIRCULAR
            | tl::NFA_CIRCULAR
            | tl::RFA_CIRCULAR
            | tl::KILLBOXBLUE_CIRCULAR
            | tl::KILLBOXPURPLE_CIRCULAR
            | tl::BLOCK
    )
}

fn handled_after_geometry_1(line_type: i32) -> bool {
    matches!(
        line_type,
        tl::FFA_RECTANGULAR
            | tl::NFA_RECTANGULAR
            | tl::RFA_RECTANGULAR
            | tl::KILLBOXBLUE_RECTANGULAR
            | tl::KILLBOXPURPLE_RECTANGULAR
            | tl::FSA_RECTANGULAR
            | tl::SHIP_AOI_RECTANGULAR
            | tl::DEFENDED_AREA_RECTANGULAR
            | tl::ATI_RECTANGULAR
            | tl::CFFZ_RECTANGULAR
            | tl::SENSOR_RECTANGULAR
            | tl::CENSOR_RECTANGULAR
            | tl::DA_RECTANGULAR
            | tl::CFZ_RECTANGULAR
            | tl::ZOR_RECTANGULAR
            | tl::TBA_RECTANGULAR
            | tl::TVAR_RECTANGULAR
            | tl::PAA
            | tl::PAA_RECTANGULAR
            | tl::RECTANGULAR_TARGET
            | tl::PAA_CIRCULAR
            | tl::RANGE_FAN
            | tl::RANGE_FAN_SECTOR
            | tl::RADAR_SEARCH
            | tl::SHIP_AOI_CIRCULAR
            | tl::MFLANE
            | tl::ENVELOPMENT
            | tl::MOBILE_DEFENSE
    )
}
