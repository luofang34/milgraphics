use crate::definition::{ControlPoint, GraphicDefinition, GraphicId};
use crate::family::{Config, construct};
use crate::geo::GeoPoint;
use crate::render::{
    FixedAdvanceMetrics, LocalEquirectangular, Projection, RenderPlan, ScreenPoint, View, render,
};
use crate::sidc::SymbolId;

/// A local frame whose depth falls linearly down the screen, as ground seen from a tilted
/// camera does near the middle of the view.
struct Sloped(LocalEquirectangular);

impl Sloped {
    fn depth(s: ScreenPoint) -> f32 {
        (0.5 + s.y * 1e-4) as f32
    }
}

impl Projection for Sloped {
    fn project(&self, p: GeoPoint) -> Option<ScreenPoint> {
        self.0.project(p)
    }

    fn project_depth(&self, p: GeoPoint) -> Option<(ScreenPoint, Option<f32>)> {
        self.0.project(p).map(|s| (s, Some(Self::depth(s))))
    }

    fn unproject(&self, s: ScreenPoint) -> Option<GeoPoint> {
        self.0.unproject(s)
    }
}

fn plan(sidc: &str, points: &[(f64, f64)], projection: &dyn Projection) -> RenderPlan {
    let d = GraphicDefinition::new(
        GraphicId::new("g").unwrap(),
        SymbolId::parse(sidc).unwrap(),
        points
            .iter()
            .map(|&(lon, lat)| ControlPoint::ground(GeoPoint::new(lon, lat).unwrap()))
            .collect(),
    );
    let c = construct(&d, &Config::default()).unwrap();
    render(
        &c,
        &View::new(0, 0),
        projection,
        &FixedAdvanceMetrics::default(),
    )
    .unwrap()
}

fn frame() -> LocalEquirectangular {
    LocalEquirectangular::new(19.95, 50.05, 25_000.0, 96.0)
}

/// FLOT (scallops), Main Attack (arrowhead and tail drawn in pixels) and Bypass Easy (a
/// mission task's glyph).
const CASES: [(&str, &[(f64, f64)]); 3] = [
    ("11032500001401000000", &[(20.0, 50.0), (20.1, 50.0)]),
    (
        "11032500001514030000",
        &[(20.1, 50.0), (20.0, 50.0), (20.05, 50.01)],
    ),
    (
        "11032500002706010000",
        &[(20.0, 50.0), (20.0, 50.02), (20.04, 50.01)],
    ),
];

#[test]
fn every_point_of_the_screen_tier_takes_the_depth_of_the_ground_under_it() {
    let sloped = Sloped(frame());
    for (sidc, points) in CASES {
        let p = plan(sidc, points, &sloped);
        assert!(!p.screen.is_empty(), "{sidc} draws");
        for item in &p.screen {
            let shape = item.shape.points();
            assert_eq!(item.depth.len(), shape.len(), "{sidc}: a depth per point");
            for (&s, &d) in shape.iter().zip(&item.depth) {
                // Projected points carry the host's depth exactly; points placed on screen
                // beside them, within a few dozen pixels, differ by that slope.
                assert!(
                    (d - Sloped::depth(s)).abs() < 5e-3,
                    "{sidc}: ({}, {}) at depth {d}, ground at {}",
                    s.x,
                    s.y,
                    Sloped::depth(s)
                );
            }
        }
        for (screen, depth) in p
            .labels
            .iter()
            .map(|l| (l.screen, l.depth))
            .chain(p.handles.iter().map(|h| (h.screen, h.depth)))
        {
            let s = screen.unwrap();
            let d = depth.unwrap();
            assert!((d - Sloped::depth(s)).abs() < 5e-3, "{sidc}: anchor at {d}");
        }
    }
}

#[test]
fn a_host_that_gives_no_depth_gets_none() {
    for (sidc, points) in CASES {
        let p = plan(sidc, points, &frame());
        assert!(p.screen.iter().all(|i| i.depth.is_empty()), "{sidc}");
        assert!(p.labels.iter().all(|l| l.depth.is_none()));
        assert!(p.handles.iter().all(|h| h.depth.is_none()));
        assert!(p.symbols.iter().all(|s| s.depth.is_none()));
    }
}
