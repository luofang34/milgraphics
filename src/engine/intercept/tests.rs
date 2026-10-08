use super::*;

#[test]
fn duplicates_are_drawn_as_their_original_marked_as_feints() {
    let feint = SymbolId::parse("10032500001406050000").unwrap();
    assert_eq!(engine_symbol(&feint).as_str(), "10032501001406030000");
    let other = SymbolId::parse("10032500001406030000").unwrap();
    assert_eq!(engine_symbol(&other), other);
    let not_control_measure = SymbolId::parse("10034500001406050000").unwrap();
    assert_eq!(engine_symbol(&not_control_measure), not_control_measure);
}

#[test]
fn version_16_codes_are_drawn_as_another_edition_or_status_draws_them() {
    let target = SymbolId::parse("16032500002408040000").unwrap();
    assert_eq!(engine_symbol(&target).as_str(), "11032500002408040000");
    let zone = SymbolId::parse("16032500002426000000").unwrap();
    assert_eq!(engine_symbol(&zone).as_str(), "16032510002426000000");
    let target_e = SymbolId::parse("15032500002408040000").unwrap();
    assert_eq!(engine_symbol(&target_e), target_e);
}
