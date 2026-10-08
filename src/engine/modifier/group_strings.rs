//! `buildAreaGroupString` and `buildAreaGroupDTGString` of Modifier2.java:
//! the multi-line text used when the settings group an area's modifiers.

use crate::engine::tactical_lines as tl;
use crate::engine::tg::Tg;

/// " -" after W when W1 follows on the next line.
fn dtg_lines(tg: &Tg) -> String {
    let mut text = String::new();
    if !tg.w.is_empty() {
        text.push_str(&format!("\n{} -", tg.w));
    }
    if !tg.w1.is_empty() {
        text.push_str(&format!("\n{}", tg.w1));
    }
    text
}

/// Upstream `buildAreaGroupDTGString`: W and W1 on two lines.
pub(super) fn build_area_group_dtg_string(tg: &Tg) -> String {
    let mut text = String::new();
    if !tg.w.is_empty() {
        text.push_str(&format!("{} -", tg.w));
    }
    if !tg.w1.is_empty() {
        text.push_str(&format!("\n{}", tg.w1));
    }
    text
}

/// Upstream `buildAreaGroupString`: the grouped text of an area, or an empty
/// string for a line type that is not grouped.
pub(super) fn build_area_group_string(tg: &Tg, label: &str) -> String {
    let name = &tg.t;
    match tg.line_type {
        tl::PAA | tl::PAA_CIRCULAR | tl::PAA_RECTANGULAR => {
            format!("{name}{}", dtg_lines(tg))
        }
        tl::ACA | tl::ACA_CIRCULAR | tl::ACA_RECTANGULAR => format!(
            "{label} {name}\n{}\nMIN ALT: {}\nMAX ALT: {}\nGRID: {}\nEFF: {} -\n{}",
            tg.t1,
            tg.x,
            tg.x1,
            tg.location(),
            tg.w,
            tg.w1
        ),
        tl::FFA
        | tl::NFA
        | tl::RFA
        | tl::FFA_RECTANGULAR
        | tl::NFA_RECTANGULAR
        | tl::RFA_RECTANGULAR
        | tl::FFA_CIRCULAR
        | tl::NFA_CIRCULAR
        | tl::RFA_CIRCULAR => format!("{label}\n{name}{}", dtg_lines(tg)),
        tl::FSA => format!("{label} {name}{}", dtg_lines(tg)),
        lt if label_and_name_group(lt) => format!("{label}\n{name}"),
        tl::WFZ_REVD | tl::OBSFAREA => {
            format!("{label}\n{name}\nTIME FROM: {}\nTIME TO: {}", tg.w, tg.w1)
        }
        tl::OBSAREA => format!("{name}\nTIME FROM: {}\nTIME TO: {}", tg.w, tg.w1),
        tl::WFZ
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
        | tl::HIMEZ => format!(
            "{label}\n{name}\nMIN ALT: {}\nMAX ALT: {}\nTIME FROM: {}\nTIME TO: {}",
            tg.x, tg.x1, tg.w, tg.w1
        ),
        tl::AC | tl::LLTR | tl::MRR | tl::SL | tl::TC | tl::SAAFR | tl::SC => format!(
            "NAME: {name}\nWIDTH: {}\nMIN ALT: {}\nMAX ALT: {}\nDTG Start: {}\nDTG End: {}\n\n{label} {name}\n\n\n\n\n\n\n",
            tg.am, tg.x, tg.x1, tg.w, tg.w1
        ),
        _ => String::new(),
    }
}

/// The zones whose group text is the label over the name.
fn label_and_name_group(line_type: i32) -> bool {
    matches!(
        line_type,
        tl::ATI
            | tl::CFFZ
            | tl::CFZ
            | tl::TBA
            | tl::TVAR
            | tl::ZOR
            | tl::DA
            | tl::SENSOR
            | tl::CENSOR
            | tl::KILLBOXBLUE
            | tl::KILLBOXPURPLE
            | tl::KILLBOXBLUE_RECTANGULAR
            | tl::KILLBOXPURPLE_RECTANGULAR
            | tl::FSA_RECTANGULAR
            | tl::ATI_RECTANGULAR
            | tl::CFFZ_RECTANGULAR
            | tl::SENSOR_RECTANGULAR
            | tl::CENSOR_RECTANGULAR
            | tl::DA_RECTANGULAR
            | tl::CFZ_RECTANGULAR
            | tl::ZOR_RECTANGULAR
            | tl::TBA_RECTANGULAR
            | tl::TVAR_RECTANGULAR
            | tl::FSA_CIRCULAR
            | tl::ATI_CIRCULAR
            | tl::CFFZ_CIRCULAR
            | tl::SENSOR_CIRCULAR
            | tl::CENSOR_CIRCULAR
            | tl::DA_CIRCULAR
            | tl::CFZ_CIRCULAR
            | tl::ZOR_CIRCULAR
            | tl::TBA_CIRCULAR
            | tl::TVAR_CIRCULAR
            | tl::KILLBOXBLUE_CIRCULAR
            | tl::KILLBOXPURPLE_CIRCULAR
    )
}
