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
    assert_eq!(slots(16), [Some(280_300); 3]);
    assert_eq!(slots(0), [Some(UNSPECIFIED); 3]);
    assert_eq!(slots(13), [Some(UNSPECIFIED); 3]);
    assert_eq!(slots(21), [Some(280_200), Some(280_300)]);
    assert_eq!(slots(46), [Some(280_201), Some(280_300), Some(280_500)]);
}

#[test]
fn mine_cluster_takes_its_place_in_the_row_as_a_figure() {
    assert_eq!(slots(19), [None; 3]);
    assert_eq!(slots(31), [Some(280_300), None]);
    let outline = cluster_outline();
    let width = outline.iter().map(|p| p[0]).fold(f64::MIN, f64::max)
        - outline.iter().map(|p| p[0]).fold(f64::MAX, f64::min);
    assert!((width - MINE_PX).abs() < 1e-9);
}
