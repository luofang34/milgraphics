//! Geodesics on the WGS84 ellipsoid.

use geographiclib_rs::{DirectGeodesic, Geodesic, GeodesicLine, InverseGeodesic, capability};

use crate::geo::GeoPoint;

mod rhumb;

pub(crate) use rhumb::Rhumb;

#[cfg(test)]
mod tests;

/// Geodesic calculations on WGS84.
///
/// Holds the precomputed series coefficients, so construct one per
/// construction run and pass it down rather than rebuilding it per call.
#[derive(Debug)]
pub(crate) struct Earth {
    geodesic: Geodesic,
}

/// Distance and azimuths of the geodesic between two points.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Inverse {
    /// Length in metres.
    pub(crate) distance_m: f64,
    /// Azimuth at the start, degrees clockwise from north.
    pub(crate) azimuth1: f64,
    /// Azimuth at the end, degrees clockwise from north.
    pub(crate) azimuth2: f64,
}

impl Earth {
    pub(crate) fn wgs84() -> Self {
        Self {
            geodesic: Geodesic::wgs84(),
        }
    }

    /// The point `distance_m` from `from` along initial azimuth `azimuth`.
    pub(crate) fn direct(&self, from: GeoPoint, azimuth: f64, distance_m: f64) -> GeoPoint {
        let (lat, lon, _): (f64, f64, f64) =
            self.geodesic
                .direct(from.lat(), from.lon(), azimuth, distance_m);
        clamp_point(lon, lat, from)
    }

    /// The geodesic from `a` to `b`.
    pub(crate) fn inverse(&self, a: GeoPoint, b: GeoPoint) -> Inverse {
        let (distance_m, azimuth1, azimuth2, _): (f64, f64, f64, f64) =
            self.geodesic.inverse(a.lat(), a.lon(), b.lat(), b.lon());
        Inverse {
            distance_m,
            azimuth1,
            azimuth2,
        }
    }

    /// The point a `fraction` of the way along the geodesic from `a` to `b`.
    pub(crate) fn interpolate(&self, a: GeoPoint, b: GeoPoint, fraction: f64) -> GeoPoint {
        self.line(a, b).at(fraction)
    }

    /// The geodesic from `a` to `b`, solved once for taking many points on it.
    pub(crate) fn line(&self, a: GeoPoint, b: GeoPoint) -> Line {
        let inv = self.inverse(a, b);
        Line {
            line: GeodesicLine::new(
                &self.geodesic,
                a.lat(),
                a.lon(),
                inv.azimuth1,
                Some(LINE_CAPS),
                None,
                None,
            ),
            from: a,
            distance_m: inv.distance_m,
        }
    }

    /// Points along the geodesic from `a` to `b`, `a` included and `b`
    /// excluded, spaced at most `max_step_m` apart (at least one point).
    pub(crate) fn densify(
        &self,
        a: GeoPoint,
        b: GeoPoint,
        max_step_m: f64,
        out: &mut Vec<GeoPoint>,
    ) -> usize {
        let inv = self.inverse(a, b);
        let steps = segment_count(inv.distance_m, max_step_m);
        out.push(a);
        for i in 1..steps {
            let s = inv.distance_m * i as f64 / steps as f64;
            out.push(self.direct(a, inv.azimuth1, s));
        }
        steps
    }
}

/// Number of equal pieces needed so none is longer than `max_step_m`.
pub(crate) fn segment_count(distance_m: f64, max_step_m: f64) -> usize {
    if !(distance_m.is_finite() && max_step_m.is_finite() && max_step_m > 0.0) {
        return 1;
    }
    let pieces = (distance_m / max_step_m).ceil();
    if pieces <= 1.0 {
        1
    } else if pieces >= usize::MAX as f64 {
        usize::MAX
    } else {
        pieces as usize
    }
}

/// A point from geodesic output, which is finite and in range for finite
/// input; `fallback` covers the degenerate case instead of panicking.
/// What a [`Line`] computes: positions at distances along it, as `direct` does.
const LINE_CAPS: u64 = capability::LATITUDE | capability::LONGITUDE | capability::DISTANCE_IN;

/// A geodesic between two points.
pub(crate) struct Line {
    line: GeodesicLine,
    from: GeoPoint,
    distance_m: f64,
}

impl Line {
    /// The point a `fraction` of the way along.
    pub(crate) fn at(&self, fraction: f64) -> GeoPoint {
        let (_, lat, lon, ..) =
            self.line
                ._gen_position(false, self.distance_m * fraction, LINE_CAPS);
        clamp_point(lon, lat, self.from)
    }
}

fn clamp_point(lon: f64, lat: f64, fallback: GeoPoint) -> GeoPoint {
    GeoPoint::new(lon, lat.clamp(-90.0, 90.0)).unwrap_or(fallback)
}
