//! Every declared graphic seen from a camera standing on it: ground just in
//! front of the camera projects arbitrarily far off screen, and ground
//! behind it is hidden. Whatever the graphic, every line and polygon the
//! plan draws stays within the viewport grown by its own size, so a host's
//! work per pixel of output stays bounded, and every screen position is
//! finite.

use milgraphics::render::ScreenRect;

use super::*;

const WIDTH: f64 = 1_000.0;
const HEIGHT: f64 = 800.0;
const FOCAL_PX: f64 = 800.0;
const METRES_PER_DEG: f64 = 111_320.0;

/// A pinhole camera `height_m` above flat ground at (`lon`, `lat`), facing
/// `heading_deg` and looking `pitch_deg` below the horizon.
struct Camera {
    lon: f64,
    lat: f64,
    height_m: f64,
    forward: [f64; 3],
    right: [f64; 3],
    up: [f64; 3],
}

impl Camera {
    fn new(lon: f64, lat: f64, heading_deg: f64, pitch_deg: f64) -> Self {
        let (st, ct) = heading_deg.to_radians().sin_cos();
        let (sp, cp) = pitch_deg.to_radians().sin_cos();
        Self {
            lon,
            lat,
            height_m: 300.0,
            forward: [st * cp, ct * cp, -sp],
            right: [ct, -st, 0.0],
            up: [st * sp, ct * sp, cp],
        }
    }

    fn east_north(&self, p: GeoPoint) -> [f64; 3] {
        [
            (p.lon() - self.lon) * METRES_PER_DEG * self.lat.to_radians().cos(),
            (p.lat() - self.lat) * METRES_PER_DEG,
            -self.height_m,
        ]
    }
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

impl Projection for Camera {
    fn project(&self, p: GeoPoint) -> Option<ScreenPoint> {
        let d = self.east_north(p);
        let depth = dot(d, self.forward);
        (depth > 0.0).then(|| ScreenPoint {
            x: WIDTH / 2.0 + FOCAL_PX * dot(d, self.right) / depth,
            y: HEIGHT / 2.0 - FOCAL_PX * dot(d, self.up) / depth,
        })
    }

    fn unproject(&self, s: ScreenPoint) -> Option<GeoPoint> {
        let (u, v) = (
            (s.x - WIDTH / 2.0) / FOCAL_PX,
            (HEIGHT / 2.0 - s.y) / FOCAL_PX,
        );
        let ray: Vec<f64> = (0..3)
            .map(|i| self.forward[i] + u * self.right[i] + v * self.up[i])
            .collect();
        let t = self.height_m / -ray[2];
        if !(t.is_finite() && t > 0.0) {
            return None;
        }
        let lat = self.lat + t * ray[1] / METRES_PER_DEG;
        let lon = self.lon + t * ray[0] / (METRES_PER_DEG * self.lat.to_radians().cos());
        GeoPoint::new(lon, lat).ok()
    }

    fn viewport(&self) -> Option<ScreenRect> {
        Some(viewport())
    }
}

fn viewport() -> ScreenRect {
    ScreenRect {
        min: ScreenPoint { x: 0.0, y: 0.0 },
        max: ScreenPoint {
            x: WIDTH,
            y: HEIGHT,
        },
    }
}

/// Cameras on the graphic's first point and at its middle, facing each
/// way, so parts of it run under, beside and behind the camera.
fn cameras(d: &GraphicDefinition) -> Vec<Camera> {
    let n = d.points.len() as f64;
    let (lon, lat) = d.points.iter().fold((0.0, 0.0), |(x, y), p| {
        (x + p.position.lon() / n, y + p.position.lat() / n)
    });
    let first = d.points[0].position;
    let mut out = Vec::new();
    for heading in [0.0, 90.0, 180.0, 270.0] {
        out.push(Camera::new(lon, lat, heading, 20.0));
        out.push(Camera::new(first.lon(), first.lat(), heading + 45.0, 60.0));
    }
    out
}

#[test]
fn a_camera_on_any_graphic_keeps_the_plan_near_the_screen() {
    let near = |p: ScreenPoint| {
        p.x.is_finite()
            && p.y.is_finite()
            && (-WIDTH..=2.0 * WIDTH).contains(&p.x)
            && (-WIDTH..=HEIGHT + WIDTH).contains(&p.y)
    };
    let mut checked = 0;
    for r in records() {
        let d = definition(&r);
        let Ok(c) = construct(&d, &Config::default()) else {
            continue;
        };
        for camera in cameras(&d) {
            let Ok(plan) = render(
                &c,
                &View::new(0, 0),
                &camera,
                &FixedAdvanceMetrics::default(),
            ) else {
                continue;
            };
            checked += 1;
            let case = r["case"].as_str().unwrap_or("?");
            let points = plan.screen.iter().flat_map(|i| i.shape.points().iter());
            let corners = plan
                .labels
                .iter()
                .flat_map(|l| l.corners.into_iter().flatten());
            let anchors = (plan.labels.iter().filter_map(|l| l.screen))
                .chain(plan.handles.iter().filter_map(|h| h.screen))
                .chain(plan.symbols.iter().filter_map(|s| s.screen));
            for p in points.copied() {
                assert!(near(p), "{case}: ({}, {}) is off the near box", p.x, p.y);
            }
            for p in corners.chain(anchors) {
                assert!(
                    p.x.is_finite() && p.y.is_finite(),
                    "{case}: ({}, {})",
                    p.x,
                    p.y
                );
            }
        }
    }
    assert!(checked > 1_000, "only {checked} plans checked");
}
