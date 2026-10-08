use super::*;

fn no_country(_: i32) -> Option<String> {
    None
}

#[test]
fn defaults() {
    let tg = Tg::new(&Settings::default());
    assert_eq!(tg.n, "ENY");
    assert_eq!(tg.status, "P");
    assert_eq!(tg.line_cap, CAP_SQUARE);
    assert_eq!(tg.icon_size, 50);
    assert!(!tg.is_hostile());
}

#[test]
fn location_prefers_y() {
    let mut tg = Tg::new(&Settings::default());
    tg.h = "H".to_owned();
    assert_eq!(tg.location(), "H");
    tg.y = "Y".to_owned();
    assert_eq!(tg.location(), "Y");
}

#[test]
fn symbol_id_of_planned_hostile_battalion() {
    let mut tg = Tg::new(&Settings::default());
    // version 11, identity 06, symbol set 25, status 1, amplifier 16
    tg.set_symbol_id("110625101600001400000000000000", &no_country)
        .unwrap();
    assert_eq!(tg.standard_identity, "06");
    assert!(tg.is_hostile());
    assert_eq!(tg.status, "A");
    assert_eq!(tg.line_style, 1);
    assert_eq!(tg.echelon_symbol, "II");
}

#[test]
fn symbol_id_country_fills_as_once() {
    let mut tg = Tg::new(&Settings::default());
    let genc = |cc: i32| (cc == 840).then(|| "USA".to_owned());
    tg.set_symbol_id("110325000000001400000000000840", &genc)
        .unwrap();
    assert_eq!(tg.as_, "USA");
    tg.as_ = "XYZ".to_owned();
    tg.set_symbol_id("110325000000001400000000000840", &genc)
        .unwrap();
    assert_eq!(tg.as_, "XYZ");
}

#[test]
fn other_symbol_sets_only_store_the_code() {
    let mut tg = Tg::new(&Settings::default());
    tg.set_symbol_id("110345000000001400000000000000", &no_country)
        .unwrap();
    assert_eq!(tg.standard_identity, "00");
    assert_eq!(tg.symbol_id, "110345000000001400000000000000");
}

#[test]
fn malformed_code_is_an_error() {
    let mut tg = Tg::new(&Settings::default());
    assert!(
        tg.set_symbol_id("11xx25000000001400000000000000", &no_country)
            .is_err()
    );
}

#[test]
fn modifier_label_defaults_fit() {
    assert!(ModifierLabel::default().fits_mbr);
}
