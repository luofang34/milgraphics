use super::*;

/// Flat view over ground that ripples on screen with a period of half the
/// test line, so the ripple is zero at the line's ends and middle.
struct Ridged {
    max_chord_px: Option<f64>,
}

impl Ridged {
    const PX_PER_DEG: f64 = 10_000.0;
    const RIPPLE_PX: f64 = 30.0;

    fn ripple(lon: f64) -> f64 {
        Self::RIPPLE_PX * (4.0 * std::f64::consts::PI * (lon - 20.0) / 0.1).sin()
    }
}

impl Projection for Ridged {
    fn project(&self, p: GeoPoint) -> Option<ScreenPoint> {
        Some(ScreenPoint {
            x: (p.lon() - 19.9) * Self::PX_PER_DEG,
            y: (50.1 - p.lat()) * Self::PX_PER_DEG + Self::ripple(p.lon()),
        })
    }

    fn unproject(&self, _s: ScreenPoint) -> Option<GeoPoint> {
        None
    }

    fn max_chord_px(&self) -> Option<f64> {
        self.max_chord_px
    }
}

#[test]
fn a_projection_onto_terrain_is_sampled_between_its_chord_ends() {
    let d = def(PL, &[(20.0, 50.0), (20.1, 50.0)], None);
    let line = |max_chord_px| {
        let p = plan(&d, &Ridged { max_chord_px });
        let ScreenShape::Polyline(pts) = p.screen[0].shape.clone() else {
            panic!()
        };
        pts
    };
    let deviation = |pts: &[ScreenPoint]| {
        pts.iter()
            .map(|s| {
                let lon = s.x / Ridged::PX_PER_DEG + 19.9;
                (s.y - (0.1 * Ridged::PX_PER_DEG + Ridged::ripple(lon))).abs()
            })
            .fold(0.0_f64, f64::max)
    };
    let trusting = line(None);
    assert_eq!(trusting.len(), 2, "the midpoint lies on the chord");
    let sampled = line(Some(8.0));
    for pair in sampled.windows(2) {
        let (dx, dy) = pair[1].sub(pair[0]);
        assert!(dx.hypot(dy) <= 8.0, "chord of {} px", dx.hypot(dy));
    }
    // Vertices lie on the ground (the geodesic bows about 0.1 px north of
    // the parallel measured against), and chords of 8 px follow a 30 px
    // ripple with a 500 px period to within a fraction of a pixel.
    assert!(deviation(&sampled) < 0.2);
    let sag = sampled
        .windows(2)
        .map(|w| {
            let mid_x = (w[0].x + w[1].x) / 2.0;
            let lon = mid_x / Ridged::PX_PER_DEG + 19.9;
            let ground = 0.1 * Ridged::PX_PER_DEG + Ridged::ripple(lon);
            ((w[0].y + w[1].y) / 2.0 - ground).abs()
        })
        .fold(0.0_f64, f64::max);
    assert!(sag < 0.5, "chords sag {sag} px off the ground");
}

const FLOT: &str = "11032500001401000000";

/// Every screen-tier vertex of a plan's decorations.
fn decoration_vertices(p: &RenderPlan) -> Vec<ScreenPoint> {
    p.screen
        .iter()
        .filter(|i| i.decoration)
        .flat_map(|i| i.shape.points().iter().copied())
        .collect()
}

#[test]
fn pixel_sized_decorations_follow_the_ground_between_control_points() {
    let d = def(FLOT, &[(20.0, 50.0), (20.1, 50.0)], None);
    let p = plan(
        &d,
        &Ridged {
            max_chord_px: Some(8.0),
        },
    );
    let scallops = decoration_vertices(&p);
    assert!(!scallops.is_empty());
    // Where the ground has fallen most on screen, an eighth of the way
    // along, the scallops have moved with it rather than running straight
    // between the two control points.
    let eighth_x = 0.1125 * Ridged::PX_PER_DEG;
    let near_eighth: Vec<f64> = scallops
        .iter()
        .filter(|s| (s.x - eighth_x).abs() < 20.0)
        .map(|s| s.y)
        .collect();
    assert!(!near_eighth.is_empty());
    let mean = near_eighth.iter().sum::<f64>() / near_eighth.len() as f64;
    let ground = 0.1 * Ridged::PX_PER_DEG + Ridged::ripple(20.0125);
    assert!(
        (mean - ground).abs() < Ridged::RIPPLE_PX / 2.0,
        "scallops at y {mean}, ground at {ground}"
    );
}

#[test]
fn a_decorated_line_running_over_the_horizon_keeps_its_visible_part() {
    let globe = Orthographic {
        lon0: 20.0,
        lat0: 50.0,
        radius_px: 1_000.0,
    };
    let d = def(FLOT, &[(20.0, 50.0), (20.0, -55.0)], None);
    assert!(globe.project(GeoPoint::new(20.0, -55.0).unwrap()).is_none());
    let p = plan(&d, &globe);
    let scallops = decoration_vertices(&p);
    assert!(
        scallops.len() > 10,
        "the near side is drawn although the far end is hidden"
    );
    assert!(scallops.iter().all(|s| s.x.is_finite() && s.y.is_finite()));
    // Nothing is drawn past the horizon, the globe's edge on screen.
    assert!(
        scallops
            .iter()
            .all(|s| s.x.hypot(s.y) <= globe.radius_px + 1e-6)
    );
}
