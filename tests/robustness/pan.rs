//! Screen-sized patterns on lines far longer than the view stay where they
//! are on the line while the view pans.

use milgraphics::render::{LocalEquirectangular, Projection, ScreenPoint, ScreenRect};
use milgraphics::{Budget, Config, GeoPoint, construct};

use crate::fixtures::{definition, rendered, symbol};

const SCREEN: ScreenRect = ScreenRect {
    min: ScreenPoint { x: 0.0, y: 0.0 },
    max: ScreenPoint {
        x: 1600.0,
        y: 1000.0,
    },
};

/// A view of `inner` moved `dx` pixels to the right.
struct Panned {
    inner: LocalEquirectangular,
    dx: f64,
}

impl Projection for Panned {
    fn project(&self, p: GeoPoint) -> Option<ScreenPoint> {
        self.inner.project(p).map(|s| ScreenPoint {
            x: s.x + self.dx,
            y: s.y,
        })
    }
    fn unproject(&self, s: ScreenPoint) -> Option<GeoPoint> {
        self.inner.unproject(ScreenPoint {
            x: s.x - self.dx,
            y: s.y,
        })
    }
    fn viewport(&self) -> Option<ScreenRect> {
        Some(SCREEN)
    }
}

fn on_screen(p: &ScreenPoint) -> bool {
    (100.0..=1500.0).contains(&p.x) && (100.0..=900.0).contains(&p.y)
}

/// The screen points drawn in the middle of the view, moved back by `dx`.
fn drawn(entity: &str, dx: f64) -> Vec<ScreenPoint> {
    // About 6 km of line through the view, at a scale that puts it across
    // far more pixels than a line is drawn whole for.
    let points = [(19.98, 50.0), (20.0, 50.0005), (20.08, 50.0)];
    let mut d = definition(entity, &points);
    d.symbol = symbol(15, 0, entity);
    let c = construct(&d, &Config::default()).unwrap();
    let inner = LocalEquirectangular::new(19.999704, 50.000619, 100.0, 96.0);
    let plan = rendered(&c, &Panned { inner, dx }, &Budget::default()).unwrap();
    plan.screen
        .iter()
        .flat_map(|i| i.shape.points().to_vec())
        .filter(on_screen)
        .map(|p| ScreenPoint {
            x: p.x - dx,
            y: p.y,
        })
        .collect()
}

#[cfg_attr(not(target_arch = "wasm32"), test)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn patterns_on_long_lines_do_not_slide_while_panning() {
    // Antitank ditch, completed; single concertina; wire.
    for entity in ["290202", "290307", "290301"] {
        let (a, b) = (drawn(entity, 0.0), drawn(entity, 300.0));
        assert!(a.len() > 20, "{entity}: {} points on screen", a.len());
        // Points seen in both views are in the same place on the line.
        let overlap: Vec<_> = a
            .iter()
            .filter(|p| (150.0..=1150.0).contains(&p.x))
            .collect();
        let moved = overlap
            .iter()
            .filter(|p| !b.iter().any(|q| (p.x - q.x).hypot(p.y - q.y) < 1.0))
            .count();
        assert_eq!(
            moved,
            0,
            "{entity}: {moved} of {} points moved",
            overlap.len()
        );
    }
}
