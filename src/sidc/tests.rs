use super::*;

#[test]
fn fields_follow_the_2525d_layout() {
    let id = SymbolId::parse("11032510001403000000").unwrap();
    assert_eq!(id.version_code(), 11);
    assert_eq!(id.standard(), Some(StandardVersion::Mil2525Dch1));
    assert_eq!(id.context(), 0);
    assert_eq!(id.identity(), 3);
    assert_eq!(id.symbol_set(), 25);
    assert_eq!(id.status(), 1);
    assert_eq!(id.hq_tf_dummy(), 0);
    assert_eq!(id.amplifier(), 0);
    assert_eq!(id.entity().get(), 140_300);
    assert_eq!(id.entity().to_string(), "140300");
    assert_eq!(id.modifier1(), 0);
    assert_eq!(id.modifier2(), 0);
}

#[test]
fn unknown_versions_parse_and_report_no_standard() {
    let id = SymbolId::parse("99032500001403000000").unwrap();
    assert_eq!(id.version_code(), 99);
    assert_eq!(id.standard(), None);
}

#[test]
fn thirty_digit_codes_round_trip() {
    let code = "110325000014030000001234567890";
    let id = SymbolId::parse(code).unwrap();
    assert_eq!(id.as_str(), code);
    assert_eq!(id.entity().get(), 140_300);
}

#[test]
fn malformed_codes_are_rejected() {
    for bad in [
        "",
        "1103250000140300000",
        "110325000014030000000",
        "1103250000140300000x",
        "１１０３２５００００１４０３００００００",
    ] {
        assert!(SymbolId::parse(bad).is_err(), "{bad:?}");
    }
}

#[test]
fn serde_uses_the_digit_string() {
    let id = SymbolId::parse("15032500001202000000").unwrap();
    let json = serde_json::to_string(&id).unwrap();
    assert_eq!(json, "\"15032500001202000000\"");
    assert_eq!(serde_json::from_str::<SymbolId>(&json).unwrap(), id);
    assert!(serde_json::from_str::<SymbolId>("\"12\"").is_err());
}

#[test]
fn every_standard_round_trips_its_code() {
    for &standard in StandardVersion::ALL {
        assert_eq!(StandardVersion::from_code(standard.code()), Some(standard));
    }
}
