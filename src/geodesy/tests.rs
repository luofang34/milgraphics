use super::*;

fn p(lon: f64, lat: f64) -> GeoPoint {
    GeoPoint::new(lon, lat).unwrap()
}

#[test]
fn direct_matches_karney_reference() {
    // GeographicLib 2.1 (Python), Geodesic.WGS84.Direct(40.6, -73.8, 45, 10000e3).
    let earth = Earth::wgs84();
    let q = earth.direct(p(-73.8, 40.6), 45.0, 10_000e3);
    assert!((q.lat() - 32.642_844_327_605_516).abs() < 1e-11);
    assert!((q.lon() - 49.011_039_583_224_175).abs() < 1e-11);
}

#[test]
fn inverse_closes_direct_across_the_antimeridian() {
    let earth = Earth::wgs84();
    let a = p(179.9, 0.0);
    let b = earth.direct(a, 90.0, 50e3);
    assert!(b.lon() < -179.0);
    let inv = earth.inverse(a, b);
    assert!((inv.distance_m - 50e3).abs() < 1e-6);
    assert!((inv.azimuth1 - 90.0).abs() < 1e-9);
}

#[test]
fn densify_spacing_and_endpoints() {
    let earth = Earth::wgs84();
    let (a, b) = (p(20.0, 50.0), p(21.0, 50.0));
    let mut out = Vec::new();
    let steps = earth.densify(a, b, 10_000.0, &mut out);
    assert_eq!(out.len(), steps);
    assert_eq!(out.first(), Some(&a));
    let total = earth.inverse(a, b).distance_m;
    assert_eq!(steps, (total / 10_000.0).ceil() as usize);
    for pair in out.windows(2) {
        assert!(earth.inverse(pair[0], pair[1]).distance_m <= 10_000.0 + 1e-6);
    }
}

#[test]
fn degenerate_spacing_yields_one_segment() {
    assert_eq!(segment_count(0.0, 10.0), 1);
    assert_eq!(segment_count(100.0, 0.0), 1);
    assert_eq!(segment_count(f64::NAN, 10.0), 1);
    assert_eq!(segment_count(100.0, 10.0), 10);
}

#[test]
fn interpolate_halfway_is_equidistant() {
    let earth = Earth::wgs84();
    let (a, b) = (p(0.0, 0.0), p(10.0, 10.0));
    let m = earth.interpolate(a, b, 0.5);
    let d1 = earth.inverse(a, m).distance_m;
    let d2 = earth.inverse(m, b).distance_m;
    assert!((d1 - d2).abs() < 1e-6);
}
