use super::*;
use crate::definition::{ControlPoint, GraphicDefinition, GraphicId};
use crate::family::{Config, construct};
use crate::sidc::SymbolId;

/// Orthographic globe view centred on (`lon0`, `lat0`); the far hemisphere
/// is hidden, so it exercises horizon clipping.
struct Orthographic {
    lon0: f64,
    lat0: f64,
    radius_px: f64,
}

impl Projection for Orthographic {
    fn project(&self, p: GeoPoint, _h: f64) -> Option<ScreenPoint> {
        let (phi, lam) = (p.lat().to_radians(), (p.lon() - self.lon0).to_radians());
        let phi0 = self.lat0.to_radians();
        let cos_c = phi0.sin() * phi.sin() + phi0.cos() * phi.cos() * lam.cos();
        (cos_c > 0.0).then(|| ScreenPoint {
            x: self.radius_px * phi.cos() * lam.sin(),
            y: -self.radius_px * (phi0.cos() * phi.sin() - phi0.sin() * phi.cos() * lam.cos()),
        })
    }

    fn unproject(&self, _s: ScreenPoint) -> Option<GeoPoint> {
        None
    }
}

fn def(sidc: &str, points: &[(f64, f64)], t: Option<&str>) -> GraphicDefinition {
    let mut d = GraphicDefinition::new(
        GraphicId::new("g").unwrap(),
        SymbolId::parse(sidc).unwrap(),
        points
            .iter()
            .map(|&(lon, lat)| ControlPoint::ground(GeoPoint::new(lon, lat).unwrap()))
            .collect(),
    );
    d.modifiers.designation = t.map(str::to_owned);
    d
}

fn view() -> View {
    View {
        view_revision: 1,
        surface_revision: 0,
        label_font: Font::default(),
    }
}

fn plan(d: &GraphicDefinition, projection: &dyn Projection) -> RenderPlan {
    let c = construct(d, &Config::default()).unwrap();
    render(
        &c,
        &view(),
        projection,
        &FixedAdvanceMetrics::default(),
        &Budget::default(),
    )
    .unwrap()
}

const PL: &str = "11032500001403000000";
const NAI: &str = "11032500001202000000";

#[test]
fn phase_line_plan_has_both_tiers_labels_and_handles() {
    let d = def(PL, &[(20.0, 50.0), (20.1, 50.02)], Some("ALPHA"));
    let frame = LocalEquirectangular::new(19.95, 50.07, 50_000.0, 96.0);
    let p = plan(&d, &frame);
    assert_eq!(p.geo.len(), 1);
    let GeoShape::Lines(lines) = &p.geo[0].shape else {
        panic!()
    };
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].first(), Some(&[20.0, 50.0]));
    assert_eq!(lines[0].last(), Some(&[20.1, 50.02]));
    assert_eq!(p.screen.len(), 1);
    assert!(matches!(p.screen[0].shape, ScreenShape::Polyline(_)));
    let texts: Vec<_> = p.labels.iter().map(|l| l.text.as_str()).collect();
    assert_eq!(texts, ["PL ALPHA", "PL ALPHA"]);
    // Start label ends at the first vertex, end label starts at the last;
    // both share the line's upright angle.
    assert_eq!(p.labels[0].align, TextAlign::Right);
    assert_eq!(p.labels[1].align, TextAlign::Left);
    assert!((p.labels[0].rotation_deg - p.labels[1].rotation_deg).abs() < 1e-6);
    assert_eq!(p.handles.len(), 2);
    assert!(p.handles.iter().all(|h| h.screen.is_some()));
    assert!(!p.terrain_missing);
}

#[test]
fn horizon_cuts_lines_and_unfills_rings() {
    let globe = Orthographic {
        lon0: 0.0,
        lat0: 0.0,
        radius_px: 1000.0,
    };
    let d = def(PL, &[(60.0, 0.0), (120.0, 0.0)], None);
    let p = plan(&d, &globe);
    assert_eq!(p.screen.len(), 1);
    let ScreenShape::Polyline(pts) = &p.screen[0].shape else {
        panic!()
    };
    let last = pts.last().unwrap();
    assert!(
        (last.x - 1000.0).abs() < 0.01,
        "ends at the horizon: {last:?}"
    );
    assert!(pts.iter().all(|s| s.x.hypot(s.y) <= 1000.0 + 1e-6));
    assert!(
        p.labels[1].screen.is_none(),
        "end label is behind the globe"
    );
    assert!(p.terrain_missing);

    let area = def(
        NAI,
        &[(80.0, -10.0), (100.0, -10.0), (100.0, 10.0), (80.0, 10.0)],
        None,
    );
    let p = plan(&area, &globe);
    assert!(!p.screen.is_empty());
    for item in &p.screen {
        assert!(matches!(item.shape, ScreenShape::Polyline(_)));
        assert_eq!(item.fill, Fill::None);
    }
}

#[test]
fn screen_tier_follows_the_geodesic_within_tolerance() {
    let globe = Orthographic {
        lon0: 20.0,
        lat0: 50.0,
        radius_px: 20_000.0,
    };
    let d = def(PL, &[(0.0, 50.0), (40.0, 50.0)], None);
    let p = plan(&d, &globe);
    let ScreenShape::Polyline(pts) = &p.screen[0].shape else {
        panic!()
    };
    assert!(pts.len() > 3, "a 40° geodesic is subdivided on screen");
    for pair in pts.windows(2) {
        let (dx, dy) = pair[1].sub(pair[0]);
        assert!(dx.hypot(dy) > 0.0);
    }
}

#[test]
fn geographic_tier_does_not_depend_on_the_view() {
    let d = def(PL, &[(20.0, 50.0), (20.1, 50.02)], Some("A"));
    let near = plan(&d, &LocalEquirectangular::new(19.95, 50.07, 25_000.0, 96.0));
    let far = plan(
        &d,
        &LocalEquirectangular::new(19.95, 50.07, 400_000.0, 96.0),
    );
    assert_eq!(near.geo, far.geo);
    let width = |p: &RenderPlan| p.screen[0].stroke.unwrap().width_px;
    assert_eq!(width(&near), width(&far), "line width stays in pixels");
}

#[test]
fn render_budget_is_enforced() {
    let globe = Orthographic {
        lon0: 20.0,
        lat0: 50.0,
        radius_px: 1e7,
    };
    let d = def(PL, &[(0.0, 50.0), (40.0, 50.0)], None);
    let c = construct(&d, &Config::default()).unwrap();
    let tight = Budget {
        max_vertices: 50,
        ..Budget::default()
    };
    let r = render(&c, &view(), &globe, &FixedAdvanceMetrics::default(), &tight);
    assert!(matches!(
        r,
        Err(RenderError::Budget(BudgetError::Vertices { .. }))
    ));
}

#[test]
fn rendering_is_repeatable() {
    let d = def(
        NAI,
        &[(20.0, 50.0), (20.08, 50.0), (20.08, 50.05), (20.0, 50.05)],
        Some("1"),
    );
    let frame = LocalEquirectangular::new(19.95, 50.1, 50_000.0, 96.0);
    assert_eq!(plan(&d, &frame), plan(&d, &frame));
}

#[test]
fn local_frame_round_trips() {
    let frame = LocalEquirectangular::new(19.95, 50.07, 50_000.0, 96.0);
    let p = GeoPoint::new(20.0312, 50.0123).unwrap();
    let s = frame.project(p, 0.0).unwrap();
    let back = frame.unproject(s).unwrap();
    assert!((back.lon() - p.lon()).abs() < 1e-9 && (back.lat() - p.lat()).abs() < 1e-12);
    assert!((frame.metres_per_px() - 13.229_166_6).abs() < 1e-6);
}

/// `inner` turned upside down on screen.
struct Rotated180<P>(P);

impl<P: Projection> Projection for Rotated180<P> {
    fn project(&self, p: GeoPoint, h: f64) -> Option<ScreenPoint> {
        self.0
            .project(p, h)
            .map(|s| ScreenPoint { x: -s.x, y: -s.y })
    }
    fn unproject(&self, s: ScreenPoint) -> Option<GeoPoint> {
        self.0.unproject(ScreenPoint { x: -s.x, y: -s.y })
    }
    fn terrain_height_m(&self, p: GeoPoint) -> Option<f64> {
        self.0.terrain_height_m(p)
    }
}

#[test]
fn corridor_information_block_stays_outside_whatever_the_rotation() {
    let mut d = def(
        "11032500001701000000",
        &[(20.0, 50.0), (20.1, 50.03), (20.2, 50.02)],
        Some("AC1"),
    );
    d.modifiers.distances_m = vec![2000.0];
    let frame = LocalEquirectangular::new(19.95, 50.1, 50_000.0, 96.0);
    let half_px = 1000.0 / frame.metres_per_px();
    for projection in [&frame as &dyn Projection, &Rotated180(frame)] {
        let p = plan(&d, projection);
        let (a, b) = (
            projection.project(d.points[0].position, 0.0).unwrap(),
            projection.project(d.points[1].position, 0.0).unwrap(),
        );
        let (ux, uy) = {
            let (dx, dy) = b.sub(a);
            let l = dx.hypot(dy);
            (dx / l, dy / l)
        };
        for l in p.labels.iter().filter(|l| !l.text.starts_with("AC")) {
            let s = l.screen.unwrap();
            let (sin, cos) = l.rotation_deg.to_radians().sin_cos();
            let oy = l.offset_em[1] * l.font.size_px;
            let centre = ScreenPoint {
                x: s.x - oy * sin,
                y: s.y + oy * cos,
            };
            let (cx, cy) = centre.sub(a);
            let across = (cx * uy - cy * ux).abs();
            assert!(
                across > half_px + 0.5 * l.font.size_px,
                "{:?} is inside the corridor ({across:.1} px)",
                l.text
            );
            assert!(
                centre.y < a.y.max(b.y),
                "{:?} must be above the corridor on screen",
                l.text
            );
        }
    }
}
