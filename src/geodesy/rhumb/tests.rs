use super::*;
use crate::geodesy::Earth;

fn p(lon: f64, lat: f64) -> GeoPoint {
    GeoPoint::new(lon, lat).unwrap()
}

#[test]
fn every_point_keeps_the_course() {
    let line = Rhumb::new(p(-10.0, 30.0), p(25.0, 62.0));
    for k in 1..10 {
        let at = line.at(f64::from(k) / 10.0);
        let course = Rhumb::new(p(-10.0, 30.0), at).course_deg();
        assert!((course - line.course_deg()).abs() < 1e-9, "{course}");
    }
    let end = line.at(1.0);
    assert!((end.lon() - 25.0).abs() < 1e-9 && (end.lat() - 62.0).abs() < 1e-9);
}

#[test]
fn a_parallel_runs_due_east_or_west() {
    let east = Rhumb::new(p(10.0, 50.0), p(11.0, 50.0));
    assert!((east.course_deg() - 90.0).abs() < 1e-9);
    assert!((east.at(0.5).lat() - 50.0).abs() < 1e-12);
    let west = Rhumb::new(p(11.0, 50.0), p(10.0, 50.0));
    assert!((west.course_deg() - 270.0).abs() < 1e-9);
    // One degree of longitude at 50°N on WGS84.
    assert!(
        (east.distance_m() - 71_695.0).abs() < 5.0,
        "{}",
        east.distance_m()
    );
}

#[test]
fn a_meridian_runs_due_north_and_measures_like_the_geodesic() {
    let north = Rhumb::new(p(20.0, 10.0), p(20.0, 11.0));
    assert!(north.course_deg().abs() < 1e-9);
    let geodesic = Earth::wgs84()
        .inverse(p(20.0, 10.0), p(20.0, 11.0))
        .distance_m;
    assert!((north.distance_m() - geodesic).abs() < 1.0);
}

#[test]
fn it_is_not_the_geodesic_on_a_long_east_west_run() {
    // About 100 km east at 50°N: the geodesic bows toward the pole by
    // about 230 m at its middle; the rhumb line stays on the parallel.
    let (a, b) = (p(20.0, 50.0), p(21.4, 50.0));
    let earth = Earth::wgs84();
    let geodesic = earth.interpolate(a, b, 0.5);
    let rhumb = Rhumb::new(a, b).at(0.5);
    let apart = earth.inverse(geodesic, rhumb).distance_m;
    assert!((200.0..260.0).contains(&apart), "{apart}");
    assert!(Rhumb::new(a, b).distance_m() > earth.inverse(a, b).distance_m);
}

#[test]
fn it_crosses_the_antimeridian_the_short_way() {
    let line = Rhumb::new(p(179.9, -5.0), p(-179.9, -5.0));
    assert!((line.course_deg() - 90.0).abs() < 1e-9);
    let mid = line.at(0.5);
    assert!(
        mid.lon() == -180.0 || mid.lon().abs() > 179.99,
        "{}",
        mid.lon()
    );
}

#[test]
fn the_poles_give_finite_points() {
    let line = Rhumb::new(p(0.0, 89.0), p(90.0, 90.0));
    for k in 0..=10 {
        let q = line.at(f64::from(k) / 10.0);
        assert!(q.lat().is_finite() && q.lon().is_finite());
    }
    assert!(line.distance_m().is_finite());
}
