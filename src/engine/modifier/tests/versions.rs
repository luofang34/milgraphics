//! Labels that follow the symbol's standard version: version 16 templates
//! and the counterattacks of version 15, beside the earlier versions they
//! must leave unchanged.

use super::super::center_label::get_center_label;
use super::super::geo::add_modifiers_geo;
use super::{graphic, width};
use crate::engine::settings::Settings;
use crate::engine::tactical_lines as tl;
use crate::engine::tg::Tg;

fn versioned(line_type: i32, version: &str, points: &[(f64, f64)]) -> Tg {
    let mut tg = graphic(line_type, points);
    tg.symbol_id = version.to_owned() + &"0".repeat(18);
    tg
}

const AREA: &[(f64, f64)] = &[(0.0, 0.0), (200.0, 0.0), (200.0, 100.0), (0.0, 100.0)];
const LINE: &[(f64, f64)] = &[(0.0, 0.0), (600.0, 0.0)];
const AXIS: &[(f64, f64)] = &[(300.0, 0.0), (0.0, 0.0), (250.0, 30.0)];

fn texts(tg: &mut Tg) -> Vec<String> {
    add_modifiers_geo(tg, &Settings::default(), &width).unwrap();
    tg.modifiers.iter().map(|m| m.text.clone()).collect()
}

#[test]
fn center_labels_of_version_16() {
    for (line_type, v11, v15, v16) in [
        (tl::AO, "AO", "AO", "AOO"),
        (tl::ATI, "ATI ZONE", "ATIZ", "ATI"),
        (tl::ATI_RECTANGULAR, "ATI ZONE", "ATIZ", "ATI ZONE"),
        (tl::ATI_CIRCULAR, "ATI ZONE", "ATIZ", "ATI ZONE"),
    ] {
        for (version, label) in [("11", v11), ("15", v15), ("16", v16)] {
            let tg = versioned(line_type, version, AREA);
            assert_eq!(get_center_label(&tg), label, "{line_type} {version}");
        }
    }
}

#[test]
fn fire_support_areas_show_t2_and_as_in_version_16() {
    let mut tg = versioned(tl::FFA, "16", AREA);
    (tg.t, tg.t2, tg.as_) = ("T".into(), "2AD".into(), "DEU".into());
    (tg.w, tg.w1) = ("W".into(), "W1".into());
    assert_eq!(texts(&mut tg), ["FFA", "2AD (DEU)", "W - ", "W1"]);
    let mut tg = versioned(tl::FFA, "15", AREA);
    (tg.t, tg.t2, tg.as_) = ("T".into(), "2AD".into(), "DEU".into());
    assert_eq!(texts(&mut tg), ["FFA", "T"]);
}

#[test]
fn fire_support_lines_show_t2_and_as_at_both_ends_in_version_16() {
    for (line_type, label) in [(tl::FSCL, "FSCL"), (tl::CFL, "CFL"), (tl::NFL, "NFL")] {
        let mut tg = versioned(line_type, "16", LINE);
        (tg.t2, tg.as_) = ("MND".into(), "USA".into());
        let text = format!("{label} MND (USA)");
        assert_eq!(texts(&mut tg), [text.as_str(), text.as_str()]);
    }
    let mut tg = versioned(tl::ICL, "16", LINE);
    (tg.t, tg.t1) = ("T".into(), "EUSTIS".into());
    assert_eq!(texts(&mut tg), ["ICL EUSTIS", "ICL EUSTIS"]);
    let mut tg = versioned(tl::FSCL, "15", LINE);
    (tg.t, tg.t2) = ("T".into(), "MND".into());
    assert_eq!(texts(&mut tg), ["T FSCL", "T FSCL"]);
}

#[test]
fn airspace_coordination_area_stack_of_version_16() {
    let mut tg = versioned(tl::ACA, "16", AREA);
    (tg.t, tg.t2, tg.x, tg.x1) = ("T".into(), "T2".into(), "X".into(), "X1".into());
    (tg.y, tg.w, tg.w1) = ("Y".into(), "W".into(), "W1".into());
    assert_eq!(
        texts(&mut tg),
        ["ACA T", "T2", "MIN ALT X", "MAX ALT X1", "Y", "W - W1"]
    );
}

#[test]
fn counterattacks_name_t_where_their_template_does() {
    for (line_type, version, expected) in [
        (tl::CATK, "11", "CATK"),
        (tl::CATK, "15", "CATK T"),
        (tl::CATK, "16", "CATK T"),
        (tl::CATKBYFIRE, "15", "CATK"),
        (tl::CATKBYFIRE, "16", "CATK T"),
    ] {
        let mut tg = versioned(line_type, version, AXIS);
        tg.t = "T".into();
        assert_eq!(texts(&mut tg), [expected], "{line_type} {version}");
    }
}

#[test]
fn version_16_areas_and_lines_carry_their_template_fields() {
    let mut tg = versioned(tl::FLOT, "16", LINE);
    assert!(texts(&mut tg).is_empty());
    let mut tg = versioned(tl::FLOT, "11", LINE);
    assert_eq!(texts(&mut tg), ["FLOT", "FLOT"]);
    let mut tg = versioned(tl::LAA, "16", AREA);
    tg.h = "H".into();
    assert_eq!(texts(&mut tg), ["LAA", "H"]);
    let mut tg = versioned(tl::DRCL, "16", AREA);
    (tg.t, tg.h) = ("T".into(), "30 CGH".into());
    assert_eq!(texts(&mut tg), ["30 CGH"]);
    let mut tg = versioned(tl::FPOL, "16", LINE);
    tg.w = "W".into();
    assert_eq!(texts(&mut tg), ["W", "P(F)"]);
}

#[test]
fn corps_support_area_of_version_16_shows_n_on_both_sides_when_hostile() {
    let mut tg = versioned(tl::CSA, "16", AREA);
    (tg.t, tg.w, tg.w1) = ("BLUE".into(), "W".into(), "W1".into());
    assert_eq!(texts(&mut tg), ["CSA BLUE", "W - ", "W1"]);
    let mut tg = versioned(tl::CSA, "16", AREA);
    tg.t = "BLUE".into();
    tg.standard_identity = "06".into();
    assert_eq!(texts(&mut tg), ["CSA BLUE", "ENY", "ENY"]);
    let mut tg = versioned(tl::CSA, "15", AREA);
    tg.t = "BLUE".into();
    assert_eq!(texts(&mut tg), ["CSA"]);
}
