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

fn baseline(c: &crate::construction::Construction) -> Vec<usize> {
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
    assert_eq!(baseline(&c), [0, 1], "both edges, and no closing edge");
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
