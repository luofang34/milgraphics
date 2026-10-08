use super::*;

fn flat(points: &[(f64, f64)]) -> Vec<f64> {
    points.iter().flat_map(|&(x, y)| [x, y]).collect()
}

#[test]
fn straight_line_has_all_good_segments() {
    let pixels = flat(&[(0.0, 0.0), (100.0, 0.0), (200.0, 0.0)]);
    assert_eq!(get_segments(&pixels, 3.0).unwrap(), vec![true, true]);
}

#[test]
fn doubling_back_marks_the_second_segment_bad() {
    let pixels = flat(&[(0.0, 0.0), (100.0, 0.0), (0.0, 0.0)]);
    assert_eq!(get_segments(&pixels, 3.0).unwrap(), vec![true, false]);
}

#[test]
fn a_single_point_has_no_segments() {
    assert!(get_segments(&[1.0, 2.0], 3.0).is_err());
}

#[test]
fn straight_line_of_contact_is_one_partition() {
    let pixels = flat(&[(0.0, 0.0), (100.0, 0.0), (200.0, 0.0)]);
    let (parts, single) = get_lc_partitions(&pixels, 40.0).unwrap();
    assert_eq!(parts, vec![Partition { start: 0, end: 1 }]);
    assert!(single.is_empty());
}

#[test]
fn sharp_turn_starts_a_new_partition() {
    // The turn at the middle point is about 5.7 degrees.
    let pixels = flat(&[(0.0, 0.0), (100.0, 0.0), (0.0, 10.0)]);
    let (parts, single) = get_lc_partitions(&pixels, 40.0).unwrap();
    assert_eq!(
        parts,
        vec![
            Partition { start: 0, end: 0 },
            Partition { start: 1, end: 1 }
        ]
    );
    assert!(single.is_empty());
}

#[test]
fn acute_angle_too_tight_for_a_channel_becomes_a_single_line() {
    // 354.3 degrees: the arms are nearly equal, so the shorter arm laid on
    // the longer ends 9.96 pixels from its own end, under the 40 width.
    let pixels = flat(&[(0.0, 10.0), (100.0, 0.0), (0.0, 0.0)]);
    let (parts, single) = get_lc_partitions(&pixels, 40.0).unwrap();
    assert_eq!(single, vec![Partition { start: 0, end: 2 }]);
    assert_eq!(
        parts,
        vec![
            Partition { start: 0, end: -1 },
            Partition { start: 2, end: 1 }
        ]
    );
    // With a channel narrower than the gap the angle is fine.
    let (parts, single) = get_lc_partitions(&pixels, 5.0).unwrap();
    assert!(single.is_empty());
    assert_eq!(parts, vec![Partition { start: 0, end: 1 }]);
}
