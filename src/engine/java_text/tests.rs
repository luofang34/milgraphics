use super::*;

#[test]
fn double_to_string_matches_java() {
    let cases = [
        (0.0, "0.0"),
        (1.0, "1.0"),
        (100.0, "100.0"),
        (5000.5, "5000.5"),
        (0.001, "0.001"),
        (0.0001, "1.0E-4"),
        (1.5e-4, "1.5E-4"),
        (9_999_999.0, "9999999.0"),
        (1.0e7, "1.0E7"),
        (12_345_678.0, "1.2345678E7"),
        (-2.5, "-2.5"),
        (123.456, "123.456"),
        (0.25, "0.25"),
        (1.0e21, "1.0E21"),
    ];
    for (v, want) in cases {
        assert_eq!(double_to_string(v), want, "{v}");
    }
    assert_eq!(double_to_string(f64::NAN), "NaN");
}

#[test]
fn parse_double_accepts_java_forms() {
    assert_eq!(parse_double(" 12 ").unwrap(), 12.0);
    assert_eq!(parse_double("1e3").unwrap(), 1000.0);
    assert_eq!(parse_double("-.5").unwrap(), -0.5);
    assert_eq!(parse_double("2.5f").unwrap(), 2.5);
    assert!(parse_double("inf").is_err());
    assert!(parse_double("").is_err());
    assert!(parse_double("1ab").is_err());
    assert!(is_number("3."));
    assert!(!is_number("x"));
}
