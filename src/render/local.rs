//! A flat local projection for exports, tests and reference comparisons.

use crate::geo::{GeoPoint, wrap_longitude};
use crate::render::{Projection, ScreenPoint};

/// Metres per degree of latitude in mil-sym's frame (Earth circumference / 360).
const METRES_PER_DEG_LAT: f64 = 40_075_017.0 / 360.0;
/// Inches per metre.
const INCHES_PER_METRE: f64 = 39.370_078_7;

/// Equirectangular projection anchored at a north-west corner, with x
/// scaled by the length of a degree of longitude at each point's own
/// latitude.
///
/// This is the pixel frame mil-sym-java renders tactical graphics in
/// (`GeoPixelConversion`, `PointConverter`), so placing output here makes
/// it directly comparable with the oracle fixtures. Every point is visible.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LocalEquirectangular {
    west: f64,
    north: f64,
    metres_per_px: f64,
}

impl LocalEquirectangular {
    /// The frame for map scale `1:scale` at `dpi`, with (0, 0) at
    /// (`west`, `north`).
    pub fn new(west: f64, north: f64, scale: f64, dpi: f64) -> Self {
        Self {
            west,
            north,
            metres_per_px: scale / dpi / INCHES_PER_METRE,
        }
    }

    /// Metres on the ground per pixel.
    pub fn metres_per_px(&self) -> f64 {
        self.metres_per_px
    }
}

/// Length of a degree of longitude at `lat`, by mil-sym's series.
fn metres_per_deg_lon(lat: f64) -> f64 {
    let phi = lat.to_radians();
    111_412.84 * phi.cos() - 93.5 * (3.0 * phi).cos() + 0.118 * (5.0 * phi).cos()
}

impl Projection for LocalEquirectangular {
    fn project(&self, p: GeoPoint, _height_m: f64) -> Option<ScreenPoint> {
        let dlon = wrap_longitude(p.lon() - self.west);
        Some(ScreenPoint {
            x: dlon * metres_per_deg_lon(p.lat()) / self.metres_per_px,
            y: -(p.lat() - self.north) * METRES_PER_DEG_LAT / self.metres_per_px,
        })
    }

    fn unproject(&self, s: ScreenPoint) -> Option<GeoPoint> {
        let lat = self.north - s.y * self.metres_per_px / METRES_PER_DEG_LAT;
        let per_deg = metres_per_deg_lon(lat);
        if per_deg.is_nan() || per_deg <= 0.0 {
            return None;
        }
        GeoPoint::new(self.west + s.x * self.metres_per_px / per_deg, lat).ok()
    }

    fn terrain_height_m(&self, _point: GeoPoint) -> Option<f64> {
        Some(0.0)
    }
}
