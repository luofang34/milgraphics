//! Planar replacements for the `mdlGeodesic` calls `clsUtilityCPOF` makes.
//! Control points are pixels (y down), a distance in metres becomes
//! `metres / meters_per_pixel` pixels, and an azimuth is a compass bearing
//! (degrees clockwise from up). The arc builders keep upstream's logic, only
//! on pixel coordinates.

use crate::engine::base::Pt;

/// Number of steps along an arc (`GetGeodesicArc` samples 101 points).
const ARC_STEPS: i32 = 100;

/// `mdlGeodesic.GetAzimuth` / the `a12` of `geodesic_distance`: the bearing
/// from `from` to `to`, in (-180, 180].
pub(super) fn azimuth(from: Pt, to: Pt) -> f64 {
    let dx = to.x - from.x;
    // Subtracting from +0.0 keeps coincident points at azimuth 0, not 180.
    let up = 0.0 - (to.y - from.y);
    dx.atan2(up).to_degrees()
}

/// `mdlGeodesic.geodesic_distance`: ground metres between two pixels.
pub(super) fn distance_m(a: Pt, b: Pt, meters_per_pixel: f64) -> f64 {
    ((b.x - a.x).hypot(b.y - a.y)) * meters_per_pixel
}

/// `mdlGeodesic.geodesic_coordinate`: the point `dist_m` metres from `start`
/// along bearing `az_deg`. The new point has style 0.
pub(super) fn coordinate(start: Pt, dist_m: f64, az_deg: f64, meters_per_pixel: f64) -> Pt {
    let d = dist_m / meters_per_pixel;
    let az = az_deg.to_radians();
    Pt::new(start.x + d * az.sin(), start.y - d * az.cos())
}

/// Azimuths and distance used by both arc builders: `a12`, `a12b`, the
/// distance to the first point and whether the end points coincide.
struct Sweep {
    start: f64,
    end: f64,
    radius_m: f64,
    circle: bool,
    same_azimuth: bool,
}

fn sweep(center: Pt, pt1: Pt, pt2: Pt, mpp: f64) -> Sweep {
    let radius_m = distance_m(center, pt1, mpp);
    let a12 = azimuth(center, pt1);
    let save_azimuth = azimuth(pt1, center);
    let mut a12b = azimuth(center, pt2);
    let a21 = azimuth(pt2, center);
    let mut start = a12;
    let same_azimuth = (a21 - save_azimuth).abs() <= 1.0;
    if same_azimuth {
        if start < 360.0 {
            start += 360.0;
        }
        a12b = start + 360.0;
    }
    if a12b < 0.0 {
        a12b += 360.0;
    }
    if start < 0.0 {
        start += 360.0;
    }
    if a12b < start {
        a12b += 360.0;
    }
    Sweep {
        start,
        end: a12b,
        radius_m,
        circle: same_azimuth,
        same_azimuth,
    }
}

fn arc_points(center: Pt, s: &Sweep, mpp: f64) -> Vec<Pt> {
    (0..=ARC_STEPS)
        .map(|j| {
            let az = s.start + (f64::from(j) / 100.0) * (s.end - s.start);
            coordinate(center, s.radius_m, az, mpp)
        })
        .collect()
}

/// `mdlGeodesic.GetGeodesicArc`: the arc from `pt1` to `pt2` about `center`
/// (a full circle when they coincide), then the centre unless it is a
/// circle, then the arc's end point.
pub(super) fn geodesic_arc(center: Pt, pt1: Pt, pt2: Pt, mpp: f64) -> Vec<Pt> {
    let s = sweep(center, pt1, pt2, mpp);
    let mut out = arc_points(center, &s, mpp);
    if !s.same_azimuth {
        out.push(center);
    }
    out.push(if s.start < s.end { pt1 } else { pt2 });
    out
}

/// `mdlGeodesic.GetGeodesicArc2`: only the arc points, and whether the
/// sector is a full circle.
pub(super) fn geodesic_arc2(center: Pt, pt1: Pt, pt2: Pt, mpp: f64) -> (Vec<Pt>, bool) {
    let s = sweep(center, pt1, pt2, mpp);
    (arc_points(center, &s, mpp), s.circle)
}

/// `mdlGeodesic.getGeoEllipse`: 37 points of an ellipse with the given
/// semi-axes in metres, rotated by `rotation` degrees about `center`.
pub(super) fn geo_ellipse(
    center: Pt,
    major_m: f64,
    minor_m: f64,
    rotation: f64,
    mpp: f64,
) -> Vec<Pt> {
    let mut out: Vec<Pt> = Vec::with_capacity(37);
    for l in 1..37 {
        let factor = (10.0 * f64::from(l)).to_radians();
        let a = major_m * factor.cos();
        let b = minor_m * factor.sin();
        let lon = coordinate(center, a, 90.0, mpp);
        let lat = coordinate(center, b, 0.0, mpp);
        let pt = Pt::new(lon.x, lat.y);
        out.push(rotate(center, pt, -rotation, mpp));
    }
    if let Some(first) = out.first().copied() {
        out.push(first);
    }
    out
}

/// `mdlGeodesic.geoRotatePoint`.
fn rotate(center: Pt, pt: Pt, rotation: f64, mpp: f64) -> Pt {
    let bearing = azimuth(center, pt);
    let dist = distance_m(center, pt, mpp);
    coordinate(center, dist, bearing + rotation, mpp)
}
