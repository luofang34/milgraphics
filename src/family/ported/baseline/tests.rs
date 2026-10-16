use super::Edge;
use crate::construction::{Decoration, GeoGeometry, PartRole};
use crate::definition::{ControlPoint, GraphicDefinition, GraphicId};
use crate::family::{Config, construct};
use crate::geo::GeoPoint;
use crate::render::{FixedAdvanceMetrics, LocalEquirectangular, Projection, View, render};
use crate::sidc::SymbolId;

const LINE: [(f64, f64); 3] = [(20.0, 50.0), (20.03, 50.012), (20.06, 50.0)];

fn def(sidc: &str) -> GraphicDefinition {
    GraphicDefinition::new(
        GraphicId::new("g").unwrap(),
        SymbolId::parse(sidc).unwrap(),
        LINE.iter()
            .map(|&(lon, lat)| ControlPoint::ground(GeoPoint::new(lon, lat).unwrap()))
            .collect(),
    )
}

fn baseline(c: &crate::construction::Construction) -> Vec<(usize, Edge)> {
    c.decorations
        .iter()
        .find_map(|d| match &d.0 {
            Decoration::Engine { baseline, .. } => Some(baseline.clone()),
            _ => None,
        })
        .unwrap()
}

#[test]
fn a_cold_front_draws_its_line_on_the_ground_and_its_teeth_on_screen() {
    let c = construct(&def("11004500001103010000"), &Config::default()).unwrap();
    assert_eq!(
        baseline(&c),
        [(0, Edge::Straight), (1, Edge::Straight)],
        "both edges, and no closing edge"
    );
    let lines: Vec<&Vec<GeoPoint>> = c
        .parts
        .iter()
        .filter(|p| p.role == PartRole::Line)
        .filter_map(|p| match &p.geometry {
            GeoGeometry::Line(points) => Some(points),
            _ => None,
        })
        .collect();
    let control: Vec<GeoPoint> = def("11004500001103010000").positions().collect();
    assert_eq!(lines, [&control], "one line through the control points");
}

#[test]
fn a_broken_line_stays_in_pixels() {
    // Frontogenesis breaks its line at pixel intervals, so no edge is whole.
    let c = construct(&def("11004500001103030000"), &Config::default()).unwrap();
    assert!(baseline(&c).is_empty());
    assert!(c.parts.iter().all(|p| p.role != PartRole::Line));
}

#[test]
fn the_screen_tier_holds_nothing_on_the_ground_line() {
    let d = def("11004500001103010000");
    let c = construct(&d, &Config::default()).unwrap();
    let projection = LocalEquirectangular::new(19.99, 50.02, 50_000.0, 96.0);
    let plan = render(
        &c,
        &View::new(0, 0),
        &projection,
        &FixedAdvanceMetrics::default(),
    )
    .unwrap();
    let control: Vec<_> = d
        .positions()
        .map(|p| projection.project(p).unwrap())
        .collect();
    let teeth: Vec<_> = plan.screen.iter().filter(|i| i.decoration).collect();
    assert!(!teeth.is_empty(), "the teeth stay in the screen tier");
    for item in teeth {
        for w in item.shape.points().windows(2) {
            let [u, v] = w else { continue };
            let on_line = control.windows(2).any(|e| {
                let [p, q] = e else { return false };
                let (dx, dy) = (q.x - p.x, q.y - p.y);
                let len = dx.hypot(dy);
                let off = |s: &crate::render::ScreenPoint| {
                    ((s.x - p.x) * dy - (s.y - p.y) * dx).abs() / len
                };
                off(u) < 0.5 && off(v) < 0.5
            });
            assert!(
                !on_line,
                "a screen segment {u:?}-{v:?} lies on the ground line"
            );
        }
    }
}

#[test]
fn a_depth_curve_follows_its_spline_on_the_ground() {
    let d = def("15004600001201020000");
    let c = construct(&d, &Config::default()).unwrap();
    assert_eq!(baseline(&c), [(0, Edge::Spline), (1, Edge::Spline)]);
    let control: Vec<GeoPoint> = d.positions().collect();
    let line = c
        .parts
        .iter()
        .find_map(|p| match (&p.role, &p.geometry) {
            (PartRole::Line, GeoGeometry::Line(points)) => Some(points),
            _ => None,
        })
        .unwrap();
    assert!(
        line.len() > control.len(),
        "the curve is sampled between control points"
    );
    assert!(line.contains(&control[1]), "and passes through them");
    assert_eq!(
        (line.first(), line.last()),
        (control.first(), control.last())
    );
}

#[test]
fn a_line_changing_style_partway_along_an_edge_keeps_that_edge_in_pixels() {
    // A leading line is solid for the first half of its samples and dashed
    // after, so the edge it changes on is not one line.
    let c = construct(&def("15004600001204070000"), &Config::default()).unwrap();
    let edges = baseline(&c);
    assert!(edges.iter().all(|&(i, _)| i == 0), "{edges:?}");
}
