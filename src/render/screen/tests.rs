use core::cell::Cell;

use crate::definition::{ControlPoint, GraphicDefinition, GraphicId};
use crate::family::{Config, construct};
use crate::geo::GeoPoint;
use crate::render::{
    FixedAdvanceMetrics, LocalEquirectangular, PlanContent, Projection, RenderPlan, ScreenPoint,
    ScreenRect, ScreenShape, View, render,
};
use crate::sidc::SymbolId;

/// A local frame that counts its projections and may report a viewport.
struct Counted {
    frame: LocalEquirectangular,
    viewport: Option<ScreenRect>,
    calls: Cell<usize>,
}

impl Counted {
    fn new(scale: f64, viewport: Option<ScreenRect>) -> Self {
        Self {
            frame: LocalEquirectangular::new(19.0, 51.0, scale, 96.0),
            viewport,
            calls: Cell::new(0),
        }
    }
}

impl Projection for Counted {
    fn project(&self, p: GeoPoint) -> Option<ScreenPoint> {
        self.calls.set(self.calls.get() + 1);
        self.frame.project(p)
    }

    fn unproject(&self, s: ScreenPoint) -> Option<GeoPoint> {
        self.frame.unproject(s)
    }

    fn viewport(&self) -> Option<ScreenRect> {
        self.viewport
    }
}

/// A phase line 700 km long, whose geodesic bows in the local frame.
fn long_line() -> GraphicDefinition {
    GraphicDefinition::new(
        GraphicId::new("g").unwrap(),
        SymbolId::parse("11032500001403000000").unwrap(),
        [(20.0, 50.0), (30.0, 50.0)]
            .iter()
            .map(|&(lon, lat)| ControlPoint::ground(GeoPoint::new(lon, lat).unwrap()))
            .collect(),
    )
}

fn plan(d: &GraphicDefinition, projection: &dyn Projection) -> RenderPlan {
    plan_with(d, projection, PlanContent::Full)
}

fn plan_with(
    d: &GraphicDefinition,
    projection: &dyn Projection,
    content: PlanContent,
) -> RenderPlan {
    let c = construct(d, &Config::default()).unwrap();
    let mut view = View::new(1, 0);
    view.content = content;
    render(&c, &view, projection, &FixedAdvanceMetrics::default()).unwrap()
}

fn lines(plan: &RenderPlan) -> Vec<Vec<ScreenPoint>> {
    plan.screen
        .iter()
        .filter(|i| !i.decoration)
        .map(|i| match &i.shape {
            ScreenShape::Polyline(p) | ScreenShape::Polygon(p) => p.clone(),
        })
        .collect()
}

fn distance_to(p: ScreenPoint, line: &[ScreenPoint]) -> f64 {
    line.windows(2)
        .map(|w| {
            let (a, b) = (w[0], w[1]);
            let (abx, aby) = b.sub(a);
            let (apx, apy) = p.sub(a);
            let len2 = abx * abx + aby * aby;
            let t = if len2 > 0.0 {
                ((apx * abx + apy * aby) / len2).clamp(0.0, 1.0)
            } else {
                0.0
            };
            (apx - abx * t).hypot(apy - aby * t)
        })
        .fold(f64::INFINITY, f64::min)
}

#[test]
fn a_graphic_a_few_pixels_across_projects_each_vertex_without_probing() {
    let d = long_line();
    let projection = Counted::new(500_000_000.0, None);
    let p = plan(&d, &projection);
    let vertices: usize = p.geo.iter().map(|g| g.vertex_count()).sum();
    // Each vertex, each handle twice (extent and placement) and four corners.
    assert!(projection.calls.get() <= vertices + 2 * p.handles.len() + 4);
}

#[test]
fn chords_outside_the_viewport_are_not_refined_and_visible_ones_are() {
    let d = long_line();
    let full = Counted::new(250_000.0, None);
    let reference = plan(&d, &full);
    let start = full
        .frame
        .project(GeoPoint::new(20.0, 50.0).unwrap())
        .unwrap();
    let window = ScreenRect {
        min: ScreenPoint {
            x: start.x - 100.0,
            y: start.y - 300.0,
        },
        max: ScreenPoint {
            x: start.x + 700.0,
            y: start.y + 300.0,
        },
    };
    let clipped = Counted::new(250_000.0, Some(window));
    let p = plan(&d, &clipped);
    // Off-screen vertices are projected once, without a probed midpoint.
    assert!(clipped.calls.get() * 3 < full.calls.get() * 2);
    let reference = lines(&reference);
    let inside = |s: &ScreenPoint| {
        (window.min.x..=window.max.x).contains(&s.x) && (window.min.y..=window.max.y).contains(&s.y)
    };
    let mut checked = 0;
    for line in lines(&p) {
        for w in line.windows(2) {
            let mid = ScreenPoint {
                x: (w[0].x + w[1].x) / 2.0,
                y: (w[0].y + w[1].y) / 2.0,
            };
            if inside(&mid) {
                checked += 1;
                let d = reference
                    .iter()
                    .map(|r| distance_to(mid, r))
                    .fold(f64::INFINITY, f64::min);
                assert!(d <= 0.5 + 1e-6, "chord strays {d} px inside the viewport");
            }
        }
    }
    assert!(checked >= 3, "{checked} chords inside the viewport");
}

fn named(mut d: GraphicDefinition) -> GraphicDefinition {
    d.modifiers.designation = Some("ALPHA".to_owned());
    d
}

#[test]
fn a_graphic_far_outside_the_viewport_projects_only_its_handles_and_bounds() {
    let d = named(long_line());
    let window = ScreenRect {
        min: ScreenPoint {
            x: 0.0,
            y: -20_000.0,
        },
        max: ScreenPoint {
            x: 800.0,
            y: -19_400.0,
        },
    };
    let projection = Counted::new(250_000.0, Some(window));
    let p = plan(&d, &projection);
    assert!(p.screen.is_empty() && p.labels.is_empty());
    assert!(!p.geo.is_empty());
    assert!(p.handles.iter().all(|h| h.screen.is_some()));
    assert_eq!(projection.calls.get(), 2 * p.handles.len() + 4);
}

#[test]
fn an_overlay_plan_is_the_full_plan_without_its_parts() {
    let d = named(long_line());
    let frame = Counted::new(2_000_000.0, None);
    let full = plan(&d, &frame);
    let overlay = plan_with(&d, &frame, PlanContent::Overlay);
    assert!(overlay.geo.is_empty());
    assert!(!full.labels.is_empty());
    assert_eq!(overlay.labels, full.labels);
    assert_eq!(overlay.handles, full.handles);
    let decorations: Vec<_> = full.screen.iter().filter(|i| i.decoration).collect();
    assert_eq!(overlay.screen.iter().collect::<Vec<_>>(), decorations);
}
