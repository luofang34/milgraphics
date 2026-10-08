use super::*;

#[test]
fn plane_round_trips_and_keeps_distances_from_origin() {
    let earth = Earth::wgs84();
    let origin = GeoPoint::new(20.0, 50.0).unwrap();
    let plane = LocalPlane::new(&earth, origin);
    let p = GeoPoint::new(20.1, 50.03).unwrap();
    let xy = plane.to_xy(p);
    assert!((xy.len() - earth.inverse(origin, p).distance_m).abs() < 1e-6);
    let back = plane.to_geo(xy);
    assert!((back.lon() - p.lon()).abs() < 1e-10 && (back.lat() - p.lat()).abs() < 1e-10);
    assert_eq!(plane.to_geo(Xy::new(0.0, 0.0)), origin);
}

#[test]
fn mitred_offset_of_a_right_angle() {
    let pts = [Xy::new(0.0, 0.0), Xy::new(10.0, 0.0), Xy::new(10.0, 10.0)];
    let left = offset_polyline(&pts, 1.0).unwrap();
    assert_eq!(
        left,
        vec![Xy::new(0.0, 1.0), Xy::new(9.0, 1.0), Xy::new(9.0, 10.0)]
    );
    let right = offset_polyline(&pts, -1.0).unwrap();
    assert_eq!(
        right,
        vec![Xy::new(0.0, -1.0), Xy::new(11.0, -1.0), Xy::new(11.0, 10.0)]
    );
}

#[test]
fn straight_joins_and_duplicates_are_handled() {
    let pts = [
        Xy::new(0.0, 0.0),
        Xy::new(5.0, 0.0),
        Xy::new(5.0, 0.0),
        Xy::new(10.0, 0.0),
    ];
    let off = offset_polyline(&pts, 2.0).unwrap();
    assert_eq!(
        off,
        vec![Xy::new(0.0, 2.0), Xy::new(5.0, 2.0), Xy::new(10.0, 2.0)]
    );
    assert!(offset_polyline(&[Xy::new(1.0, 1.0), Xy::new(1.0, 1.0)], 1.0).is_none());
}

#[test]
fn parallel_lines_do_not_intersect() {
    let u = Xy::new(1.0, 0.0);
    assert!(intersect(Xy::new(0.0, 0.0), u, Xy::new(0.0, 1.0), u).is_none());
    assert_eq!(
        intersect(Xy::new(0.0, 0.0), u, Xy::new(3.0, -1.0), Xy::new(0.0, 1.0)),
        Some(Xy::new(3.0, 0.0))
    );
}
