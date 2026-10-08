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
