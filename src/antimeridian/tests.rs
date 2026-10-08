use super::*;

fn pts(coords: &[(f64, f64)]) -> Vec<GeoPoint> {
    coords
        .iter()
        .map(|&(lon, lat)| GeoPoint::new(lon, lat).unwrap())
        .collect()
}

#[test]
fn lines_away_from_the_antimeridian_are_one_piece() {
    let pieces = split_line(&pts(&[(20.0, 50.0), (20.1, 50.02), (21.0, 51.0)]));
    assert_eq!(
        pieces,
        vec![vec![[20.0, 50.0], [20.1, 50.02], [21.0, 51.0]]]
    );
}

#[test]
fn eastward_crossing_splits_at_plus_and_minus_180() {
    let pieces = split_line(&pts(&[(179.0, 10.0), (-179.0, 12.0)]));
    assert_eq!(pieces.len(), 2);
    assert_eq!(pieces[0], vec![[179.0, 10.0], [180.0, 11.0]]);
    assert_eq!(pieces[1], vec![[-180.0, 11.0], [-179.0, 12.0]]);
}

#[test]
fn westward_crossing_splits_too() {
    let pieces = split_line(&pts(&[(-179.5, 0.0), (179.5, 0.0), (179.0, 1.0)]));
    assert_eq!(pieces.len(), 2);
    assert_eq!(pieces[0], vec![[-179.5, 0.0], [-180.0, 0.0]]);
    assert_eq!(pieces[1][0], [180.0, 0.0]);
    assert_eq!(pieces[1].last(), Some(&[179.0, 1.0]));
}

#[test]
fn rings_crossing_the_antimeridian_become_two_closed_rings() {
    let rings = split_ring(&pts(&[
        (179.0, 0.0),
        (-179.0, 0.0),
        (-179.0, 2.0),
        (179.0, 2.0),
    ]));
    assert_eq!(rings.len(), 2);
    for ring in &rings {
        assert_eq!(ring.first(), ring.last());
        assert!(ring.iter().all(|p| (-180.0..=180.0).contains(&p[0])));
    }
    let west = rings
        .iter()
        .find(|r| r.iter().any(|p| p[0] == 179.0))
        .unwrap();
    assert!(west.iter().all(|p| p[0] >= 179.0));
    let east = rings
        .iter()
        .find(|r| r.iter().any(|p| p[0] == -179.0))
        .unwrap();
    assert!(east.iter().all(|p| p[0] <= -179.0));
}

#[test]
fn ordinary_rings_are_closed_and_unchanged() {
    let rings = split_ring(&pts(&[
        (20.0, 50.0),
        (20.08, 50.0),
        (20.08, 50.05),
        (20.0, 50.05),
    ]));
    assert_eq!(
        rings,
        vec![vec![
            [20.0, 50.0],
            [20.08, 50.0],
            [20.08, 50.05],
            [20.0, 50.05],
            [20.0, 50.0]
        ]]
    );
}
