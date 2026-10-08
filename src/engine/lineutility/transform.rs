//! Port of the rotation helpers of lineutility.java.

use super::basics::{calc_center_point_double, calc_distance_double};
use crate::engine::base::{At, EngineError, Pt, idx};
use std::f64::consts::PI;

/// Upstream `RotateGeometryDoubleOrigin`: rotates the first `vbl_counter`
/// points about the first point by `l_angle` degrees (upstream's angle
/// convention, see the gamma term).
pub(crate) fn rotate_geometry_double_origin(
    pts: &mut [Pt],
    vbl_counter: i32,
    l_angle: i32,
) -> Result<(), EngineError> {
    if l_angle == 0 {
        return Ok(());
    }
    let rotate = f64::from(l_angle) * PI / 180.0;
    let center = pts.at(0)?;
    let len = pts.len();
    for j in 0..vbl_counter {
        let p = pts.at_mut(idx(j, len)?)?;
        let mut gamma = PI + ((p.y - center.y) / (p.x - center.x)).atan();
        if p.x >= center.x {
            gamma += PI;
        }
        let theta = rotate + gamma;
        let dist = calc_distance_double(*p, center);
        p.y = center.y + dist * theta.sin();
        p.x = center.x + dist * theta.cos();
    }
    Ok(())
}

/// Upstream `RotateGeometryDouble`: rotates the first `vbl_counter` points
/// about the centre of their bounding box by `l_angle` degrees.
pub(crate) fn rotate_geometry_double(
    pts: &mut [Pt],
    vbl_counter: i32,
    l_angle: f64,
) -> Result<(), EngineError> {
    if l_angle == 0.0 {
        return Ok(());
    }
    let rotate = l_angle * PI / 180.0;
    let center = calc_center_point_double(pts, vbl_counter)?;
    let len = pts.len();
    for j in 0..vbl_counter {
        let p = pts.at_mut(idx(j, len)?)?;
        let mut gamma = if p.x == center.x {
            if p.y > center.y {
                PI + PI / 2.0
            } else {
                PI / 2.0
            }
        } else {
            PI + ((p.y - center.y) / (p.x - center.x)).atan()
        };
        if p.x >= center.x {
            gamma += PI;
        }
        let theta = rotate + gamma;
        let dist = calc_distance_double(*p, center);
        p.y = center.y + dist * theta.sin();
        p.x = center.x + dist * theta.cos();
    }
    Ok(())
}
