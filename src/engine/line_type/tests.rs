use super::classes::{MsInfo, is_autoshape, is_change1_area, is_channel};
use super::*;
use crate::engine::tactical_lines::*;

#[test]
fn control_measures_by_version() {
    // clsRenderer.getCMLineType: edition-specific branches.
    assert_eq!(line_type(11, 25, 172_000), Some(WFZ_REVD));
    assert_eq!(line_type(13, 25, 172_000), Some(WFZ_REVD));
    assert_eq!(line_type(15, 25, 172_000), Some(WFZ));
    assert_eq!(line_type(11, 25, 120_500), Some(BASE_CAMP_REVD));
    assert_eq!(line_type(15, 25, 120_500), Some(BASE_CAMP));
    assert_eq!(line_type(11, 25, 120_600), Some(GUERILLA_BASE_REVD));
    assert_eq!(line_type(13, 25, 260_400), Some(BCL));
    assert_eq!(line_type(11, 25, 260_400), Some(BCL_REVD));
    assert_eq!(line_type(11, 25, 310_100), Some(DHA_REVD));
    assert_eq!(line_type(15, 25, 310_100), Some(DHA));
    assert_eq!(line_type(11, 25, 150_300), Some(ASSY));
    assert_eq!(line_type(11, 25, 241_601), Some(SENSOR));
    assert_eq!(line_type(15, 25, 240_804), None);
}

#[test]
fn control_measures_without_edition_branch() {
    assert_eq!(line_type(11, 25, 110_500), Some(DECISION_LINE));
    assert_eq!(line_type(15, 25, 110_500), Some(DECISION_LINE));
    assert_eq!(line_type(11, 25, 151_000), Some(FORT));
    assert_eq!(line_type(11, 25, 150_600), Some(DZ));
    assert_eq!(line_type(11, 25, 220_102), Some(BEARING_EW));
    assert_eq!(line_type(11, 25, 999_999), None);
}

#[test]
fn weather_ignores_version_and_set() {
    assert_eq!(line_type(11, 45, 110_301), Some(CF));
    assert_eq!(line_type(15, 46, 110_301), Some(CF));
    assert_eq!(line_type(11, 45, 110_402), Some(UPPER_TROUGH));
    assert_eq!(line_type(11, 45, 162_004), None);
    assert_eq!(line_type(11, 10, 110_301), None);
    assert_eq!(is_weather(25, 110_301), None);
}

#[test]
fn version_15_troughs_are_smooth_curves() {
    assert_eq!(line_type(15, 45, 110_401), Some(ESTIMATED_ICE_EDGE));
    assert_eq!(line_type(15, 45, 110_402), Some(UPPER_AIR));
    assert_eq!(line_type(11, 45, 110_401), Some(TROUGH));
    assert_eq!(is_weather(45, 110_402), Some(UPPER_TROUGH));
}

#[test]
fn classes() {
    assert!(is_change1_area(RANGE_FAN));
    assert!(!is_change1_area(PL));
    assert!(is_channel(MAIN));
    assert!(!is_channel(PL));
}

#[test]
fn autoshape() {
    let fixed = MsInfo {
        draw_rule: 1,
        min_points: 3,
        max_points: 3,
    };
    let open = MsInfo {
        draw_rule: 1,
        min_points: 2,
        max_points: 50,
    };
    assert!(is_autoshape(BS_RECTANGLE, None));
    assert!(!is_autoshape(PL, None));
    assert!(is_autoshape(BYPASS, Some(fixed)));
    assert!(!is_autoshape(PL, Some(open)));
    assert!(!is_autoshape(DIRATKGND, Some(fixed)));
    assert!(!is_autoshape(RANGE_FAN, Some(fixed)));
}
