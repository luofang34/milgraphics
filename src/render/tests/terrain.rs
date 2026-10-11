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

#[test]
fn a_range_fan_pointer_follows_the_ground() {
    let mut d = def("11032500002422000000", &[(20.0, 50.0)], None);
    // A 90° bearing and a 6.5 km range put the pointer's far end about 0.1°
    // east, over two periods of the ripple.
    d.modifiers.distances_m = vec![0.0, 6_500.0];
    d.modifiers.azimuths_deg = vec![80.0, 100.0];
    let p = plan(
        &d,
        &Ridged {
            max_chord_px: Some(8.0),
        },
    );
    let pointer = p
        .screen
        .iter()
        .find(|i| i.role == PartRole::Orientation)
        .expect("the fan has a pointer");
    let pts = pointer.shape.points();
    assert!(pts.len() > 6, "the shaft is more than one chord");
    let eighth_x = 0.1125 * Ridged::PX_PER_DEG;
    let at = pts
        .iter()
        .min_by(|a, b| (a.x - eighth_x).abs().total_cmp(&(b.x - eighth_x).abs()))
        .unwrap();
    let ground = 0.1 * Ridged::PX_PER_DEG + Ridged::ripple(20.0125);
    assert!(
        (at.y - ground).abs() < Ridged::RIPPLE_PX / 2.0,
        "shaft at y {}, ground at {ground}",
        at.y
    );
}

/// A view tilted toward the north: ground distances north–south show at a
/// quarter of their east–west size, as a camera pitched about 75° shows
/// them.
struct Tilted;

impl Tilted {
    const PX_PER_DEG: f64 = 20_000.0;
}

impl Projection for Tilted {
    fn project(&self, p: GeoPoint) -> Option<ScreenPoint> {
        Some(ScreenPoint {
            x: (p.lon() - 19.95) * Self::PX_PER_DEG,
            y: (50.05 - p.lat()) * Self::PX_PER_DEG * 0.25 / 50.0_f64.to_radians().cos(),
        })
    }

    fn unproject(&self, _s: ScreenPoint) -> Option<GeoPoint> {
        None
    }
}

/// Where a label's text sits on screen: its anchor moved by its offset.
fn text_position(l: &Label) -> ScreenPoint {
    let s = l.screen.unwrap();
    let em = l.font.size_px;
    let (sin, cos) = l.rotation_deg.to_radians().sin_cos();
    let (x, y) = (l.offset_em[0] * em, l.offset_em[1] * em);
    ScreenPoint {
        x: s.x + x * cos - y * sin,
        y: s.y + x * sin + y * cos,
    }
}

#[test]
fn stacked_labels_keep_their_spacing_in_a_tilted_view() {
    let mut d = def("11032500002407030000", &[(20.0, 50.0), (20.1, 50.0)], None);
    d.modifiers.target_number = Some("AP1".into());
    d.modifiers.designation2 = Some("T2Y".into());
    d.modifiers.equipment_type = Some("VV".into());
    let p = plan(&d, &Tilted);
    let stack: Vec<ScreenPoint> = p.labels.iter().map(text_position).collect();
    assert!(stack.len() >= 3, "{} labels", stack.len());
    let size = p.labels[0].font.size_px;
    for pair in stack.windows(2) {
        let (dx, dy) = pair[1].sub(pair[0]);
        assert!(
            dx.hypot(dy) >= 0.95 * size,
            "lines {} px apart at a {size} px font",
            dx.hypot(dy)
        );
    }
}
