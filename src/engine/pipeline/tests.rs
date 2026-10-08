use super::*;
use crate::modifier::Modifiers;
use crate::sidc::SymbolId;

#[test]
fn a_phase_line_draws_its_line_and_both_end_labels() {
    let symbol = SymbolId::parse("11032500001403000000").unwrap();
    let modifiers = Modifiers {
        designation: Some("ALPHA".into()),
        ..Modifiers::default()
    };
    let width = |t: &str| 7.0 * t.chars().count() as f64;
    let out = render(&Input {
        line_type: tl::PL,
        symbol: &symbol,
        pixels: vec![Pt::new(100.0, 100.0), Pt::new(300.0, 120.0)],
        modifiers: &modifiers,
        meters_per_pixel: 10.0,
        text_width: &width,
        ms_info: None,
        style: crate::engine::api::Style::default(),
    })
    .unwrap();
    let lines: Vec<_> = out.shapes.iter().flat_map(Shape::polylines).collect();
    assert_eq!(lines, vec![vec![(100.0, 100.0), (300.0, 120.0)]]);
    let texts: Vec<&str> = out.labels.iter().map(|l| l.text.as_str()).collect();
    assert_eq!(texts, ["PL ALPHA", "PL ALPHA"]);
}
