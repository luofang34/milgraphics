//! BBS_AREA, the BS_* shapes and TRAINING_AREA.

use super::super::inside_outside::get_inside_outside_double2;
use super::super::work::{Work, at_mut, get, set, set_style};
use crate::engine::base::{EngineError, Pt};
use crate::engine::lineutility::basics::calc_center_point_double;
use crate::engine::lineutility::bounds::{calc_mbr_points, mbr_distance};
use crate::engine::lineutility::circle::calc_circle_double;
use crate::engine::lineutility::exterior::get_exterior_points;
use crate::engine::lineutility::transform::rotate_geometry_double;

/// Java `Math.PI`-based ellipse of 37 points rotated to `azimuth - 90`
/// degrees (`getRotatedEllipsePoints`).
fn rotated_ellipse_points(
    center: Pt,
    pt_width: Pt,
    pt_height: Pt,
    azimuth: f64,
) -> Result<Vec<Pt>, EngineError> {
    use crate::engine::lineutility::basics::calc_distance_double;
    let a = calc_distance_double(center, pt_width);
    let b = calc_distance_double(center, pt_height);
    let mut e = vec![Pt::default(); 36];
    for l in 1..37 {
        let factor = (10.0 * f64::from(l)) * std::f64::consts::PI / 180.0;
        let p = at_mut(&mut e, l - 1)?;
        p.x = center.x + a * factor.cos();
        p.y = center.y + b * factor.sin();
        p.style = 0;
    }
    rotate_geometry_double(&mut e, 36, azimuth - 90.0)?;
    let first = get(&e, 0)?;
    e.push(first);
    Ok(e)
}

pub(super) fn bbs_area(w: &mut Work<'_>) -> Result<(), EngineError> {
    get_exterior_points(
        &mut w.p,
        w.save,
        w.line_type,
        false,
        get_inside_outside_double2,
    )?;
    w.ac = w.save;
    Ok(())
}

pub(super) fn bs_cross(w: &mut Work<'_>) -> Result<(), EngineError> {
    let pt0 = get(&w.p, 0)?;
    set(
        &mut w.p,
        0,
        Pt {
            x: pt0.x - 10.0,
            ..pt0
        },
    )?;
    set(
        &mut w.p,
        1,
        Pt {
            x: pt0.x + 10.0,
            style: 10,
            ..pt0
        },
    )?;
    set(
        &mut w.p,
        2,
        Pt {
            y: pt0.y + 10.0,
            ..pt0
        },
    )?;
    set(
        &mut w.p,
        3,
        Pt {
            y: pt0.y - 10.0,
            ..pt0
        },
    )?;
    w.ac = 4;
    Ok(())
}

pub(super) fn bs_rectangle(w: &mut Work<'_>) -> Result<(), EngineError> {
    let len = i32::try_from(w.p.len()).map_err(|_| EngineError::Degenerate("too many points"))?;
    calc_mbr_points(&w.p, len, &mut w.pt0, &mut w.pt2)?;
    let (pt0, pt2) = (w.pt0, w.pt2);
    let pt1 = Pt { x: pt2.x, ..pt0 };
    let pt3 = Pt { y: pt2.y, ..pt0 };
    w.p = vec![pt0, pt1, pt2, pt3, pt0];
    w.ac = 5;
    Ok(())
}

pub(super) fn bbs_rectangle(w: &mut Work<'_>) -> Result<(), EngineError> {
    let buffer = f64::from(get(&w.p, 0)?.style);
    let (p0, p1, p2, p3) = (get(&w.p, 0)?, get(&w.p, 1)?, get(&w.p, 2)?, get(&w.p, 3)?);
    w.orig = vec![p0, p1, p2, p3, p0];
    let pt0 = Pt {
        x: p0.x - buffer,
        y: p0.y - buffer,
        ..p0
    };
    let pt1 = Pt {
        x: p1.x + buffer,
        y: p1.y - buffer,
        ..p1
    };
    let pt2 = Pt {
        x: p2.x + buffer,
        y: p2.y + buffer,
        ..p2
    };
    let pt3 = Pt {
        x: p3.x - buffer,
        y: p3.y + buffer,
        ..p3
    };
    w.p = vec![pt0, pt1, pt2, pt3, pt0];
    w.save = 5;
    w.ac = 5;
    Ok(())
}

pub(super) fn bs_ellipse(w: &mut Work<'_>) -> Result<(), EngineError> {
    let azimuth = get(&w.p, 3)?.x;
    w.p = rotated_ellipse_points(get(&w.p, 0)?, get(&w.p, 1)?, get(&w.p, 2)?, azimuth)?;
    w.ac = 37;
    Ok(())
}

pub(super) fn training_area(w: &mut Work<'_>) -> Result<(), EngineError> {
    let save = w.save;
    let dpi = w.dpi;
    let d_mbr = mbr_distance(&w.p, save)?;
    w.d_mbr = d_mbr;
    let mut d = 20.0 * dpi;
    if d_mbr < 60.0 * dpi {
        d = d_mbr / 4.0;
    }
    if d < 5.0 * dpi {
        d = 5.0 * dpi;
    }
    for j in 0..save {
        set_style(&mut w.p, j, 1)?;
    }
    set_style(&mut w.p, save - 1, 5)?;
    let pt0 = calc_center_point_double(&w.p, save - 1)?;
    let mut arc = vec![Pt::default(); 26];
    calc_circle_double(pt0, d, 26, &mut arc, 0)?;
    for (i, a) in arc.iter().enumerate() {
        set(&mut w.p, save + i32::try_from(i).unwrap_or(0), *a)?;
    }
    set_style(&mut w.p, save + 25, 5)?;
    if d_mbr < 50.0 * dpi {
        d *= 0.6;
    } else {
        d = 12.0 * dpi;
    }
    let pt1 = Pt {
        y: pt0.y - d,
        style: 0,
        ..pt0
    };
    let pt2 = Pt {
        y: pt1.y + d,
        style: 5,
        ..pt1
    };
    let pt3 = Pt {
        y: pt2.y + d / 4.0 + f64::from(w.tg.line_thickness),
        style: 0,
        ..pt2
    };
    let pt4 = Pt {
        y: pt3.y + d / 4.0,
        style: 5,
        ..pt3
    };
    let mut j = save + 26;
    for p in [pt1, pt2, pt3, pt4] {
        set(&mut w.p, j, p)?;
        j += 1;
    }
    w.vbl = j;
    w.ac = j;
    Ok(())
}
