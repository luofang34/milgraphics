use super::*;

#[test]
fn defaults_match_the_oracle_configuration() {
    let s = Settings::default();
    assert_eq!(s.dpi, 96);
    assert_eq!(s.label_font.size, 12);
    assert!(s.label_font.bold);
    assert_eq!(s.text_outline_width, 0);
    assert!(!s.group_modifiers);
    assert!(s.auto_collapse_modifiers);
    assert!(!s.two_label_only);
    assert!((s.pattern_scale - 1.0).abs() < f64::EPSILON);
    assert!((s.dpi_scale_factor() - 1.0).abs() < f64::EPSILON);
}
