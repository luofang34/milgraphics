//! Rhumb lines (loxodromes) on the WGS84 ellipsoid: lines of constant
//! course, which are straight on a Mercator map.

use crate::geo::{GeoPoint, wrap_longitude};

#[cfg(test)]
mod tests;

/// WGS84 semi-major axis in metres and flattening.
const A_M: f64 = 6_378_137.0;
const F: f64 = 1.0 / 298.257_223_563;
/// Latitudes are held this far from the poles, where a rhumb line spirals
/// without end and the isometric latitude is infinite.
const POLE_MARGIN_DEG: f64 = 1e-5;
/// Fixed-point steps that invert the isometric latitude; each gains about
/// two orders of magnitude, so this is past double precision.
const INVERSE_STEPS: usize = 8;

/// The rhumb line from one point to another, taking the shorter way in
/// longitude.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Rhumb {
    from: GeoPoint,
    psi: f64,
    d_psi: f64,
    d_lon_deg: f64,
    course_deg: f64,
    distance_m: f64,
}

impl Rhumb {
    pub(crate) fn new(from: GeoPoint, to: GeoPoint) -> Self {
        let (lat1, lat2) = (held(from.lat()), held(to.lat()));
        let psi = isometric(lat1.to_radians());
        let d_psi = isometric(lat2.to_radians()) - psi;
        let d_lon_deg = wrap_longitude(to.lon() - from.lon());
        let d_lon = d_lon_deg.to_radians();
        let course_deg = d_lon.atan2(d_psi).to_degrees().rem_euclid(360.0);
        let distance_m = if d_psi.abs() > 1e-12 {
            // Meridian distance over the cosine of the course.
            let mid = ((lat1 + lat2) / 2.0).to_radians();
            let d_m = meridian_radius(mid) * (lat2 - lat1).to_radians();
            (d_m * (1.0 + (d_lon / d_psi).powi(2)).sqrt()).abs()
        } else {
            (d_lon * parallel_radius(lat1.to_radians())).abs()
        };
        Self {
            from,
            psi,
            d_psi,
            d_lon_deg,
            course_deg,
            distance_m,
        }
    }

    /// Constant course, degrees clockwise from north in `[0, 360)`.
    pub(crate) fn course_deg(&self) -> f64 {
        self.course_deg
    }

    /// Length in metres.
    pub(crate) fn distance_m(&self) -> f64 {
        self.distance_m
    }

    /// The point a `fraction` of the way along in longitude and isometric
    /// latitude, both of which change linearly along a rhumb line.
    pub(crate) fn at(&self, fraction: f64) -> GeoPoint {
        let lat = latitude(self.psi + fraction * self.d_psi).to_degrees();
        let lon = self.from.lon() + fraction * self.d_lon_deg;
        GeoPoint::new(lon, lat.clamp(-90.0, 90.0)).unwrap_or(self.from)
    }
}

fn eccentricity() -> f64 {
    (F * (2.0 - F)).sqrt()
}

fn held(lat_deg: f64) -> f64 {
    lat_deg.clamp(-90.0 + POLE_MARGIN_DEG, 90.0 - POLE_MARGIN_DEG)
}

/// Isometric latitude ψ of geodetic latitude `phi` (radians).
fn isometric(phi: f64) -> f64 {
    let e = eccentricity();
    phi.sin().atanh() - e * (e * phi.sin()).atanh()
}

/// Geodetic latitude (radians) of isometric latitude `psi`.
fn latitude(psi: f64) -> f64 {
    let e = eccentricity();
    let mut phi = psi.tanh().asin();
    for _ in 0..INVERSE_STEPS {
        phi = (psi + e * (e * phi.sin()).atanh()).tanh().asin();
    }
    phi
}

/// Radius of curvature in the meridian at `phi` (radians).
fn meridian_radius(phi: f64) -> f64 {
    let e2 = F * (2.0 - F);
    A_M * (1.0 - e2) / (1.0 - e2 * phi.sin().powi(2)).powf(1.5)
}

/// Radius of the parallel at `phi` (radians).
fn parallel_radius(phi: f64) -> f64 {
    let e2 = F * (2.0 - F);
    A_M * phi.cos() / (1.0 - e2 * phi.sin().powi(2)).sqrt()
}
