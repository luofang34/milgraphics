//! Port of `GetCenterLabel` from mil-sym-java JavaTacticalRenderer/Modifier2.java:
//! the fixed text a graphic carries in its centre or at its ends.

use crate::engine::tactical_lines as tl;
use crate::engine::tg::Tg;

/// `SymbolID.Version_2525E`.
pub(crate) const VERSION_2525E: i32 = 13;
/// `SymbolID.Version_2525Ech1`.
pub(crate) const VERSION_2525E_CH1: i32 = 15;
/// `SymbolID.Version_APP6Ech2`.
pub(crate) const VERSION_APP6E_2: i32 = 16;

/// Upstream `SymbolID.getVersion`: the two-digit version of a symbol code of
/// at least 20 characters, 11 otherwise. `None` where upstream's parse would
/// throw.
pub(crate) fn symbol_version(symbol_id: &str) -> Option<i32> {
    if symbol_id.len() < 20 {
        return Some(11);
    }
    symbol_id.get(0..2)?.parse::<i32>().ok()
}

/// True when the graphic's symbol code is of version 16, whose templates
/// differ from upstream's labels for some graphics.
pub(crate) fn is_app6e_2(tg: &Tg) -> bool {
    symbol_version(&tg.symbol_id) == Some(VERSION_APP6E_2)
}

/// Upstream `GetCenterLabel`: the generic label per MIL-STD-2525 for the
/// graphic's line type, or an empty string. Version 16 templates name the
/// area of operations and the artillery target intelligence zones
/// differently.
pub(crate) fn get_center_label(tg: &Tg) -> String {
    let version = symbol_version(&tg.symbol_id);
    if let Some(label) = app6e_label(tg) {
        return label.to_owned();
    }
    let lt = tg.line_type;
    let versioned = match lt {
        tl::AO if version == Some(VERSION_APP6E_2) => Some("AOO"),
        tl::ATI if version == Some(VERSION_APP6E_2) => Some("ATI"),
        tl::ATI_CIRCULAR | tl::ATI_RECTANGULAR if version == Some(VERSION_APP6E_2) => {
            Some("ATI ZONE")
        }
        tl::BRDGHD | tl::BRDGHD_GE => Some(match version {
            Some(v) if v >= VERSION_2525E => "BL",
            Some(_) => "B",
            None => "",
        }),
        tl::ATI | tl::ATI_CIRCULAR | tl::ATI_RECTANGULAR => Some(match version {
            Some(v) if v >= VERSION_2525E_CH1 => "ATIZ",
            Some(_) => "ATI ZONE",
            None => "",
        }),
        _ => None,
    };
    versioned
        .or_else(|| fixed_0(lt))
        .or_else(|| fixed_1(lt))
        .or_else(|| fixed_2(lt))
        .unwrap_or("")
        .to_owned()
}

/// The label of a version 16 code that upstream draws with another code's
/// line type (`line_type::control_measures`).
fn app6e_label(tg: &Tg) -> Option<&'static str> {
    if !is_app6e_2(tg) {
        return None;
    }
    let entity: u32 = tg.symbol_id.get(10..16)?.parse().ok()?;
    Some(match entity {
        120_800 => "BA",
        370_100 => "HT",
        242_600 => "ZF",
        242_400 => "AMA",
        242_500 => "ARA",
        344_600 => "R",
        _ => return None,
    })
}

/// Labels that do not depend on the symbol version (part 1).
fn fixed_0(line_type: i32) -> Option<&'static str> {
    Some(match line_type {
        tl::SHIP_AOI_RECTANGULAR | tl::SHIP_AOI_CIRCULAR => "AOI",
        tl::DEFENDED_AREA_RECTANGULAR | tl::DEFENDED_AREA_CIRCULAR => "DA",
        tl::NOTACK => "N",
        tl::LAUNCH_AREA => "LA",
        tl::SL => "SL",
        tl::TC => "TC",
        tl::AARROZ => "AARROZ",
        tl::UAROZ => "UAROZ",
        tl::WEZ => "WEZ",
        tl::FEZ => "FEZ",
        tl::JEZ => "JEZ",
        tl::IFF_OFF => "IFF OFF",
        tl::IFF_ON => "IFF ON",
        tl::BCL_REVD | tl::BCL => "BCL",
        tl::ICL => "ICL",
        tl::FEBA => "FEBA",
        tl::BDZ => "BDZ",
        tl::JTAA => "JTAA",
        tl::SAA => "SAA",
        tl::SGAA => "SGAA",
        tl::ASSAULT => "ASLT",
        tl::SAAFR => "SAAFR",
        tl::AC => "AC",
        tl::SECURE | tl::SEIZE => "S",
        tl::TURN => "T",
        tl::EVACUATE => "E",
        tl::RETAIN => "R",
        tl::PENETRATE => "P",
        tl::OCCUPY => "O",
        tl::ISOLATE => "I",
        tl::AREA_DEFENSE => "AD",
        tl::FIX => "F",
        tl::DISRUPT => "D",
        tl::CAPTURE | tl::CANALIZE | tl::CLEAR | tl::CONTROL => "C",
        tl::BREACH | tl::BYPASS => "B",
        tl::CORDONKNOCK => "C/K",
        tl::CORDONSEARCH => "C/S",
        tl::UXO => "UXO",
        tl::RETIRE => "R",
        tl::PURSUIT => "P",
        tl::ENVELOPMENT => "E",
        tl::FPOL => "P(F)",
        tl::RPOL => "P(R)",
        tl::HOLD | tl::HOLD_GE => "HL",
        tl::PL => "PL",
        tl::LL => "LL",
        tl::LOCATE => "LOC",
        tl::EWL => "EWL",
        tl::SCREEN => "S",
        tl::COVER => "C",
        tl::GUARD => "G",
        tl::RIP => "RIP",
        tl::MOBILE_DEFENSE => "MD",
        tl::DEMONSTRATE => "DEM",
        _ => return None,
    })
}

/// Labels that do not depend on the symbol version (part 2).
fn fixed_1(line_type: i32) -> Option<&'static str> {
    Some(match line_type {
        tl::WITHDRAW => "W",
        tl::DISENGAGE => "DIS",
        tl::WDRAWUP => "WP",
        tl::CATK | tl::CATKBYFIRE => "CATK",
        tl::FLOT => "FLOT",
        tl::LC => "LC",
        tl::ASSY => "AA",
        tl::EA => "EA",
        tl::DZ => "DZ",
        tl::EZ => "EZ",
        tl::LZ => "LZ",
        tl::LAA => "LAA",
        tl::PZ => "PZ",
        tl::MRR => "MRR",
        tl::SC => "SC",
        tl::LLTR => "LLTR",
        tl::ROZ => "ROZ",
        tl::FAADZ => "SHORADEZ",
        tl::HIDACZ => "HIDACZ",
        tl::MEZ => "MEZ",
        tl::LOMEZ => "LOMEZ",
        tl::HIMEZ => "HIMEZ",
        tl::WFZ_REVD | tl::WFZ => "WFZ",
        tl::MINED | tl::FENCED => "M",
        tl::PNO => "(P)",
        tl::OBJ => "OBJ",
        tl::NAI => "NAI",
        tl::TAI => "TAI",
        tl::BASE_CAMP_REVD | tl::BASE_CAMP => "BC",
        tl::GUERILLA_BASE_REVD | tl::GUERILLA_BASE => "GB",
        tl::LINTGTS => "SMOKE",
        tl::FPF => "FPF",
        tl::ATKPOS => "ATK",
        tl::FCL => "FCL",
        tl::LOA => "LOA",
        tl::LOD => "LD",
        tl::PLD => "PLD",
        tl::DELAY | tl::DENY => "D",
        tl::RELEASE => "RL",
        tl::HOL => "HOL",
        tl::BHL => "BHL",
        tl::SMOKE => "SMOKE",
        tl::NFL => "NFL",
        tl::MFP => "MFP",
        tl::FSCL => "FSCL",
        tl::CFL => "CFL",
        tl::RFL => "RFL",
        tl::AO => "AO",
        tl::BOMB => "BOMB",
        tl::TGMF => "TGMF",
        tl::FSA => "FSA",
        tl::FSA_CIRCULAR | tl::FSA_RECTANGULAR => "FSA",
        tl::ACA | tl::ACA_CIRCULAR | tl::ACA_RECTANGULAR => "ACA",
        tl::FFA | tl::FFA_CIRCULAR | tl::FFA_RECTANGULAR => "FFA",
        tl::NFA | tl::NFA_CIRCULAR | tl::NFA_RECTANGULAR => "NFA",
        _ => return None,
    })
}

/// Labels that do not depend on the symbol version (part 3).
fn fixed_2(line_type: i32) -> Option<&'static str> {
    Some(match line_type {
        tl::RFA | tl::RFA_CIRCULAR | tl::RFA_RECTANGULAR => "RFA",
        tl::PAA | tl::PAA_CIRCULAR | tl::PAA_RECTANGULAR => "PAA",
        tl::CFFZ | tl::CFFZ_CIRCULAR | tl::CFFZ_RECTANGULAR => "CFF ZONE",
        tl::CFZ | tl::CFZ_CIRCULAR | tl::CFZ_RECTANGULAR => "CF ZONE",
        tl::SENSOR | tl::SENSOR_CIRCULAR | tl::SENSOR_RECTANGULAR => "SENSOR ZONE",
        tl::CENSOR | tl::CENSOR_CIRCULAR | tl::CENSOR_RECTANGULAR => "CENSOR ZONE",
        tl::DA | tl::DA_CIRCULAR | tl::DA_RECTANGULAR => "DA",
        tl::ZOR | tl::ZOR_CIRCULAR | tl::ZOR_RECTANGULAR => "ZOR",
        tl::TBA | tl::TBA_CIRCULAR | tl::TBA_RECTANGULAR => "TBA",
        tl::TVAR | tl::TVAR_CIRCULAR | tl::TVAR_RECTANGULAR => "TVAR",
        tl::KILLBOXBLUE | tl::KILLBOXBLUE_CIRCULAR | tl::KILLBOXBLUE_RECTANGULAR => "BKB",
        tl::KILLBOXPURPLE | tl::KILLBOXPURPLE_CIRCULAR | tl::KILLBOXPURPLE_RECTANGULAR => "PKB",
        tl::MSR | tl::MSR_ONEWAY | tl::MSR_TWOWAY | tl::MSR_ALT => "MSR",
        tl::ASR | tl::ASR_ONEWAY | tl::ASR_TWOWAY | tl::ASR_ALT => "ASR",
        tl::TRAFFIC_ROUTE | tl::TRAFFIC_ROUTE_ONEWAY | tl::TRAFFIC_ROUTE_ALT => "ROUTE",
        tl::LDLC => "LD/LC",
        tl::AIRHEAD => "AIRHEAD LINE",
        tl::BLOCK | tl::BEARING => "B",
        tl::BEARING_J => "J",
        tl::BEARING_RDF => "RDF",
        tl::ELECTRO | tl::ESCORT => "E",
        tl::BEARING_EW => "EW",
        tl::ACOUSTIC | tl::ACOUSTIC_AMB => "A",
        tl::TORPEDO => "T",
        tl::OPTICAL => "O",
        tl::DHA => "DHA",
        tl::KILL_ZONE => "KILL ZONE",
        tl::FARP => "FARP",
        tl::BSA => "BSA",
        tl::DSA => "DSA",
        tl::CSA => "CSA",
        tl::RSA => "RSA",
        tl::CONTAIN => "C",
        tl::OBSFAREA => "FREE",
        tl::TRIP => "t",
        tl::EXFILTRATION => "EX",
        tl::INFILTRATION => "IN",
        _ => return None,
    })
}
