//! Graphics the ported renderer leaves incomplete for MIL-STD-2525E change
//! 1 (METOC) and for version 16, drawn as the standard's templates show
//! them.

use crate::construction::{GeoGeometry, LabelPlacement, PartRole};
use crate::definition::{ControlPoint, GraphicDefinition, GraphicId};
use crate::family::{Config, construct};
use crate::geo::GeoPoint;
use crate::render::{FixedAdvanceMetrics, LocalEquirectangular, View, render};
use crate::sidc::SymbolId;
use crate::style::{DashPattern, Fill, Motif, Rgba};

const LINE: [(f64, f64); 4] = [(20.0, 50.0), (20.02, 50.01), (20.04, 50.0), (20.06, 50.01)];
const AREA: [(f64, f64); 4] = [(20.0, 50.0), (20.06, 50.0), (20.06, 50.03), (20.0, 50.03)];

fn def(sidc: &str, points: &[(f64, f64)]) -> GraphicDefinition {
    GraphicDefinition::new(
        GraphicId::new("g").unwrap(),
        SymbolId::parse(sidc).unwrap(),
        points
            .iter()
            .map(|&(lon, lat)| ControlPoint::ground(GeoPoint::new(lon, lat).unwrap()))
            .collect(),
    )
}

#[test]
fn isopleths_carry_their_value_at_both_ends_and_the_middle() {
    let mut d = def("15004500001802000000", &LINE);
    d.modifiers.designation = Some("540".to_owned());
    let c = construct(&d, &Config::default()).unwrap();
    let texts: Vec<&str> = c.labels.iter().map(|l| l.text.as_str()).collect();
    assert_eq!(texts, ["540"; 3]);
    let [start, middle, end] = c.labels.as_slice() else {
        panic!("three labels")
    };
    assert!(matches!(start.placement, LabelPlacement::LineEnd { .. }));
    assert!(matches!(middle.placement, LabelPlacement::Along { .. }));
    assert!(matches!(end.placement, LabelPlacement::LineEnd { .. }));
    let near =
        |p: GeoPoint, lon: f64, lat: f64| (p.lon() - lon).abs() + (p.lat() - lat).abs() < 1e-6;
    assert!(near(start.anchor, 20.0, 50.0) && near(end.anchor, 20.06, 50.01));
    // Halfway along the curve, which runs through the control points.
    assert!((middle.anchor.lon() - 20.03).abs() < 0.005);
    // The 2525D edition declares no such amplifier.
    let mut d11 = def("11004500001802000000", &LINE);
    d11.modifiers.designation = Some("540".to_owned());
    assert!(construct(&d11, &Config::default()).is_err());
}

#[test]
fn a_depth_contour_carries_its_depth() {
    let mut d = def("15004600001201030000", &LINE);
    d.modifiers.designation = Some("30 M".to_owned());
    let c = construct(&d, &Config::default()).unwrap();
    assert_eq!(c.labels.len(), 3);
    assert!(c.labels.iter().all(|l| l.text == "30 M"), "{:?}", c.labels);
}

#[test]
fn tropical_storm_wind_areas_are_closed_red_areas_with_their_time() {
    let mut d = def("15004500001620040000", &AREA);
    d.modifiers.dtg_start = Some("081200Z".to_owned());
    let c = construct(&d, &Config::default()).unwrap();
    let [part] = c.parts.as_slice() else {
        panic!("one part: {:?}", c.parts)
    };
    assert!(matches!(part.geometry, GeoGeometry::Ring(_)));
    assert_eq!(part.stroke.map(|s| s.color), Some(Rgba::RED));
    let [label] = c.labels.as_slice() else {
        panic!("one label")
    };
    assert_eq!(label.text, "081200Z");
    assert!(label.anchor.lat() > 50.029);
}

#[test]
fn pattern_areas_carry_their_figures_without_an_outline() {
    // Swept Area: magenta dots, no boundary.
    let c = construct(&def("15004600001504000000", &AREA), &Config::default()).unwrap();
    let filled: Vec<_> = c
        .parts
        .iter()
        .filter(|p| matches!(p.fill, Fill::Pattern(_)))
        .collect();
    let [area] = filled.as_slice() else {
        panic!("one patterned part: {:?}", c.parts)
    };
    let Fill::Pattern(pattern) = area.fill else {
        panic!("{:?}", area.fill)
    };
    assert_eq!(pattern.motif, Motif::Dot);
    assert_eq!(pattern.color, Rgba::opaque(255, 0, 255));
    assert!(area.stroke.is_none());
    let frame = LocalEquirectangular::new(19.99, 50.04, 25_000.0, 96.0);
    let plan = render(
        &c,
        &View::new(0, 0),
        &frame,
        &FixedAdvanceMetrics::default(),
    )
    .unwrap();
    let dots = plan.screen.iter().filter(|i| i.role == PartRole::Pattern);
    assert!(dots.count() > 0);
}

#[test]
fn fish_traps_keep_their_figures_under_a_dashed_outline() {
    let c = construct(&def("15004600001203120000", &AREA), &Config::default()).unwrap();
    assert!(c.parts.iter().any(|p| matches!(p.fill, Fill::Pattern(_))));
    assert!(
        c.parts
            .iter()
            .any(|p| p.stroke.is_some_and(|s| s.dash == DashPattern::Dashed))
    );
}

#[test]
fn turbulence_is_outlined_in_dots() {
    let c = construct(&def("15004500001703000000", &AREA), &Config::default()).unwrap();
    assert!(!c.parts.is_empty());
    assert!(
        c.parts
            .iter()
            .all(|p| p.stroke.is_some_and(|s| s.dash == DashPattern::Dotted))
    );
}

#[test]
fn a_pattern_zoomed_far_in_stays_within_the_vertex_budget() {
    // Beach Slope - Steep: a dot every 30 px over some 16,000 x 12,000 px.
    let c = construct(&def("15004600001302040000", &AREA), &Config::default()).unwrap();
    let frame = LocalEquirectangular::new(19.99, 50.04, 1_000.0, 96.0);
    let plan = render(
        &c,
        &View::new(0, 0),
        &frame,
        &FixedAdvanceMetrics::default(),
    )
    .unwrap();
    let dots = plan
        .screen
        .iter()
        .filter(|i| i.role == PartRole::Pattern)
        .count();
    assert!((1..=4000).contains(&dots), "{dots} dots");
}

fn render_texts(c: &crate::construction::Construction) -> Vec<String> {
    let frame = LocalEquirectangular::new(19.9, 50.1, 250_000.0, 96.0);
    let plan = render(c, &View::new(0, 0), &frame, &FixedAdvanceMetrics::default()).unwrap();
    plan.labels.iter().map(|l| l.text.clone()).collect()
}

#[test]
fn the_navigational_rhumb_line_keeps_its_course_and_draws_it() {
    // About 100 km due east at 50°N, where the geodesic would bow some
    // 230 m toward the pole.
    let mut d = def("16032500002201090000", &[(20.0, 50.0), (21.4, 50.0)]);
    d.modifiers.designation = Some("15".to_owned());
    let c = construct(&d, &Config::default()).unwrap();
    let line = c
        .parts
        .iter()
        .find(|p| matches!(p.geometry, GeoGeometry::Line(_)))
        .unwrap();
    let points = line.geometry.points();
    assert!(points.len() > 10);
    assert!(points.iter().all(|p| (p.lat() - 50.0).abs() < 1e-9));
    // AN, not entered, is the line's course.
    let texts = render_texts(&c);
    assert!(texts.contains(&"090".to_owned()), "{texts:?}");
    assert!(texts.contains(&"15".to_owned()));
    d.modifiers.azimuths_deg = vec![92.0];
    let texts = render_texts(&construct(&d, &Config::default()).unwrap());
    assert!(texts.contains(&"092".to_owned()), "{texts:?}");
    assert!(!texts.contains(&"090".to_owned()));
}

#[test]
fn the_rhumb_line_puts_an_on_its_north_or_west_side() {
    use super::rhumb::left_is_north_west as left;
    // East-bound and north-bound lines have north and west on their left.
    assert!(left(90.0) && left(0.0) && left(330.0) && left(100.0));
    assert!(!left(270.0) && !left(180.0) && !left(200.0));
}

#[test]
fn the_zone_of_fire_outline_is_broken_in_every_status() {
    let present = construct(&def("16032500002426000000", &AREA), &Config::default()).unwrap();
    assert!(
        present
            .parts
            .iter()
            .all(|p| p.stroke.is_some_and(|s| s.dash == DashPattern::Dashed))
    );
}

#[test]
fn the_version_16_single_target_is_drawn_as_in_2525d() {
    let draw = |sidc: &str| {
        let mut d = def(sidc, &[(20.0, 50.0), (20.05, 50.01)]);
        d.modifiers.distances_m = vec![2000.0];
        d.modifiers.designation = Some("NSFS002".to_owned());
        let c = construct(&d, &Config::default()).unwrap();
        let parts: Vec<_> = c.parts.iter().map(|p| p.geometry.clone()).collect();
        (parts, render_texts(&c))
    };
    let app6e = draw("16032500002408040000");
    assert!(!app6e.0.is_empty());
    assert_eq!(app6e, draw("11032500002408040000"));
}
