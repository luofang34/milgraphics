use super::*;

#[test]
fn sector_1_codes_name_the_mine_types_in_order() {
    assert_eq!(types(14), [0]);
    assert_eq!(types(19), [5]);
    assert_eq!(types(20), [0, 1]);
    assert_eq!(types(34), [4, 5]);
    assert_eq!(types(35), [0, 1, 2]);
    assert_eq!(types(42), [0, 3, 4]);
    assert_eq!(types(50), [1, 4, 5]);
    assert!(types(51).is_empty() && types(13).is_empty() && types(0).is_empty());
}

#[test]
fn a_single_type_is_drawn_three_times_and_unknown_codes_are_unspecified() {
    assert_eq!(entities(16), [280_300; 3]);
    assert_eq!(entities(0), [UNSPECIFIED; 3]);
    assert_eq!(entities(13), [UNSPECIFIED; 3]);
    assert_eq!(entities(21), [280_200, 280_300]);
    assert_eq!(entities(46), [280_201, 280_300, 280_500]);
}

#[test]
fn mine_cluster_has_no_symbol_to_embed() {
    assert!(entities(19).is_empty());
    assert_eq!(entities(31), [280_300]);
}
