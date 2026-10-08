use super::*;

#[test]
fn values_match_upstream() {
    assert_eq!(BS_LINE, 10_000_000);
    assert_eq!(PL, 22_124_000);
    assert_eq!(BOUNDARY, 22_121_000);
    assert_eq!(BCL, 1_325_260_400);
    assert_eq!(WFZ, 22_235_000);
    assert_eq!(RANGE_FAN, 243_111_000);
    assert_eq!(RECTANGULAR, 24_311_000);
    assert_eq!(CF, 31_131_000);
    assert_eq!(UPPER_TROUGH, 45_110_402);
    assert_eq!(FREEFORM, 317_100_000);
    assert_eq!(MEDIUM_SILT, 324_111_000);
    assert_eq!(OPERATOR_DEFINED, 32_560_000);
}
