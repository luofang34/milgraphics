use super::*;

#[test]
fn closed_polygons() {
    assert!(is_closed_polygon(NAI));
    assert!(!is_closed_polygon(PL));
    assert!(!is_closed_polygon(MAIN));
}

#[test]
fn fill_lines() {
    assert!(lines_with_fill(PL));
    assert!(!lines_with_fill(MAIN));
}
