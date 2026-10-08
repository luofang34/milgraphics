//! The ten-point half-circle every flot is drawn with (`CalcNewPoint`,
//! `CalcAnglePoints`).

/// Upstream `CalcNewPoint`: the integer point `dist` pixels from
/// (`locx`, `loc_y`) along the compass `angle` in degrees (0 up, clockwise,
/// y down). An angle outside 0..=360 after one wrap yields (0, 0), as the
/// upstream default.
pub(crate) fn calc_new_point(locx: i32, loc_y: i32, angle: f64, dist: f64) -> [i32; 2] {
    let mut angle = angle;
    if angle < 0.0 {
        angle += 360.0;
    }
    if angle > 360.0 {
        angle -= 360.0;
    }
    let mut quadrant = -1;
    if (0.0..=90.0).contains(&angle) {
        quadrant = 0;
        angle = (90.0 - angle).abs() * (std::f64::consts::PI / 180.0);
    } else if 90.0 < angle && angle <= 180.0 {
        quadrant = 1;
        angle = (angle - 90.0).abs() * (std::f64::consts::PI / 180.0);
    } else if 180.0 < angle && angle <= 270.0 {
        quadrant = 2;
        angle = (270.0 - angle).abs() * (std::f64::consts::PI / 180.0);
    } else if 270.0 < angle && angle <= 360.0 {
        quadrant = 3;
        angle = (angle - 270.0).abs() * (std::f64::consts::PI / 180.0);
    }
    let m = angle.tan().abs();
    let delta_x = (dist / (1.0 + m * m).sqrt()).abs();
    let delta_y = (m * delta_x).abs();
    let (lx, ly) = (f64::from(locx), f64::from(loc_y));
    let (dx, dy) = match quadrant {
        0 => (lx + delta_x, ly - delta_y),
        1 => (lx + delta_x, ly + delta_y),
        2 => (lx - delta_x, ly + delta_y),
        3 => (lx - delta_x, ly - delta_y),
        _ => (0.0, 0.0),
    };
    [dx as i32, dy as i32]
}

/// Upstream `CalcAnglePoints`: ten points on the half circle around
/// (`locx`, `loc_y`), 20 degrees apart starting at `angle - 90`. The
/// result is laid out as upstream's `int[30]`: x and y at offsets 3j and
/// 3j+1, the third slot of each triple stays 0.
pub(crate) fn calc_angle_points(locx: i32, loc_y: i32, angle: f64, dist: f64) -> [i32; 30] {
    let mut points = [0_i32; 30];
    for (j, triple) in points.chunks_exact_mut(3).enumerate() {
        let [px, py] = calc_new_point(locx, loc_y, angle - 90.0 + 20.0 * j as f64, dist);
        if let [x, y, _] = triple {
            *x = px;
            *y = py;
        }
    }
    points
}
