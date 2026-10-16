use super::*;

/// Tip at the origin pointing west, second point 10 km east, width point
/// 1 km behind the tip and 400 m off the axis.
const STRAIGHT: [Xy; 3] = [
    Xy { x: 0.0, y: 0.0 },
    Xy {
        x: 10_000.0,
        y: 0.0,
    },
    Xy {
        x: 1_000.0,
        y: 400.0,
    },
];

#[test]
fn straight_axis_proportions() {
    let s = shape(&STRAIGHT, AxisKind::Main).unwrap();
    assert_eq!(
        s.lower,
        vec![Xy::new(10_000.0, 200.0), Xy::new(1_000.0, 200.0)]
    );
    assert_eq!(
        s.upper,
        vec![Xy::new(10_000.0, -200.0), Xy::new(1_000.0, -200.0)]
    );
    assert_eq!(
        s.head,
        vec![
            Xy::new(0.0, 0.0),
            Xy::new(1_000.0, 400.0),
            Xy::new(1_000.0, 200.0),
            Xy::new(200.0, 0.0),
            Xy::new(1_000.0, -200.0),
            Xy::new(1_000.0, -400.0),
            Xy::new(0.0, 0.0),
        ]
    );
    assert!(s.figures.is_empty());
}

#[test]
fn supporting_head_is_an_open_chevron_without_a_notch() {
    let s = shape(&STRAIGHT, AxisKind::Supporting).unwrap();
    assert_eq!(
        s.head,
        vec![
            Xy::new(1_000.0, 200.0),
            Xy::new(1_000.0, 400.0),
            Xy::new(0.0, 0.0),
            Xy::new(1_000.0, -400.0),
            Xy::new(1_000.0, -200.0),
        ]
    );
    assert!(s.figures.is_empty());
}

#[test]
fn airborne_sides_cross_before_the_head() {
    let s = shape(&STRAIGHT, AxisKind::Airborne).unwrap();
    assert_eq!(
        s.lower,
        vec![Xy::new(10_000.0, 200.0), Xy::new(1_000.0, -200.0)]
    );
    assert_eq!(
        s.upper,
        vec![Xy::new(10_000.0, -200.0), Xy::new(1_000.0, 200.0)]
    );
    // The head still joins the sides where they end.
    assert_eq!(s.head.first(), Some(&Xy::new(1_000.0, 200.0)));
    assert_eq!(s.head.last(), Some(&Xy::new(1_000.0, -200.0)));
}

#[test]
fn rotor_stands_across_the_crossing() {
    let s = shape(&STRAIGHT, AxisKind::AttackHelicopter).unwrap();
    assert_eq!(s.figures.len(), 5);
    let (stem, dashed) = &s.figures[0];
    assert!(!dashed);
    // Travel is west, so its left is south; the crossing is mid-body.
    assert_eq!(
        stem,
        &vec![Xy::new(5_500.0, 400.0), Xy::new(5_500.0, -400.0)]
    );
    assert_eq!(s.figures[1].0[1], Xy::new(5_500.0, -400.0));
}

#[test]
fn feint_chevron_is_dashed_ahead_of_the_head() {
    let s = shape(&STRAIGHT, AxisKind::Feint).unwrap();
    let [(chevron, dashed)] = s.figures.as_slice() else {
        panic!("one figure expected");
    };
    assert!(dashed);
    assert_eq!(
        chevron,
        &vec![
            Xy::new(1_000.0, 600.0),
            Xy::new(-500.0, 0.0),
            Xy::new(1_000.0, -600.0),
        ]
    );
}

#[test]
fn rear_end_stays_open() {
    for kind in [
        AxisKind::Main,
        AxisKind::Supporting,
        AxisKind::Airborne,
        AxisKind::AttackHelicopter,
        AxisKind::Feint,
    ] {
        let s = shape(&STRAIGHT, kind).unwrap();
        let tail = [s.lower[0], s.upper[0]];
        let joins_tail = |l: &[Xy]| {
            l.windows(2)
                .any(|w| tail.contains(&w[0]) && tail.contains(&w[1]))
        };
        assert!(!joins_tail(&s.lower) && !joins_tail(&s.upper) && !joins_tail(&s.head));
        assert!(s.figures.iter().all(|(f, _)| !joins_tail(f)));
    }
}

#[test]
fn bends_keep_every_centreline_point() {
    let pts = [
        Xy::new(0.0, 0.0),
        Xy::new(5_000.0, 0.0),
        Xy::new(5_000.0, 5_000.0),
        Xy::new(500.0, 100.0),
    ];
    let s = shape(&pts, AxisKind::Main).unwrap();
    // Tail, mitred bend at point 2, then the arrowhead base.
    assert_eq!(s.lower.len(), 3);
    assert_eq!(s.lower[1], Xy::new(4_950.0, 50.0));
    assert_eq!(s.upper[1], Xy::new(5_050.0, -50.0));
}

#[test]
fn degenerate_widths_are_refused() {
    let on_axis = [Xy::new(0.0, 0.0), Xy::new(10.0, 0.0), Xy::new(5.0, 0.0)];
    assert!(shape(&on_axis, AxisKind::Supporting).is_none());
    let same = [Xy::new(0.0, 0.0), Xy::new(0.0, 0.0), Xy::new(5.0, 5.0)];
    assert!(shape(&same, AxisKind::Supporting).is_none());
}
