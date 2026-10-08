use super::*;

#[test]
fn longitude_wraps_into_half_open_range() {
    for (input, expected) in [
        (0.0, 0.0),
        (180.0, -180.0),
        (-180.0, -180.0),
        (190.0, -170.0),
        (-190.0, 170.0),
        (540.0, -180.0),
        (-1e-300, -1e-300),
    ] {
        assert_eq!(
            GeoPoint::new(input, 0.0).unwrap().lon(),
            expected,
            "{input}"
        );
    }
}

#[test]
fn invalid_coordinates_are_rejected() {
    assert!(GeoPoint::new(f64::NAN, 0.0).is_err());
    assert!(GeoPoint::new(0.0, f64::INFINITY).is_err());
    assert!(GeoPoint::new(0.0, 90.000_001).is_err());
    assert!(GeoPoint::new(0.0, -90.0).is_ok());
}

#[test]
fn serde_validates_points() {
    let p: GeoPoint = serde_json::from_str(r#"{"lon":200.5,"lat":10.25}"#).unwrap();
    assert_eq!((p.lon(), p.lat()), (-159.5, 10.25));
    assert!(serde_json::from_str::<GeoPoint>(r#"{"lon":0,"lat":91}"#).is_err());
}

#[test]
fn altitude_datums_use_short_names() {
    let a = Altitude::new(120.5, VerticalDatum::AboveGround);
    let json = serde_json::to_string(&a).unwrap();
    assert_eq!(json, r#"{"metres":120.5,"datum":"agl"}"#);
    assert_eq!(serde_json::from_str::<Altitude>(&json).unwrap(), a);
}
