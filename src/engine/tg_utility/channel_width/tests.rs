use super::*;

#[test]
fn horizontal_last_segment_uses_the_vertical_offset() {
    // Control point 30 below a horizontal last segment: 30 * 8, and the
    // control point sits right below the segment end so the arrowhead depth
    // is zero.
    let pixels = [0.0, 0.0, 100.0, 0.0, 100.0, 30.0];
    let mut depth = 99.0;
    assert_eq!(channel_width(&pixels, &mut depth).unwrap(), 240);
    assert!(depth.abs() < 1e-9);
}

#[test]
fn near_vertical_last_segment_uses_the_horizontal_offset() {
    let pixels = [0.0, 0.0, 0.0, 100.0, 30.0, 100.0];
    let mut depth = 0.0;
    assert_eq!(channel_width(&pixels, &mut depth).unwrap(), 120);
    assert_eq!(depth, 30.0);
}

#[test]
fn diagonal_last_segment() {
    // The control point is 70.71 from the segment's line, so the width is
    // truncated 70 * 8 and the arrow depth is sqrt(100^2 - 70.71^2).
    let pixels = [0.0, 0.0, 100.0, 100.0, 100.0, 0.0];
    let mut depth = 0.0;
    assert_eq!(channel_width(&pixels, &mut depth).unwrap(), 560);
    assert!((depth - 5000.0_f64.sqrt()).abs() < 1e-9);
}

#[test]
fn fewer_than_three_points_leave_the_depth_alone() {
    let mut depth = 7.0;
    assert_eq!(channel_width(&[0.0, 0.0, 1.0, 1.0], &mut depth).unwrap(), 0);
    assert_eq!(depth, 7.0);
}

#[test]
fn tiny_widths_are_raised_to_two() {
    let pixels = [0.0, 0.0, 100.0, 0.0, 100.0, 0.1];
    let mut depth = 0.0;
    assert_eq!(channel_width(&pixels, &mut depth).unwrap(), 2);
}
