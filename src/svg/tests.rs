use super::*;
use crate::definition::{ControlPoint, GraphicDefinition, GraphicId};
use crate::family::{Config, construct};
use crate::geo::GeoPoint;
use crate::render::{FixedAdvanceMetrics, LocalEquirectangular, View, render};
use crate::sidc::SymbolId;

fn svg_for(sidc: &str, t: &str) -> String {
    let mut d = GraphicDefinition::new(
        GraphicId::new("g").unwrap(),
        SymbolId::parse(sidc).unwrap(),
        vec![
            ControlPoint::ground(GeoPoint::new(20.0, 50.0).unwrap()),
            ControlPoint::ground(GeoPoint::new(20.1, 50.02).unwrap()),
        ],
    );
    d.modifiers.designation = Some(t.to_owned());
    let c = construct(&d, &Config::default()).unwrap();
    let view = View::new(0, 0);
    let frame = LocalEquirectangular::new(19.95, 50.07, 50_000.0, 96.0);
    let plan = render(&c, &view, &frame, &FixedAdvanceMetrics::default()).unwrap();
    to_svg(&plan, &SvgOptions::new(1100.0, 900.0))
}

#[test]
fn output_is_stable_and_well_formed() {
    let a = svg_for("11032500001403000000", "ALPHA");
    assert_eq!(a, svg_for("11032500001403000000", "ALPHA"));
    assert!(a.starts_with("<svg "));
    assert!(a.ends_with("</svg>\n"));
    assert_eq!(a.matches("<polyline").count(), 1);
    assert_eq!(a.matches(">PL ALPHA</text>").count(), 2);
    assert!(!a.contains("stroke-dasharray"));
}

#[test]
fn planned_status_is_dashed_in_line_widths() {
    let a = svg_for("11032510001403000000", "A");
    assert!(a.contains(r#"stroke-dasharray="6 6""#), "{a}");
}

#[test]
fn text_is_escaped() {
    let a = svg_for("11032500001403000000", "<b>&\"'");
    assert!(a.contains("PL &lt;b&gt;&amp;&quot;&#39;</text>"), "{a}");
    assert!(!a.contains("<b>"));
}

#[test]
fn numbers_are_rounded_without_noise() {
    assert_eq!(num(1.0), "1");
    assert_eq!(num(-0.001), "0");
    assert_eq!(num(2.345_6), "2.35");
    assert_eq!(num(-10.5), "-10.5");
}
